//! Authentication re-exports for backward compatibility.
//!
//! The canonical auth module is now at [`crate::auth`]. This module
//! re-exports its types so existing `use crate::websocket::auth::*` paths
//! continue to work.

pub use crate::auth::{
    AuthCredentials, AuthError, AuthResult, AUTH_API_URL, generate_signin_message,
    generate_signin_message_with_timestamp,
};

#[cfg(feature = "auth")]
pub use crate::auth::authenticate;
