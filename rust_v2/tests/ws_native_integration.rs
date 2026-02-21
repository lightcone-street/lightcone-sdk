//! Integration tests for the native WebSocket client.
//!
//! These tests connect to the staging WS server and exercise the full
//! connect → subscribe → receive → unsubscribe → disconnect lifecycle.
//!
//! All tests are `#[ignore]` because they require network access.
//!
//! Run with:
//! ```bash
//! cargo test -p lightcone-sdk-v2 --features ws-native --test ws_native_integration -- --ignored
//! ```

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use lightcone_sdk_v2::shared::OrderBookId;
use lightcone_sdk_v2::ws::native::WsClient;
use lightcone_sdk_v2::ws::{Kind, MessageOut, WsConfig, WsEvent};

const WS_URL: &str = "wss://tws.lightcone.xyz/ws";
const TEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Known orderbook ID on staging (BTC-USD Yes).
const TEST_ORDERBOOK_ID: &str = "2cXSqCoN";

fn test_config() -> WsConfig {
    WsConfig {
        url: WS_URL.into(),
        reconnect: false,
        ..Default::default()
    }
}

/// Connect and wait for the `Connected` event.
async fn connected_client() -> WsClient {
    let mut client = WsClient::new(test_config());
    client.connect().await.expect("connect should succeed");
    wait_for_connected(&client).await;
    client
}

async fn wait_for_connected(client: &WsClient) {
    let events = client.events();
    tokio::pin!(events);

    let first = timeout(TEST_TIMEOUT, events.next())
        .await
        .expect("timed out waiting for Connected")
        .expect("event stream ended");

    assert!(
        matches!(first, WsEvent::Connected),
        "first event should be Connected, got: {first:?}"
    );
}

/// Wait for the next event that matches the predicate, ignoring others.
/// The events stream is created and dropped within this call.
async fn next_matching(client: &WsClient, predicate: impl Fn(&WsEvent) -> bool) -> WsEvent {
    let events = client.events();
    tokio::pin!(events);

    timeout(TEST_TIMEOUT, async {
        while let Some(ev) = events.next().await {
            if predicate(&ev) {
                return ev;
            }
        }
        panic!("event stream ended without a matching event");
    })
    .await
    .expect("timed out waiting for matching event")
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn connect_and_receive_connected_event() {
    let mut client = connected_client().await;
    assert!(client.is_connected());
    client.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn ping_pong() {
    let mut client = connected_client().await;

    client.send(MessageOut::ping()).expect("send ping");

    let pong = next_matching(&client, |ev| matches!(ev, WsEvent::Message(Kind::Pong(_)))).await;

    assert!(matches!(pong, WsEvent::Message(Kind::Pong(_))));

    drop(pong);
    client.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn subscribe_books_receives_snapshot() {
    let mut client = connected_client().await;

    let ob_id = OrderBookId::new(TEST_ORDERBOOK_ID);
    client
        .send(MessageOut::subscribe_books(vec![ob_id]))
        .expect("subscribe books");

    let event = next_matching(&client, |ev| {
        matches!(ev, WsEvent::Message(Kind::BookUpdate(_)))
    })
    .await;

    match event {
        WsEvent::Message(Kind::BookUpdate(book)) => {
            assert!(book.is_snapshot, "first book message should be a snapshot");
        }
        other => panic!("expected BookUpdate, got: {other:?}"),
    }

    client.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn subscribe_ticker_receives_data() {
    let mut client = connected_client().await;

    let ob_id = OrderBookId::new(TEST_ORDERBOOK_ID);
    client
        .send(MessageOut::subscribe_ticker(vec![ob_id]))
        .expect("subscribe ticker");

    let event = next_matching(&client, |ev| {
        matches!(ev, WsEvent::Message(Kind::Ticker(_)))
    })
    .await;

    assert!(matches!(event, WsEvent::Message(Kind::Ticker(_))));

    client.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn subscribe_and_unsubscribe_books() {
    let mut client = connected_client().await;

    let ob_id = OrderBookId::new(TEST_ORDERBOOK_ID);
    client
        .send(MessageOut::subscribe_books(vec![ob_id.clone()]))
        .expect("subscribe books");

    // Wait for the initial snapshot
    next_matching(&client, |ev| {
        matches!(ev, WsEvent::Message(Kind::BookUpdate(_)))
    })
    .await;

    // Unsubscribe
    client
        .send(MessageOut::unsubscribe_books(vec![ob_id]))
        .expect("unsubscribe books");

    // After unsubscribing, send a ping and verify we get a pong back
    // (proving the connection is alive) without more BookUpdate messages.
    client.send(MessageOut::ping()).expect("send ping");

    let event = next_matching(&client, |ev| {
        matches!(
            ev,
            WsEvent::Message(Kind::Pong(_)) | WsEvent::Message(Kind::BookUpdate(_))
        )
    })
    .await;

    assert!(
        matches!(event, WsEvent::Message(Kind::Pong(_))),
        "expected Pong after unsubscribe, got: {event:?}"
    );

    client.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn graceful_disconnect() {
    let mut client = connected_client().await;
    assert!(client.is_connected());

    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert!(!client.is_connected());
}

#[tokio::test]
#[ignore]
async fn multiple_subscriptions() {
    let mut client = connected_client().await;

    let ob_id = OrderBookId::new(TEST_ORDERBOOK_ID);

    client
        .send(MessageOut::subscribe_books(vec![ob_id.clone()]))
        .expect("subscribe books");
    client
        .send(MessageOut::subscribe_ticker(vec![ob_id]))
        .expect("subscribe ticker");

    let mut got_book = false;
    let mut got_ticker = false;

    {
        let events = client.events();
        tokio::pin!(events);

        timeout(TEST_TIMEOUT, async {
            while let Some(ev) = events.next().await {
                match &ev {
                    WsEvent::Message(Kind::BookUpdate(_)) => got_book = true,
                    WsEvent::Message(Kind::Ticker(_)) => got_ticker = true,
                    _ => {}
                }
                if got_book && got_ticker {
                    break;
                }
            }
        })
        .await
        .expect("timed out waiting for both book and ticker messages");
    }

    assert!(got_book, "should have received a BookUpdate");
    assert!(got_ticker, "should have received a Ticker");

    client.disconnect().await.unwrap();
}
