// Server-only entry: importing this module loads Node's ws transport.
type socketOptions = {headers: dict<string>, followRedirects: bool}
@module("ws") @new external socket: (string, socketOptions) => WebSocketClient.t = "WebSocket"
@genType let useNativeSigner = Client.useNativeSigner
@genType
let login = (client, ~useEmbeddedWallet=?) =>
  SdkError.unwrap(Auth__Client.login(client, ~useEmbeddedWallet?))
@genType let authToken = Client.authToken
@genType let setAuthToken = (client: Client.t, token) => Http.setAuthToken(client.http, token)
@genType
let setServerApiKey = (client: Client.t, key) =>
  SdkError.unwrap(Promise.resolve(Http.setServerApiKey(client.http, key)))
@genType
let connect = (
  client: Client.t,
  ~onMessage,
  ~onConnected=() => (),
  ~onError=_ => (),
): TypeScriptApi.WsClient.wsConnection => {
  if Http.isBrowser() {
    SdkError.throwAsJsError(
      "server WebSockets are unavailable in browsers",
      SdkError.Validation("server-only WebSocket transport"),
    )
  }
  Ws.connect(
    ~url=client.wsUrl,
    ~onMessage,
    ~onConnected,
    ~onError,
    ~socketFactory=url => {
      // Read the current token for every handshake, including reconnects.
      let headers = Dict.make()
      Client.authToken(client)->Option.forEach(token =>
        Dict.set(headers, "Cookie", `lightcone-token=${token}`)
      )
      socket(url, {headers, followRedirects: false})
    },
    (),
  )
}
