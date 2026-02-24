//! Submit orders with TIF, submit trigger orders (TP/SL), and cancel trigger orders.
//!
//! Requires a valid Solana keypair, authentication, and env vars:
//!   MARKET_PUBKEY  — market to trade on (must have deposited into it)
//!
//! ```bash
//! cargo run --example trigger_orders --features native
//! ```

use lightcone_sdk_v2::auth::native::sign_login_message;
use lightcone_sdk_v2::prelude::*;
use lightcone_sdk_v2::program::builder::OrderBuilder;
use lightcone_sdk_v2::program::client::LightconePinocchioClient;
use lightcone_sdk_v2::shared::OrderbookDecimals;
use solana_signer::Signer;
use std::time::{SystemTime, UNIX_EPOCH};

fn load_keypair() -> solana_keypair::Keypair {
    dotenvy::dotenv().ok();
    let path = std::env::var("KEYPAIR_PATH").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/.config/solana/id.json", home)
    });
    let bytes: Vec<u8> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("keypair file not found"))
            .expect("invalid keypair JSON");
    solana_keypair::Keypair::try_from(bytes.as_slice()).expect("invalid keypair bytes")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = load_keypair();
    let wallet = keypair.pubkey().to_string();
    let client = LightconeClient::builder().build()?;

    // ── 1. Authenticate ──────────────────────────────────────────────────
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let signed = sign_login_message(&keypair, timestamp);
    client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;
    println!("Authenticated as: {}\n", keypair.pubkey());

    // ── 2. Fetch on-chain nonce ─────────────────────────────────────────
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let onchain = LightconePinocchioClient::new(&rpc_url);
    let nonce = onchain.get_current_nonce(&keypair.pubkey()).await?;
    println!("On-chain nonce: {}\n", nonce);

    // ── 3. Discover market and orderbook ────────────────────────────────
    let market_pubkey = std::env::var("MARKET_PUBKEY").expect("Set MARKET_PUBKEY in .env");
    let market = client.markets().get_by_pubkey(&market_pubkey).await?;
    let pair = market
        .orderbook_pairs
        .first()
        .expect("market has no orderbooks");
    let orderbook_id = pair.orderbook_id.as_str();
    let base_mint = pair.base.pubkey().to_pubkey().expect("invalid base mint");
    let quote_mint = pair.quote.pubkey().to_pubkey().expect("invalid quote mint");
    let market_pk = market.pubkey.to_pubkey().expect("invalid market pubkey");

    println!("Market:    {} ({})", market.name, market.pubkey);
    println!("Orderbook: {}", orderbook_id);

    // ── 4. Fetch decimals for scaling ────────────────────────────────────
    let dec = client.orderbooks().decimals(orderbook_id).await?;
    let decimals = OrderbookDecimals {
        orderbook_id: orderbook_id.to_string(),
        base_decimals: dec.base_decimals,
        quote_decimals: dec.quote_decimals,
        price_decimals: dec.price_decimals,
        tick_size: 0,
    };
    println!(
        "Decimals:  base={}, quote={}, price={}\n",
        dec.base_decimals, dec.quote_decimals, dec.price_decimals
    );

    // =====================================================================
    // TIF Orders — test all 4 time-in-force variants
    // =====================================================================

    // ── 5a. GTC (good-til-cancelled) — default, stays on book ───────────
    println!("── GTC order ──");
    let gtc_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.40")
        .size("2")
        .gtc()
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: BID  price: 0.40  size: 2  tif: GTC");
    match client.orders().submit(&gtc_request).await {
        Ok(resp) => {
            println!("  order_hash: {}", resp.order_hash);
            println!("  filled: {}  remaining: {}", resp.filled, resp.remaining);
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // ── 5b. IOC (immediate-or-cancel) — unfilled remainder cancelled ────
    println!("\n── IOC order ──");
    let ioc_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.45")
        .size("5")
        .ioc()
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: BID  price: 0.45  size: 5  tif: IOC");
    match client.orders().submit(&ioc_request).await {
        Ok(resp) => {
            println!("  order_hash: {}", resp.order_hash);
            println!("  filled: {}  remaining: {}", resp.filled, resp.remaining);
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // ── 5c. FOK (fill-or-kill) — must fill entirely or reject ───────────
    println!("\n── FOK order ──");
    let fok_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.42")
        .size("3")
        .fok()
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: BID  price: 0.42  size: 3  tif: FOK");
    match client.orders().submit(&fok_request).await {
        Ok(resp) => {
            println!("  order_hash: {}", resp.order_hash);
            println!("  filled: {}  remaining: {}", resp.filled, resp.remaining);
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // ── 5d. ALO (add-liquidity-only / post-only) — rejected if crosses ──
    println!("\n── ALO order ──");
    let alo_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.35")
        .size("4")
        .alo()
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: BID  price: 0.35  size: 4  tif: ALO");
    match client.orders().submit(&alo_request).await {
        Ok(resp) => {
            println!("  order_hash: {}", resp.order_hash);
            println!("  filled: {}  remaining: {}", resp.filled, resp.remaining);
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // =====================================================================
    // Trigger Orders — test TP and SL, plus cancel
    // =====================================================================

    // ── 6. Take-profit trigger order (submit + cancel) ───────────────────
    println!("\n── Take-profit trigger order ──");
    let tp_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .ask()
        .price("0.80")
        .size("5")
        .take_profit(0.75)
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: ASK  price: 0.80  size: 5  trigger: TP @ 0.75");
    match client.orders().submit_trigger(&tp_request).await {
        Ok(resp) => {
            println!("  trigger_order_id: {}", resp.trigger_order_id);
            println!("  order_hash: {}", resp.order_hash);
            println!("  status: {}", resp.status);

            // Cancel the trigger order we just created
            println!("\n── Cancel trigger order ──");
            let cancel_body = CancelTriggerBody::signed(
                resp.trigger_order_id.clone(),
                wallet.clone(),
                &keypair,
            );
            match client.orders().cancel_trigger(&cancel_body).await {
                Ok(c) => println!("  Cancelled: {}", c.trigger_order_id),
                Err(e) => println!("  Cancel failed: {}", e),
            }
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // ── 7. Stop-loss trigger order ───────────────────────────────────────
    println!("\n── Stop-loss trigger order ──");
    let sl_request = OrderBuilder::new()
        .nonce(nonce)
        .maker(keypair.pubkey())
        .market(market_pk)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .ask()
        .price("0.25")
        .size("5")
        .stop_loss(0.30)
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;
    println!("  side: ASK  price: 0.25  size: 5  trigger: SL @ 0.30");
    match client.orders().submit_trigger(&sl_request).await {
        Ok(resp) => {
            println!("  trigger_order_id: {}", resp.trigger_order_id);
            println!("  order_hash: {}", resp.order_hash);
            println!("  status: {}", resp.status);
        }
        Err(e) => println!("  Failed: {}", e),
    }

    // =====================================================================
    // Verify — fetch all orders from backend
    // =====================================================================

    // ── 8. Fetch user orders to verify trigger orders from backend ───────
    println!("\n── Verify via get_user_orders ──");
    let user_orders = client
        .orders()
        .get_user_orders(&wallet, Some(100), None)
        .await?;

    println!("  open orders:    {}", user_orders.orders.len());
    println!("  trigger orders: {}", user_orders.trigger_orders.len());
    for t in &user_orders.trigger_orders {
        println!(
            "    id={}  type={}  price={}  side={}  maker_amount={}  taker_amount={}",
            t.trigger_order_id, t.trigger_type, t.trigger_price, t.side,
            t.maker_amount, t.taker_amount
        );
    }

    Ok(())
}
