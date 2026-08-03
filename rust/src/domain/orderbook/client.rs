//! Orderbooks sub-client — depth and on-chain orderbook operations.

use crate::client::LightconeClient;
use crate::domain::orderbook::aggregation::BookAggregation;
use crate::domain::orderbook::wire::{DecimalsResponse, OrderbookDepthResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{CloseOrderbookAltParams, CloseOrderbookParams};
use async_lock::OnceCell;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::sync::Arc;

pub struct Orderbooks<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orderbooks<'a> {
    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Orderbook PDA.
    pub fn pda(&self, mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
        crate::program::pda::get_orderbook_pda(mint_a, mint_b, &self.client.program_id).0
    }

    // ── HTTP methods ─────────────────────────────────────────────────────

    /// Get live orderbook depth, optionally aggregated (Hyperliquid-style).
    ///
    /// `depth` is capped server-side at 20 levels per side (omitted, `0`, or
    /// `>20` all serve 20). Pass [`BookAggregation::FULL`] for the raw book.
    /// Invalid aggregation combinations are rejected client-side before any
    /// request is made (the server would 400 with `INVALID_ORDERBOOK_QUERY`),
    /// and unknown query params are rejected server-side — only `depth`,
    /// `nSigFigs`, and `mantissa` are ever sent.
    pub async fn get(
        &self,
        orderbook_id: &str,
        depth: Option<u32>,
        aggregation: BookAggregation,
    ) -> Result<OrderbookDepthResponse, SdkError> {
        let aggregation = BookAggregation::validate(aggregation.n_sig_figs, aggregation.mantissa)
            .map_err(|message| SdkError::Validation(message.to_string()))?;
        let url = format!(
            "{}/api/orderbook/{}",
            self.client.http.base_url(),
            orderbook_id
        );
        let mut query = Vec::new();
        if let Some(depth) = depth {
            query.push(("depth", depth.to_string()));
        }
        if let Some(n_sig_figs) = aggregation.n_sig_figs {
            query.push(("nSigFigs", n_sig_figs.to_string()));
        }
        if let Some(mantissa) = aggregation.mantissa {
            query.push(("mantissa", mantissa.to_string()));
        }
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Fetch and permanently cache immutable trading rules for an active book.
    /// Failed requests are not cached.
    pub async fn decimals(&self, orderbook_id: &str) -> Result<DecimalsResponse, SdkError> {
        let cached = {
            let cache = self.client.orderbook_rules.read().await;
            cache.get(orderbook_id).cloned()
        };
        let cell = if let Some(cell) = cached {
            cell
        } else {
            let mut cache = self.client.orderbook_rules.write().await;
            Arc::clone(
                cache
                    .entry(orderbook_id.to_string())
                    .or_insert_with(|| Arc::new(OnceCell::new())),
            )
        };

        let rules = cell
            .get_or_try_init(|| async {
                let url = format!(
                    "{}/api/orderbooks/{}/decimals",
                    self.client.http.base_url(),
                    orderbook_id
                );
                let rules: DecimalsResponse =
                    self.client.http.get(&url, RetryPolicy::Idempotent).await?;
                rules
                    .validate_for_orderbook(orderbook_id)
                    .map_err(crate::program::error::SdkError::from)?;
                Ok::<_, SdkError>(rules)
            })
            .await?;

        Ok(rules.clone())
    }

    /// Remove one orderbook's cached rules. An in-flight fetch may still
    /// complete for its existing caller, but will not be reinserted.
    pub async fn invalidate_decimals(&self, orderbook_id: &str) {
        self.client
            .orderbook_rules
            .write()
            .await
            .remove(orderbook_id);
    }

    /// Remove every cached orderbook-rules entry.
    pub async fn clear_decimals_cache(&self) {
        self.client.orderbook_rules.write().await.clear();
    }

    /// Build CloseOrderbookAlt instruction.
    pub fn close_orderbook_alt_ix(&self, params: &CloseOrderbookAltParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_close_orderbook_alt_ix(params, pid)
    }

    /// Build CloseOrderbookAlt transaction.
    pub fn close_orderbook_alt_tx(
        &self,
        params: CloseOrderbookAltParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_orderbook_alt_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build CloseOrderbook instruction.
    pub fn close_orderbook_ix(&self, params: &CloseOrderbookParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_close_orderbook_ix(params, pid)
    }

    /// Build CloseOrderbook transaction.
    pub fn close_orderbook_tx(
        &self,
        params: CloseOrderbookParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_orderbook_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::client::LightconeClient;
    use crate::program::error::SdkError as ProgramSdkError;
    use crate::shared::ScalingError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::{timeout, Duration};

    async fn respond_with_rules(mut socket: TcpStream, orderbook_id: &str) {
        let mut request = [0_u8; 4096];
        let bytes_read = socket.read(&mut request).await.unwrap();
        assert!(bytes_read > 0);

        let body = format!(
            r#"{{"status":"success","body":{{"orderbook_id":"{orderbook_id}","base_decimals":8,"quote_decimals":6,"price_decimals":4,"trading_rules":{{"base_size_decimals":5,"max_price_decimals":1,"max_price_significant_figures":5,"integer_prices_always_allowed":true,"price_quantum":"0.1000","price_quantum_raw":"1000","base_size_quantum":"0.00001000","base_size_quantum_raw":"1000"}}}}}}"#
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    }

    async fn spawn_rules_server(response_orderbook_ids: Vec<&str>) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(AtomicUsize::new(0));
        let server_requests = Arc::clone(&requests);
        let response_orderbook_ids = response_orderbook_ids
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();

        tokio::spawn(async move {
            for orderbook_id in response_orderbook_ids {
                let (socket, _) = listener.accept().await.unwrap();
                server_requests.fetch_add(1, Ordering::SeqCst);
                respond_with_rules(socket, &orderbook_id).await;
            }
        });

        (format!("http://{address}"), requests)
    }

    #[tokio::test]
    async fn concurrent_rule_discovery_uses_one_request() {
        let (base_url, requests) = spawn_rules_server(vec!["ob"]).await;
        let client = LightconeClient::builder()
            .base_url(&base_url)
            .build()
            .unwrap();
        let orderbooks = client.orderbooks();

        let (first, second) = timeout(Duration::from_secs(2), async {
            tokio::join!(orderbooks.decimals("ob"), orderbooks.decimals("ob"))
        })
        .await
        .unwrap();

        assert_eq!(first.unwrap(), second.unwrap());
        assert_eq!(requests.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn different_orderbooks_do_not_block_each_others_fetches() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            // Wait for both connections before answering either. A cache lock
            // held across the first HTTP request would deadlock this server.
            let (first, _) = listener.accept().await.unwrap();
            let (second, _) = listener.accept().await.unwrap();
            tokio::join!(
                respond_with_rules(first, "first"),
                respond_with_rules(second, "second")
            );
        });
        let client = LightconeClient::builder()
            .base_url(&format!("http://{address}"))
            .build()
            .unwrap();
        let orderbooks = client.orderbooks();

        let (first, second) = timeout(Duration::from_secs(2), async {
            tokio::join!(orderbooks.decimals("first"), orderbooks.decimals("second"))
        })
        .await
        .unwrap();

        assert_eq!(first.unwrap().orderbook_id, "first");
        assert_eq!(second.unwrap().orderbook_id, "second");
    }

    #[tokio::test]
    async fn mismatched_rules_are_rejected_and_not_cached() {
        let (base_url, requests) = spawn_rules_server(vec!["wrong", "ob"]).await;
        let client = LightconeClient::builder()
            .base_url(&base_url)
            .build()
            .unwrap();

        let first = timeout(Duration::from_secs(2), client.orderbooks().decimals("ob"))
            .await
            .unwrap();
        assert!(matches!(
            first,
            Err(SdkError::Program(ProgramSdkError::Scaling(
                ScalingError::OrderbookMismatch {
                    ref expected,
                    ref actual,
                }
            ))) if expected == "ob" && actual == "wrong"
        ));

        let second = timeout(Duration::from_secs(2), client.orderbooks().decimals("ob"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second.orderbook_id, "ob");
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Orderbooks<'a> {
    /// Fetch an Orderbook account by mint pair.
    pub async fn get_onchain(
        &self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<crate::program::accounts::Orderbook, SdkError> {
        let rpc = crate::rpc::resolve_solana_rpc(self.client).await?;
        let pda = self.pda(mint_a, mint_b);
        let account = rpc.get_account(&pda).await.map_err(|e| {
            SdkError::Program(crate::program::error::SdkError::AccountNotFound(format!(
                "Orderbook: {}",
                e
            )))
        })?;
        Ok(crate::program::accounts::Orderbook::deserialize(
            &account.data,
        )?)
    }
}
