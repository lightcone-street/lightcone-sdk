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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relayed<T> {
    pub body: T,
    pub set_cookie: Vec<String>,
}

/// Request header that carries the relayed visitor country.
pub const VISITOR_COUNTRY_HEADER: &str = "x-lightcone-visitor-country";
/// Request header that carries the relay secret.
pub const RELAY_SECRET_HEADER: &str = "x-lightcone-relay-secret";
/// Request header that carries an API key.
pub const API_KEY_HEADER: &str = "x-lightcone-api-key";
