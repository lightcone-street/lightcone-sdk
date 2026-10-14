# Native client session recovery

This guide covers native clients using `lightcone-token` through the Rust native, Python, and TypeScript/Node SDKs. This includes terminal and server integrations. Browser clients use Privy. Native clients must not synthesize an Origin header for self-custody nonce or login requests.

The published [WebSocket authentication reference](https://github.com/lightcone-street/docs/blob/main/websocket/authentication.mdx) belongs to `lightcone-street/docs`. Backend session guarantees and operational timing belong to the [native session contract](https://github.com/lightcone-street/lightcone-backend/blob/staging/docs/lightcone-session-auth.md). The procedures below describe how applications using these SDK versions react to that contract.

Each native client keeps its selected credential and explicitly sends `Cookie: lightcone-token=<value>`. A host-only, Strict login cookie is a delivery envelope. It does not require the API and WebSocket to share a cookie domain. Use the matching environment's endpoints. Switching endpoints does not make an existing token valid in another environment.

## Logout

Logout presents the current token before clearing local state. Local token and credential cleanup happens even if the server call fails. A successful server result confirms presented-token revocation and completion of its private WebSocket output fence. Other independently issued tokens remain active.

| Error code | Meaning and caller action |
| --- | --- |
| `AUTH_SESSION_UNAVAILABLE` | Token verification or session setup failed before revocation started. Report incomplete teardown. The credential may remain valid. |
| `TOKEN_REVOCATION_UNAVAILABLE` | Revocation could not be confirmed. The token may still authorize requests. Report incomplete teardown. |
| `TOKEN_REVOCATION_RECOVERY_UNAVAILABLE` | The token is inactive, but recovery coordination could not be recorded. Discard it and report incomplete teardown. |
| `TOKEN_REVOCATION_FENCE_PENDING` | The token is revoked, but remote socket fencing is not confirmed. Discard it and report incomplete teardown. |
| `AMBIGUOUS_LIGHTCONE_TOKEN` | The request carried multiple same-name credentials. Remove stale cookie-jar entries and select one credential explicitly. The server did not revoke either token. |

All these errors remain visible to the native client. HTTP 401 retains the SDK's existing already-logged-out treatment. The SDK does not retain a retry copy or automatically repeat logout. A native client that explicitly retained the presented token may retry to join the server's revocation transaction. Do not install that token as an active trading credential again.

The server waits at most 10 seconds for logout. A missing session record requires 19 seconds of recovery coordination. Its first logout therefore returns `TOKEN_REVOCATION_FENCE_PENDING`. This is expected after eviction. A retained credential can join completion after that recovery window. Store availability and remaining writer leases can still prevent success.

## WebSocket downgrade

An `auth` frame with `status: anonymous` changes authority, not the connection's transport state. Public subscriptions can continue. Private subscriptions are gone and cannot be restored on that same connection.

- `TOKEN_EXPIRED` or `TOKEN_REVOKED`: discard the old credential and obtain a new session before restoring private access.
- `INVALID_TOKEN`: the handshake could not establish valid active authority, including when the session record is missing. Discard the credential and sign in again.
- No reason: the server did not prove the credential invalid. A session-store or lease check may have failed. After availability returns, check the credential and reconnect. A missing server-side record requires signing in again.
- `PRIVATE_SNAPSHOT_UNAVAILABLE`: the private subscription was not committed. Use `wallet_address` to correlate the failure and reissue the subscription while authenticated.

The SDK emits these frames without automatically reconnecting or deleting public replay state. The native client owns recovery. Reissue the subscriptions it requires after creating a replacement connection. Replacing an anonymous connection can temporarily interrupt public data if the new handshake fails.

Unavailable Lightcone authentication during a native client's handshake returns HTTP 503 and uses the existing reconnect/backoff behavior. That behavior has a finite attempt budget. Handle exhaustion rather than assuming the native client retries forever:

| Language | Budget setting | Exhaustion event | Explicit recovery entry point |
| --- | --- | --- | --- |
| Rust native | `WsConfig::max_reconnect_attempts` | `WsEvent::MaxReconnectReached` | Drop the borrowed event stream, then call `disconnect().await?` followed by `connect().await?`. |
| Python | `WsConfig.max_reconnect_attempts` | `WsEventType.MAX_RECONNECT_REACHED` (`max_reconnect_reached`) | `restart_connection()` |
| TypeScript/Node | `WsConfig.maxReconnectAttempts` | `{ type: "MaxReconnectReached" }` | After retries have stopped, remove the old event listener and create a replacement `WsClient` with the same configuration and token callback. Attach listeners and call `connect()`. |

The colocated auth-session tests use small retry budgets and recover after a handshake failure. Follow the language-specific procedure above after the endpoint becomes available, then reissue required subscriptions. Rust's restart helper can ignore an exhausted native client that remains in `Connecting`. Node's restart and disconnect helpers can wait for another close event on an already-closed socket. Native clients must choose when to retry based on service availability. The retry count does not define a fixed recovery duration.

HTTP credential restoration is separate. A registered restorer can create a new session after an ordinary HTTP 401. It does not revive a revoked token or automatically restore an anonymous WebSocket's private subscriptions.
