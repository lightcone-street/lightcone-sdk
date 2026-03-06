#![allow(dead_code)]

use std::{
    env,
    error::Error,
    io,
    time::{SystemTime, UNIX_EPOCH},
};

use dotenvy::dotenv;
use lightcone::shared::OrderbookDecimals;
use lightcone::{auth::native::sign_login_message, prelude::*, program::LightconePinocchioClient};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::Pubkey;

pub type ExampleResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn other(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.into())
}

pub fn load_env() {
    let _ = dotenv();
}

pub fn optional_var(name: &str) -> Option<String> {
    load_env();
    env::var(name).ok().filter(|value| !value.is_empty())
}

pub fn required_var(name: &str) -> ExampleResult<String> {
    optional_var(name).ok_or_else(|| other(format!("set {name} in .env or the environment")).into())
}

pub fn env_flag(name: &str) -> bool {
    matches!(
        optional_var(name).as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

pub fn write_enabled() -> bool {
    env_flag("LIGHTCONE_EXECUTE_WRITES")
}

pub fn token_2022() -> bool {
    env_flag("LIGHTCONE_TOKEN_2022")
}

pub fn unix_timestamp() -> ExampleResult<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    )?)
}

pub async fn fresh_order_nonce(
    rpc: &LightconePinocchioClient,
    user: &Pubkey,
) -> ExampleResult<u64> {
    Ok(u64::from(rpc.get_current_nonce(user).await?).max(unix_timestamp()? as u64))
}

pub fn rest_client() -> ExampleResult<LightconeClient> {
    let mut builder = LightconeClient::builder();
    if let Some(url) = optional_var("LIGHTCONE_API_URL") {
        builder = builder.base_url(&url);
    }
    if let Some(url) = optional_var("LIGHTCONE_WS_URL") {
        builder = builder.ws_url(&url);
    }
    Ok(builder.build()?)
}

pub fn rpc_client() -> LightconePinocchioClient {
    LightconePinocchioClient::new(
        &optional_var("SOLANA_RPC_URL")
            .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string()),
    )
}

pub fn wallet() -> ExampleResult<Keypair> {
    Ok(read_keypair_file(required_var("LIGHTCONE_WALLET_PATH")?)?)
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
    if let Some(slug) = optional_var("LIGHTCONE_MARKET_SLUG") {
        return Ok(client.markets().get_by_slug(&slug).await?);
    }

    if let Some(slug) = client
        .markets()
        .featured()
        .await?
        .into_iter()
        .next()
        .map(|market| market.slug)
    {
        return Ok(client.markets().get_by_slug(&slug).await?);
    }

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
