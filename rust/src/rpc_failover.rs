//! RPC failover: automatic switch to a backup Solana RPC endpoint on
//! infrastructure errors, with 120 s cooldown recovery to primary.

use std::time::Duration;

use crate::error::HttpError;

pub const FAST_RETRY_DELAY: Duration = Duration::from_millis(100);
pub const COOLDOWN_DURATION: Duration = Duration::from_secs(120);

// ── Cross-platform timestamp ────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Timestamp(std::time::Instant);

#[cfg(not(target_arch = "wasm32"))]
impl Timestamp {
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Timestamp(f64);

#[cfg(target_arch = "wasm32")]
impl Timestamp {
    pub fn now() -> Self {
        Self(js_sys::Date::now())
    }

    pub fn elapsed(&self) -> Duration {
        let elapsed_ms = (js_sys::Date::now() - self.0).max(0.0);
        Duration::from_millis(elapsed_ms as u64)
    }
}

// ── Failover types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveRpc {
    Primary,
    Backup,
}

impl ActiveRpc {
    pub fn other(self) -> ActiveRpc {
        match self {
            ActiveRpc::Primary => ActiveRpc::Backup,
            ActiveRpc::Backup => ActiveRpc::Primary,
        }
    }
}

pub struct RpcFailoverState {
    pub(crate) active: ActiveRpc,
    pub(crate) flipped_to_backup_at: Option<Timestamp>,
}

impl RpcFailoverState {
    pub fn new() -> Self {
        Self {
            active: ActiveRpc::Primary,
            flipped_to_backup_at: None,
        }
    }

    pub fn active(&self) -> ActiveRpc {
        self.active
    }

    pub fn maybe_recover_to_primary(&mut self) {
        if self.active == ActiveRpc::Backup {
            if let Some(flipped_at) = self.flipped_to_backup_at {
                if flipped_at.elapsed() >= COOLDOWN_DURATION {
                    self.active = ActiveRpc::Primary;
                    self.flipped_to_backup_at = None;
                }
            }
        }
    }

    pub fn flip_to_backup(&mut self) {
        self.active = ActiveRpc::Backup;
        self.flipped_to_backup_at = Some(Timestamp::now());
    }

    pub fn flip_to_primary(&mut self) {
        self.active = ActiveRpc::Primary;
        self.flipped_to_backup_at = None;
    }

    pub fn flip_to(&mut self, target: ActiveRpc) {
        match target {
            ActiveRpc::Primary => self.flip_to_primary(),
            ActiveRpc::Backup => self.flip_to_backup(),
        }
    }
}

// ── Generic failover executor ───────────────────────────────────────────────

/// Execute an operation with fast retry + automatic failover.
///
/// 1. Cooldown check — maybe recover to primary
/// 2. `try_on(active)` — success or non-infra error returns immediately
/// 3. Infra error → 100 ms delay → `try_on(active)` again
/// 4. Still infra + `has_backup` → `try_on(other)`:
///    - Success → flip state → return Ok
///    - Non-infra error → flip (endpoint is reachable) → return Err
///    - Infra error → don't flip → return Err
pub async fn with_failover<T, E, TryFn, Fut, IsInfraFn>(
    failover_state: &async_lock::RwLock<RpcFailoverState>,
    try_on: TryFn,
    has_backup: bool,
    is_infra: IsInfraFn,
) -> Result<T, E>
where
    TryFn: Fn(ActiveRpc) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    IsInfraFn: Fn(&E) -> bool,
    E: std::fmt::Display,
{
    let original_active = {
        let mut state = failover_state.write().await;
        state.maybe_recover_to_primary();
        state.active()
    };

    // First attempt on active endpoint.
    match try_on(original_active).await {
        Ok(result) => return Ok(result),
        Err(error) if !is_infra(&error) => return Err(error),
        Err(first_error) => {
            tracing::warn!("RPC infra error: {first_error}");

            // Fast retry on the same endpoint.
            futures_timer::Delay::new(FAST_RETRY_DELAY).await;
            match try_on(original_active).await {
                Ok(result) => return Ok(result),
                Err(error) if !is_infra(&error) => return Err(error),
                Err(retry_error) => {
                    tracing::warn!("RPC retry failed: {retry_error}");

                    if !has_backup {
                        return Err(retry_error);
                    }

                    // Try the other endpoint.
                    let other = original_active.other();
                    tracing::info!("Failing over RPC to {other:?}");
                    match try_on(other).await {
                        Ok(result) => {
                            failover_state.write().await.flip_to(other);
                            return Ok(result);
                        }
                        Err(backup_error) => {
                            if !is_infra(&backup_error) {
                                failover_state.write().await.flip_to(other);
                            }
                            return Err(backup_error);
                        }
                    }
                }
            }
        }
    }
}

// ── Infrastructure error classifiers ────────────────────────────────────────

pub fn is_infrastructure_error_http(error: &HttpError) -> bool {
    match error {
        // Any reqwest error is transport-level — HTTP status errors have
        // their own HttpError variants (ServerError, RateLimited, etc.).
        #[cfg(feature = "http")]
        HttpError::Reqwest(_) => true,
        HttpError::ServerError { status, .. } => matches!(status, 502 | 503 | 504),
        HttpError::Timeout => true,
        _ => false,
    }
}

#[cfg(feature = "solana-rpc")]
pub fn is_infrastructure_error_solana(error: &solana_client::client_error::ClientError) -> bool {
    use solana_client::client_error::ClientErrorKind;
    match error.kind() {
        ClientErrorKind::Io(_) => true,
        // Any reqwest error in the solana client is transport-level —
        // application errors surface through RpcError / TransactionError.
        ClientErrorKind::Reqwest(_) => true,
        _ => false,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_primary() {
        let state = RpcFailoverState::new();
        assert_eq!(state.active(), ActiveRpc::Primary);
        assert!(state.flipped_to_backup_at.is_none());
    }

    #[test]
    fn flip_to_backup_records_timestamp() {
        let mut state = RpcFailoverState::new();
        state.flip_to_backup();
        assert_eq!(state.active(), ActiveRpc::Backup);
        assert!(state.flipped_to_backup_at.is_some());
    }

    #[test]
    fn flip_to_primary_clears_timestamp() {
        let mut state = RpcFailoverState::new();
        state.flip_to_backup();
        state.flip_to_primary();
        assert_eq!(state.active(), ActiveRpc::Primary);
        assert!(state.flipped_to_backup_at.is_none());
    }

    #[test]
    fn no_recovery_before_cooldown() {
        let mut state = RpcFailoverState::new();
        state.flip_to_backup();
        state.maybe_recover_to_primary();
        assert_eq!(state.active(), ActiveRpc::Backup);
    }

    #[test]
    fn recovery_when_primary_never_flipped() {
        let mut state = RpcFailoverState::new();
        state.maybe_recover_to_primary();
        assert_eq!(state.active(), ActiveRpc::Primary);
    }

    #[test]
    fn is_infra_error_timeout() {
        assert!(is_infrastructure_error_http(&HttpError::Timeout));
    }

    #[test]
    fn is_infra_error_server_502() {
        assert!(is_infrastructure_error_http(&HttpError::ServerError {
            status: 502,
            body: String::new(),
        }));
    }

    #[test]
    fn is_infra_error_server_503() {
        assert!(is_infrastructure_error_http(&HttpError::ServerError {
            status: 503,
            body: String::new(),
        }));
    }

    #[test]
    fn is_infra_error_server_504() {
        assert!(is_infrastructure_error_http(&HttpError::ServerError {
            status: 504,
            body: String::new(),
        }));
    }

    #[test]
    fn not_infra_error_rate_limited() {
        assert!(!is_infrastructure_error_http(&HttpError::RateLimited {
            retry_after_ms: None,
        }));
    }

    #[test]
    fn not_infra_error_bad_request() {
        assert!(!is_infrastructure_error_http(&HttpError::BadRequest(
            "bad".into()
        )));
    }

    #[test]
    fn not_infra_error_unauthorized() {
        assert!(!is_infrastructure_error_http(&HttpError::Unauthorized));
    }

    #[test]
    fn not_infra_error_not_found() {
        assert!(!is_infrastructure_error_http(&HttpError::NotFound(
            "missing".into()
        )));
    }

    #[test]
    fn not_infra_error_server_500() {
        assert!(!is_infrastructure_error_http(&HttpError::ServerError {
            status: 500,
            body: String::new(),
        }));
    }

    #[test]
    fn not_infra_error_max_retries() {
        assert!(!is_infrastructure_error_http(
            &HttpError::MaxRetriesExceeded {
                attempts: 3,
                last_error: "timeout".into(),
            }
        ));
    }
}
