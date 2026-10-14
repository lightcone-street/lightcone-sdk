// Notification wire decoding (adjacently-tagged kind, hand-decoded). Public
// surface: the notification decoder (consumed by `ws/Messages` and the Order
// snapshot decoder) and the list-response decoder; the payload decoders are
// internal.

// Decode one notification.
let decode: JSON.t => result<Notification__Model.t, Spice.decodeError>

// Decode the list response `{ notifications: [...] }`.
let decodeResponse: JSON.t => result<array<Notification__Model.t>, Spice.decodeError>
