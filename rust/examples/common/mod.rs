#![allow(dead_code)]

use std::{
    env,
    error::Error,
    io,
    time::{SystemTime, UNIX_EPOCH},
};

use dotenvy::dotenv;
use lightcone::shared::OrderbookDecimals;
use lightcone::{auth::native::sign_login_message, prelude::*};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::Pubkey;

pub type ExampleResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn other(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.into())
}

pub fn unix_timestamp() -> ExampleResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    )?)
}

pub fn unix_timestamp_ms() -> ExampleResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
    )?)
}

pub async fn fresh_order_nonce(
    client: &LightconeClient,
    user: &Pubkey,
) -> ExampleResult<u64> {
    Ok(client.orders().get_nonce(user).await?)
}

pub fn rest_client() -> ExampleResult<LightconeClient> {
    Ok(LightconeClient::builder()
        .rpc_url("https://api.devnet.solana.com")
        .build()?)
}

pub fn wallet() -> ExampleResult<Keypair> {
    let _ = dotenv();
    let raw = env::var("LIGHTCONE_WALLET_PATH")
        .map_err(|_| other("set LIGHTCONE_WALLET_PATH in .env or the environment"))?;
    let path = if let Some(rest) = raw.strip_prefix("~/") {
        let home = env::var("HOME").map_err(|_| other("HOME not set"))?;
        std::path::PathBuf::from(home).join(rest)
    } else {
        raw.into()
    };
    Ok(read_keypair_file(path)?)
}

pub async fn login(
    client: &LightconeClient,
    keypair: &Keypair,
    use_embedded_wallet: bool,
) -> ExampleResult<User> {
    let nonce = client.auth().get_nonce().await?;
    let signed = sign_login_message(keypair, &nonce);
    Ok(client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            use_embedded_wallet.then_some(true),
        )
        .await?)
}

pub async fn market(client: &LightconeClient) -> ExampleResult<Market> {
    client
        .markets()
        .get(None, Some(1))
        .await?
        .markets
        .into_iter()
        .next()
        .ok_or_else(|| other("no markets returned by the API").into())
}

pub async fn market_and_orderbook(
    client: &LightconeClient,
) -> ExampleResult<(Market, OrderBookPair)> {
    let market = market(client).await?;
    let orderbook = market
        .orderbook_pairs
        .iter()
        .find(|pair| pair.active)
        .or_else(|| market.orderbook_pairs.first())
        .cloned()
        .ok_or_else(|| other("selected market has no orderbooks"))?;
    Ok((market, orderbook))
}

pub fn parse_pubkey(value: &PubkeyStr) -> ExampleResult<Pubkey> {
    value.to_pubkey().map_err(|err| other(err).into())
}

pub fn orderbook_mints(orderbook: &OrderBookPair) -> ExampleResult<(Pubkey, Pubkey)> {
    Ok((
        parse_pubkey(orderbook.base.pubkey())?,
        parse_pubkey(orderbook.quote.pubkey())?,
    ))
}

pub async fn scaling_decimals(
    client: &LightconeClient,
    orderbook: &OrderBookPair,
) -> ExampleResult<OrderbookDecimals> {
    let decimals = client
        .orderbooks()
        .decimals(orderbook.orderbook_id.as_str())
        .await?;

    Ok(OrderbookDecimals {
        orderbook_id: decimals.orderbook_id,
        base_decimals: decimals.base_decimals,
        quote_decimals: decimals.quote_decimals,
        price_decimals: decimals.price_decimals,
        tick_size: orderbook.tick_size.max(0) as u64,
    })
}

pub fn deposit_mint(market: &Market) -> ExampleResult<Pubkey> {
    parse_pubkey(
        market
            .deposit_assets
            .first()
            .ok_or_else(|| other("selected market has no deposit assets"))?
            .pubkey(),
    )
}

pub fn num_outcomes(market: &Market) -> ExampleResult<u8> {
    u8::try_from(market.outcomes.len()).map_err(|_| other("market outcome count exceeds u8").into())
}
