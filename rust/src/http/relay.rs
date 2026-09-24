//! Request-scoped relay for servers that call the API on behalf of one visitor.
//!
//! A server such as the Lightcone web app holds one API key for the whole
//! process but forwards each visitor's own cookies and country. The shared
//! [`crate::LightconeClient`] must not absorb any of that per-visitor state, so
//! relayed calls carry a [`RelayContext`] and return a [`Relayed`] envelope
//! with the backend's raw `Set-Cookie` values for the server to pass on.
//!
//! ```text
//! browser -> server function -> RelayContext { cookies, country, relay secret }
//!         -> SDK relayed call (API key + relay headers, no session capture)
//!         -> Relayed { body, set_cookie } -> server relays Set-Cookie to the browser
//! ```

/// Per-request relay inputs. Every field is optional so public routes can be
/// relayed without cookies and local development can omit the country relay.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RelayContext {
    /// The visitor's raw `Cookie` header, sent verbatim to the API origin.
    pub cookie_header: Option<String>,
    /// The visitor's normalized country (`US`, `T1`, `XX`, ...), sent as
    /// `x-lightcone-visitor-country` together with the relay secret.
    pub visitor_country: Option<String>,
    /// The relay secret Cloudflare requires before it trusts `visitor_country`.
    /// Sent as `x-lightcone-relay-secret`. Never logged by the SDK.
    pub relay_secret: Option<String>,
}

impl std::fmt::Debug for RelayContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelayContext")
            .field(
                "cookie_header",
                &self.cookie_header.as_ref().map(|_| "[REDACTED]"),
            )
            .field("visitor_country", &self.visitor_country)
            .field(
                "relay_secret",
                &self.relay_secret.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

impl RelayContext {
    /// A context that forwards only the visitor's cookies.
    pub fn with_cookies(cookie_header: impl Into<String>) -> Self {
        Self {
            cookie_header: Some(cookie_header.into()),
            ..Self::default()
        }
    }

    /// Adds the visitor country and the relay secret that vouches for it.
    pub fn with_visitor_country(
        mut self,
        country: impl Into<String>,
        relay_secret: impl Into<String>,
    ) -> Self {
        self.visitor_country = Some(country.into());
        self.relay_secret = Some(relay_secret.into());
        self
    }
}

/// A relayed response: the parsed body plus every raw `Set-Cookie` header the
/// API returned, in order, so a server can relay them to the browser unchanged.
#[derive(Clone, PartialEq, Eq)]
pub struct Relayed<T> {
    pub body: T,
    pub set_cookie: Vec<String>,
}

impl<T> std::fmt::Debug for Relayed<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Relayed")
            .field("body", &"[REDACTED]")
            .field("set_cookie", &"[REDACTED]")
            .finish()
    }
}

/// Request header that carries the relayed visitor country.
pub const VISITOR_COUNTRY_HEADER: &str = "x-lightcone-visitor-country";
/// Request header that carries the relay secret.
pub const RELAY_SECRET_HEADER: &str = "x-lightcone-relay-secret";
/// Request header that carries an API key.
pub const API_KEY_HEADER: &str = "x-lightcone-api-key";

#[cfg(test)]
mod tests {
    use super::{RelayContext, Relayed};

    #[test]
    fn debug_redacts_visitor_credentials() {
        let context = RelayContext::with_cookies("private-cookie")
            .with_visitor_country("US", "private-relay-secret");
        let printed = format!("{context:?}");
        assert!(printed.contains("US"));
        assert!(!printed.contains("private-cookie"));
        assert!(!printed.contains("private-relay-secret"));
    }

    #[test]
    fn relayed_debug_redacts_response_and_session_cookies() {
        let response = Relayed {
            body: "private-response",
            set_cookie: vec!["private-session".to_string()],
        };
        let printed = format!("{response:?}");
        assert!(!printed.contains("private-response"));
        assert!(!printed.contains("private-session"));
    }
}
