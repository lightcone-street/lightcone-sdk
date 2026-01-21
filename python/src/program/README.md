# Program Module Reference

On-chain interaction with the Lightcone Solana program.

## Client

### LightconePinocchioClient

```python
from solana.rpc.async_api import AsyncClient
from lightcone_sdk.program import LightconePinocchioClient
from lightcone_sdk.shared import PROGRAM_ID

# Create client
connection = AsyncClient("https://api.mainnet-beta.solana.com")
client = LightconePinocchioClient(connection)

# With custom program ID (e.g., devnet)
client = LightconePinocchioClient(connection, program_id=custom_program_id)
```

## Account Types

### Exchange

Global exchange state account.

```python
exchange = await client.get_exchange()

# Fields
exchange.authority     # Pubkey - Admin authority
exchange.operator      # Pubkey - Order matching operator
exchange.market_count  # int - Number of markets created
exchange.paused        # bool - Trading paused
exchange.bump          # int - PDA bump seed
```

### Market

Individual market account.

```python
# By market ID
market = await client.get_market(market_id)

# By address
market = await client.get_market_by_address(market_pubkey)

# Fields
market.market_id       # int - Sequential ID
market.num_outcomes    # int - Number of outcomes
market.status          # MarketStatus - PENDING/ACTIVE/RESOLVED/CANCELLED
market.winning_outcome # Optional[int] - Winner if settled
market.bump            # int - PDA bump
market.oracle          # Pubkey - Oracle authority
market.question_id     # bytes - Question identifier
market.condition_id    # bytes - Computed condition ID
```

### Position

User position in a market (optional account).

```python
position = await client.get_position(owner_pubkey, market_pubkey)
if position:
    position.owner   # Pubkey - Position owner
    position.market  # Pubkey - Market address
    position.bump    # int - PDA bump
```

### OrderStatus

Tracks filled/cancelled state of orders.

```python
order_status = await client.get_order_status(order_hash_bytes)
if order_status:
    order_status.remaining    # int - Remaining amount
    order_status.is_cancelled # bool - Cancelled flag
```

### UserNonce

User's nonce for order signing.

```python
nonce = await client.get_user_nonce(user_pubkey)
next_nonce = await client.get_next_nonce(user_pubkey)
```

## Transaction Builders

All transaction builders return a `Transaction` ready for signing.

### Exchange Administration

```python
# Initialize exchange (one-time)
tx = await client.initialize(authority_pubkey)

# Pause/unpause trading
tx = await client.set_paused(authority_pubkey, paused=True)

# Change operator
tx = await client.set_operator(authority_pubkey, new_operator_pubkey)
```

### Market Lifecycle

```python
from lightcone_sdk.shared import ActivateMarketParams

# Create market
tx = await client.create_market(
    authority=authority_pubkey,
    num_outcomes=2,
    oracle=oracle_pubkey,
    question_id=question_id_bytes,  # 32 bytes
)

# Add deposit mint (with outcome metadata)
from lightcone_sdk.shared import AddDepositMintParams, OutcomeMetadata

tx = await client.add_deposit_mint(
    AddDepositMintParams(
        payer=payer_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        outcome_metadata=[
            OutcomeMetadata(name="Yes", symbol="YES", uri=""),
            OutcomeMetadata(name="No", symbol="NO", uri=""),
        ],
    ),
    num_outcomes=2,
)

# Activate market
tx = await client.activate_market(ActivateMarketParams(
    authority=authority_pubkey,
    market=market_pubkey,
))

# Settle market
from lightcone_sdk.shared import SettleMarketParams

tx = await client.settle_market(SettleMarketParams(
    oracle=oracle_pubkey,
    market=market_pubkey,
    winning_outcome=0,
))
```

### Position Management

```python
from lightcone_sdk.shared import (
    MintCompleteSetParams,
    MergeCompleteSetParams,
    RedeemWinningsParams,
    WithdrawFromPositionParams,
)

# Mint complete set (deposit collateral, receive outcome tokens)
tx = await client.mint_complete_set(
    MintCompleteSetParams(
        user=user_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        amount=1_000_000,  # 1 USDC (6 decimals)
    ),
    num_outcomes=2,
)

# Merge complete set (burn outcome tokens, receive collateral)
tx = await client.merge_complete_set(
    MergeCompleteSetParams(
        user=user_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        amount=1_000_000,
    ),
    num_outcomes=2,
)

# Redeem winnings (after settlement)
tx = await client.redeem_winnings(
    RedeemWinningsParams(
        user=user_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        amount=1_000_000,
    ),
    winning_outcome=0,
)

# Withdraw tokens from position
tx = await client.withdraw_from_position(
    WithdrawFromPositionParams(
        user=user_pubkey,
        position=position_pubkey,
        mint=token_mint,
        amount=1_000_000,
    ),
    is_token_2022=True,  # True for conditional tokens
)
```

### Order Management

```python
# Cancel order
tx = await client.cancel_order(maker_pubkey, order)

# Increment nonce (invalidates all orders with current nonce)
tx = await client.increment_nonce(user_pubkey)
```

### Order Matching

Three strategies with different transaction size/verification tradeoffs:

```python
from lightcone_sdk.shared import MakerFill

maker_fills = [
    MakerFill(order=maker_order_1, fill_amount=100_000),
    MakerFill(order=maker_order_2, fill_amount=200_000),
]

# 1. Without Ed25519 verification (signatures verified off-chain)
tx = await client.match_orders_multi(
    operator=operator_pubkey,
    market=market_pubkey,
    base_mint=base_mint,
    quote_mint=quote_mint,
    taker_order=taker_order,
    maker_fills=maker_fills,
)

# 2. With batch Ed25519 verification (signatures in instruction data)
tx = await client.match_orders_multi_with_verify(
    operator=operator_pubkey,
    market=market_pubkey,
    base_mint=base_mint,
    quote_mint=quote_mint,
    taker_order=taker_order,
    maker_fills=maker_fills,
)

# 3. With cross-reference Ed25519 (smallest transaction size)
tx = await client.match_orders_multi_cross_ref(
    operator=operator_pubkey,
    market=market_pubkey,
    base_mint=base_mint,
    quote_mint=quote_mint,
    taker_order=taker_order,
    maker_fills=maker_fills,
)
```

## PDA Derivation

```python
from lightcone_sdk.program import (
    get_exchange_pda,
    get_market_pda,
    get_vault_pda,
    get_mint_authority_pda,
    get_conditional_mint_pda,
    get_order_status_pda,
    get_user_nonce_pda,
    get_position_pda,
    get_all_conditional_mints,
)
from lightcone_sdk.shared import PROGRAM_ID

# Exchange PDA
exchange_pda, bump = get_exchange_pda(PROGRAM_ID)

# Market PDA
market_pda, bump = get_market_pda(market_id, PROGRAM_ID)

# Vault PDA (token account)
vault_pda, bump = get_vault_pda(market_pubkey, deposit_mint, PROGRAM_ID)

# Mint authority PDA
mint_auth_pda, bump = get_mint_authority_pda(market_pubkey, deposit_mint, PROGRAM_ID)

# Conditional token mint PDA
cond_mint_pda, bump = get_conditional_mint_pda(
    market_pubkey, deposit_mint, outcome_index, PROGRAM_ID
)

# Order status PDA
order_status_pda, bump = get_order_status_pda(order_hash, PROGRAM_ID)

# User nonce PDA
user_nonce_pda, bump = get_user_nonce_pda(user_pubkey, PROGRAM_ID)

# Position PDA
position_pda, bump = get_position_pda(owner_pubkey, market_pubkey, PROGRAM_ID)

# All conditional mints for a market
mints = get_all_conditional_mints(market_pubkey, deposit_mint, num_outcomes, PROGRAM_ID)
```

### Client Helper Methods

```python
# Convenience methods on client
exchange_addr = client.get_exchange_address()
market_addr = client.get_market_address(market_id)
position_addr = client.get_position_address(owner, market)
order_status_addr = client.get_order_status_address(order_hash)
user_nonce_addr = client.get_user_nonce_address(user)
conditional_mints = client.get_conditional_mints(market, deposit_mint, num_outcomes)
```

## Order Types

### FullOrder (225 bytes)

Complete order with signature for matching.

| Offset | Field | Size | Description |
|--------|-------|------|-------------|
| 0 | nonce | 8 | Order nonce (u64) |
| 8 | maker | 32 | Maker pubkey |
| 40 | market | 32 | Market pubkey |
| 72 | base_mint | 32 | Base token mint |
| 104 | quote_mint | 32 | Quote token mint |
| 136 | side | 1 | 0=BID, 1=ASK |
| 137 | maker_amount | 8 | Amount maker gives (u64) |
| 145 | taker_amount | 8 | Amount maker receives (u64) |
| 153 | expiration | 8 | Unix timestamp (0=never) |
| 161 | signature | 64 | Ed25519 signature |

### CompactOrder (65 bytes)

Order without market/mints (derived from context).

| Offset | Field | Size |
|--------|-------|------|
| 0 | nonce | 8 |
| 8 | maker | 32 |
| 40 | side | 1 |
| 41 | maker_amount | 8 |
| 49 | taker_amount | 8 |
| 57 | expiration | 8 |

## Order Functions

```python
from lightcone_sdk.program import (
    create_bid_order,
    create_ask_order,
    create_signed_bid_order,
    create_signed_ask_order,
    hash_order,
    sign_order,
    verify_order_signature,
    serialize_full_order,
    deserialize_full_order,
    serialize_compact_order,
    deserialize_compact_order,
    to_compact_order,
    validate_order,
    validate_signed_order,
)
from lightcone_sdk.shared import BidOrderParams, AskOrderParams

# Create orders
bid_order = create_bid_order(BidOrderParams(
    nonce=1,
    maker=maker_pubkey,
    market=market_pubkey,
    base_mint=yes_token_mint,
    quote_mint=no_token_mint,
    maker_amount=1_000_000,   # Quote tokens given
    taker_amount=500_000,     # Base tokens received
    expiration=0,
))

ask_order = create_ask_order(AskOrderParams(
    nonce=1,
    maker=maker_pubkey,
    market=market_pubkey,
    base_mint=yes_token_mint,
    quote_mint=no_token_mint,
    maker_amount=500_000,     # Base tokens given
    taker_amount=1_000_000,   # Quote tokens received
    expiration=0,
))

# Create and sign in one step
from solders.keypair import Keypair
keypair = Keypair()

signed_bid = create_signed_bid_order(bid_params, keypair)
signed_ask = create_signed_ask_order(ask_params, keypair)

# Or sign separately
order = create_bid_order(bid_params)
signature = sign_order(order, keypair)
order.signature = signature

# Hash order (keccak256)
order_hash = hash_order(order)  # 32 bytes

# Verify signature
is_valid = verify_order_signature(order)

# Serialize/deserialize
order_bytes = serialize_full_order(order)     # 225 bytes
order = deserialize_full_order(order_bytes)

compact_bytes = serialize_compact_order(compact)  # 65 bytes
compact = deserialize_compact_order(compact_bytes)

# Convert to compact
compact = to_compact_order(full_order)

# Validation
validate_order(order)         # Raises if invalid
validate_signed_order(order)  # Also verifies signature
```

### Client Helper Methods

```python
# Create orders via client
bid = client.create_bid_order(bid_params)
ask = client.create_ask_order(ask_params)

# Create and sign
signed_bid = client.create_signed_bid_order(bid_params, keypair)
signed_ask = client.create_signed_ask_order(ask_params, keypair)

# Hash and sign
order_hash = client.hash_order(order)
signature = client.sign_order(order, keypair)
```

## Ed25519 Verification

Three strategies for signature verification in transactions.

### Individual Verification

One Ed25519 instruction per order (largest tx size).

```python
from lightcone_sdk.program import (
    build_ed25519_verify_instruction,
    build_ed25519_verify_instruction_for_order,
)

# From raw components
ix = build_ed25519_verify_instruction(pubkey_bytes, message_bytes, signature_bytes)

# From order
ix = build_ed25519_verify_instruction_for_order(order)
```

### Batch Verification

Multiple signatures in single Ed25519 instruction.

```python
from lightcone_sdk.program import build_ed25519_batch_verify_instruction

orders = [taker_order, maker_order_1, maker_order_2]
ix = build_ed25519_batch_verify_instruction(orders)
```

### Cross-Reference Verification

Smallest transaction size. Ed25519 instructions reference data within the match instruction.

```python
from lightcone_sdk.program import (
    create_cross_ref_ed25519_instructions,
    create_single_cross_ref_ed25519_instruction,
    CrossRefEd25519Params,
    MatchIxOffsets,
)

num_makers = 3
match_ix_index = 1 + num_makers  # Ed25519 instructions first

# Create all verification instructions
ed25519_ixs = create_cross_ref_ed25519_instructions(num_makers, match_ix_index)

# Or create individual cross-ref instruction
params = CrossRefEd25519Params(
    instruction_index=match_ix_index,
    pubkey_offset=MatchIxOffsets.TAKER_MAKER_OFFSET,
    message_offset=MatchIxOffsets.TAKER_ORDER_OFFSET,
    message_size=MatchIxOffsets.FULL_ORDER_SIZE - MatchIxOffsets.SIGNATURE_SIZE,
    signature_offset=MatchIxOffsets.TAKER_ORDER_OFFSET + MatchIxOffsets.SIGNATURE_OFFSET,
)
ix = create_single_cross_ref_ed25519_instruction(params)
```

## Instruction Builders

Low-level instruction construction.

```python
from lightcone_sdk.program import (
    build_initialize_instruction,
    build_create_market_instruction,
    build_create_market_instruction_with_id,
    build_add_deposit_mint_instruction,
    build_mint_complete_set_instruction,
    build_merge_complete_set_instruction,
    build_cancel_order_instruction,
    build_increment_nonce_instruction,
    build_settle_market_instruction,
    build_redeem_winnings_instruction,
    build_set_paused_instruction,
    build_set_operator_instruction,
    build_withdraw_from_position_instruction,
    build_activate_market_instruction,
    build_match_orders_multi_instruction,
)
```

## Account Deserialization

```python
from lightcone_sdk.program import (
    deserialize_exchange,
    deserialize_market,
    deserialize_position,
    deserialize_order_status,
    deserialize_user_nonce,
)

# From raw account data bytes
exchange = deserialize_exchange(account_data)
market = deserialize_market(account_data)
position = deserialize_position(account_data)
order_status = deserialize_order_status(account_data)
user_nonce = deserialize_user_nonce(account_data)
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `FULL_ORDER_SIZE` | 225 | Full order byte size |
| `COMPACT_ORDER_SIZE` | 65 | Compact order byte size |
| `SIGNATURE_SIZE` | 64 | Ed25519 signature size |
| `ORDER_HASH_SIZE` | 32 | Keccak256 hash size |
| `MAX_MAKERS` | 5 | Max makers per match |
| `MAX_OUTCOMES` | 6 | Max outcomes per market |
| `MIN_OUTCOMES` | 2 | Min outcomes per market |

## Type Definitions

```python
from lightcone_sdk.shared import (
    MarketStatus,      # PENDING=0, ACTIVE=1, RESOLVED=2, CANCELLED=3
    OrderSide,         # BID=0, ASK=1
    Exchange,          # Exchange account data
    Market,            # Market account data
    Position,          # Position account data
    OrderStatus,       # Order status data
    UserNonce,         # User nonce data
    FullOrder,         # Full order struct
    CompactOrder,      # Compact order struct
    MakerFill,         # Maker + fill amount
    OutcomeMetadata,   # Outcome name/symbol/uri
)
```

## Errors

```python
from lightcone_sdk.shared import (
    LightconeError,
    InvalidDiscriminatorError,
    AccountNotFoundError,
    InvalidAccountDataError,
    InvalidOrderError,
    InvalidSignatureError,
    OrderExpiredError,
    InsufficientBalanceError,
    MarketNotActiveError,
    ExchangePausedError,
    InvalidOutcomeError,
    TooManyMakersError,
    OrdersDoNotCrossError,
)
```
