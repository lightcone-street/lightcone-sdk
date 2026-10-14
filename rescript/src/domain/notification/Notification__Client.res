// Notification client — fetch + dismiss the authenticated user's notifications.

// Fetch all notifications for the authenticated user.
let fetch = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<array<Notification__Model.t>, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/notifications",
    ~cookieHeader?,
    ~decode=Notification__Raw.decodeResponse,
  )

// Dismiss a single notification by id.
let dismiss = async (client: Client.t, ~notificationId: string): result<unit, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("notification_id", JSON.String(notificationId))]))
  await Http.post(client.http, ~path="/api/notifications/dismiss", ~body, ~decode=_ => Ok())
}
