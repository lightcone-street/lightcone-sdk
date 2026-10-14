// Notification client — fetch + dismiss the authenticated user's notifications.

// Fetch all notifications for the authenticated user.
let fetch: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<array<Notification__Model.t>, SdkError.t>>

// Dismiss a single notification by id.
let dismiss: (Client.t, ~notificationId: string) => promise<result<unit, SdkError.t>>
