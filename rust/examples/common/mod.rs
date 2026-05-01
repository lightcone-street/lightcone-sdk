#![allow(dead_code)]

use std::{
    env,
    error::Error,
    io,
    time::{SystemTime, UNIX_EPOCH},
};

use lightcone::{auth::native::sign_login_message, prelude::*};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::Pubkey;

pub type ExampleResult<T = ()> = Result<T, Box<dyn Error>>;

const DEFAULT_WALLET_PATH: &str = "~/.config/solana/id.json";

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

pub async fn fresh_order_nonce(client: &LightconeClient, user: &Pubkey) -> ExampleResult<u64> {
    Ok(client.orders().get_nonce(user).await?)
}

pub fn rest_client() -> ExampleResult<LightconeClient> {
    let mut builder = LightconeClient::builder();
    if let Ok(env_str) = env::var("LIGHTCONE_ENV") {
        let environment = match env_str.to_lowercase().as_str() {
            "local" => LightconeEnv::Local,
            "staging" => LightconeEnv::Staging,
            "prod" => LightconeEnv::Prod,
            other => {
                return Err(format!(
                    "invalid LIGHTCONE_ENV '{other}'. Options: local, staging, prod"
                )
                .into())
            }
        };
        builder = builder.env(environment);
    }
    Ok(builder.build()?)
}

pub fn get_keypair() -> ExampleResult<Keypair> {
    let raw = env::var("LIGHTCONE_WALLET_PATH").unwrap_or_else(|_| DEFAULT_WALLET_PATH.to_string());
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

pub async fn wait_for_global_balance(
    client: &LightconeClient,
    mint: &Pubkey,
    minimum: rust_decimal::Decimal,
) -> ExampleResult {
    use std::time::{Duration, Instant};

    let mint_str = mint.to_string();
    let deadline = Instant::now() + Duration::from_secs(30);
    let interval = Duration::from_secs(2);
    let mut attempt = 0u32;

    println!("waiting for global balance: mint={mint_str} required={minimum}");

    loop {
        attempt += 1;
        let balances = client.positions().deposit_token_balances().await?;
        let entry = balances
            .values()
            .find(|balance| balance.mint.as_str() == mint_str);
        let current_idle = entry.map(|e| e.idle).unwrap_or_default();
        let symbol = entry.map(|e| e.symbol.as_str()).unwrap_or("unknown");

        if current_idle >= minimum {
            println!("global balance ready: {symbol} idle={current_idle} (attempt {attempt})");
            return Ok(());
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        println!(
            "global balance not ready: {symbol} idle={current_idle}/{minimum} \
             (attempt {attempt}, {}s remaining)",
            remaining.as_secs()
        );

        if Instant::now() >= deadline {
            return Err(format!(
                "global balance for {mint_str} did not reach {minimum} within 30s"
            )
            .into());
        }
        tokio::time::sleep(interval).await;
    }
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

pub fn quote_deposit_mint(orderbook: &OrderBookPair) -> ExampleResult<Pubkey> {
    parse_pubkey(&orderbook.quote.deposit_asset)
}

pub fn num_outcomes(market: &Market) -> ExampleResult<u8> {
    u8::try_from(market.outcomes.len()).map_err(|_| other("market outcome count exceeds u8").into())
}
