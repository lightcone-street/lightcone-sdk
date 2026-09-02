# Program Module Reference

On-chain interaction with the Lightcone Solana program.

This module contains all program-specific code including:
- **Types** - Enums, account data structures, order types, parameter types
- **Constants** - Program IDs, PDA seeds, discriminators, sizes, limits
- **Errors** - Custom exception classes
- **Utilities** - Hashing, encoding, ATA derivation, validation
- **Client** - High-level async client for program interaction
- **PDA Functions** - Program Derived Address derivation
- **Instructions** - Low-level instruction builders
- **Orders** - Order creation, signing, serialization
- **Ed25519** - Signature verification helpers

## Client

### LightconePinocchioClient

```python
from solana.rpc.async_api import AsyncClient
from lightcone_sdk.program import LightconePinocchioClient, PROGRAM_ID

# Create client
connection = AsyncClient("https://api.mainnet-beta.solana.com")
client = LightconePinocchioClient(connection)

# With custom program ID (e.g., devnet)
client = LightconePinocchioClient(connection, program_id=custom_program_id)
```

## Types

### Enums

```python
from lightcone_sdk.program import MarketStatus, OrderSide

# MarketStatus
MarketStatus.PENDING    # 0 - Not yet active
MarketStatus.ACTIVE     # 1 - Trading enabled
MarketStatus.RESOLVED   # 2 - Market settled
MarketStatus.CANCELLED  # 3 - Market cancelled

# OrderSide
OrderSide.BID  # 0 - Buyer gives quote, receives base
OrderSide.ASK  # 1 - Seller gives base, receives quote
```

### Account Types

```python
from lightcone_sdk.program import (
    Exchange,
    GlobalDepositToken,
    Market,
    PendingRoleKind,
    Position,
    OrderStatus,
    UserNonce,
)
```

#### Exchange

| Field | Type | Description |
|-------|------|-------------|
| `authority` | Pubkey | Admin authority |
| `operator` | Pubkey | Order matching operator |
| `manager` | Pubkey | Market and orderbook manager |
| `market_count` | int | Number of markets created |
| `paused` | bool | Trading paused |
| `bump` | int | PDA bump seed |
| `deposit_token_count` | int | Number of whitelisted deposit tokens |
| `fee_receiver` | Pubkey | Current protocol fee receiver |
| `pending_role` | Pubkey | Pending privileged-role recipient |
| `pending_role_kind` | PendingRoleKind | Pending role kind: none, authority, manager, or operator |

#### Market

| Field | Type | Description |
|-------|------|-------------|
| `market_id` | int | Sequential market ID |
| `num_outcomes` | int | Number of outcomes |
| `status` | MarketStatus | Current status |
| `bump` | int | PDA bump seed |
| `maker_fee_bps` | int | Maker fee in basis points |
| `taker_fee_bps` | int | Taker fee in basis points |
| `oracle` | Pubkey | Oracle authority |
| `question_id` | bytes | Question identifier |
| `condition_id` | bytes | Computed condition ID |
| `payout_numerators` | tuple[int, int, int, int, int, int] | Resolution vector; first `num_outcomes` entries are meaningful |
| `payout_denominator` | int | Sum of meaningful payout numerators |
| `deposit_mint_count` | int | Deposit mints registered through `add_deposit_mint` (byte 148; capped at `MAX_DEPOSIT_MINTS_PER_MARKET`) |

#### GlobalDepositToken

| Field | Type | Description |
|-------|------|-------------|
| `mint` | Pubkey | Whitelisted deposit mint |
| `bump` | int | PDA bump seed |
| `index` | int | Deposit token ordering index |
| `active` | bool | Backend-visible status flag |

#### Position

| Field | Type | Description |
|-------|------|-------------|
| `owner` | Pubkey | Position owner |
| `market` | Pubkey | Market address |
| `bump` | int | PDA bump seed |

#### OrderStatus

| Field | Type | Description |
|-------|------|-------------|
| `remaining` | int | Remaining order amount |
| `base_remaining` | int | Remaining base-side amount |
| `is_cancelled` | bool | Cancelled flag |

#### UserNonce

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | int | Current nonce value |

### Order Types

```python
from lightcone_sdk.program import FullOrder, CompactOrder, MakerFill
```

#### FullOrder (233 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | int | Order nonce |
| `maker` | Pubkey | Maker public key |
| `market` | Pubkey | Market address |
| `base_mint` | Pubkey | Base token mint |
| `quote_mint` | Pubkey | Quote token mint |
| `side` | OrderSide | BID or ASK |
| `amount_in` | int | Amount maker gives |
| `amount_out` | int | Amount maker receives |
| `expiration` | int | Expiration timestamp |
| `signature` | bytes | Ed25519 signature (64 bytes) |

#### CompactOrder (37 bytes)

Compact instruction form with `nonce`, `salt`, `side`, `amount_in`, `amount_out`, and `expiration`.

#### MakerFill

| Field | Type | Description |
|-------|------|-------------|
| `order` | FullOrder | The maker order |
| `fill_amount` | int | Amount to fill |

### Parameter Types

```python
from lightcone_sdk.program import (
    InitializeParams,
    CreateMarketParams,
    AddDepositMintParams,
    MintCompleteSetParams,
    MergeCompleteSetParams,
    SettleMarketParams,
    RedeemWinningsParams,
    WithdrawConditionalFromPositionParams,
    WithdrawFromPositionParams,
    ActivateMarketParams,
    MatchOrdersMultiParams,
    BidOrderParams,
    AskOrderParams,
    OutcomeMetadata,
)
```

## Constants

### Program IDs

```python
from lightcone_sdk.program import (
    PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    SYSTEM_PROGRAM_ID,
    RENT_SYSVAR_ID,
    INSTRUCTIONS_SYSVAR_ID,
    INITIALIZE_AUTHORITY,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `PROGRAM_ID` | Production default, derived from `LightconeEnv.PROD` | Lightcone program |
| `TOKEN_PROGRAM_ID` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | SPL Token program |
| `ASSOCIATED_TOKEN_PROGRAM_ID` | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | ATA program |
| `SYSTEM_PROGRAM_ID` | `11111111111111111111111111111111` | System program |
| `RENT_SYSVAR_ID` | `SysvarRent111111111111111111111111111111111` | Rent sysvar |
| `INSTRUCTIONS_SYSVAR_ID` | `Sysvar1nstructions1111111111111111111111111` | Instructions sysvar |
| `INITIALIZE_AUTHORITY` | `2m6iAtMVmd3jE2BpNxoa9E79Kj7NeE6UxBFNyCBp6QEb` | Program initialization authority |

### Current Program Alignment Notes

- `Exchange` and `Market` accounts are 216 bytes.
- `GlobalDepositToken` accounts are 47 bytes, with `bump` at offset 40, `index` at 41..43, and `active` at offset 43.
- `set_authority`, `set_manager`, and `set_operator` now propose role transfers. The matching `accept_authority`, `accept_manager`, or `accept_operator` instruction performs the effective role change.
- `match_orders_multi` and `deposit_and_swap` include the fee receiver and associated token program in their fixed account lists.
- `set_fee_receiver_with_atas` can append quote mint / fee receiver ATA pairs for idempotent ATA creation.
- `refresh_orderbook_alt` appends the current fee receiver quote ATA when missing, but does not fully reshape older orderbook ALTs.
- Instruction discriminators are current through `SET_DEPOSIT_TOKEN_STATUS = 38`; `INSTRUCTION_EVENT_BATCH = 255` is reserved for the program's private event self-CPI and is never built by the SDK.
- Every public instruction ends with the event transport trailer (event-authority PDA, then the program account; both read-only, non-signer). Every `build_*_instruction` appends it automatically; see [Event Transport Trailer](#event-transport-trailer).
- `add_deposit_mint` writes the market account and increments `Market.deposit_mint_count`, capped at `MAX_DEPOSIT_MINTS_PER_MARKET`.
- `init_position_tokens` is idempotent per `recent_slot` and `extend_position_tokens` is permissionless (`payer` replaces `operator`); both accept at most `MAX_DEPOSIT_MINTS_PER_IX` deposit mints per instruction.

### PDA Seeds

```python
from lightcone_sdk.program import (
    SEED_CENTRAL_STATE,
    SEED_MARKET,
    SEED_VAULT,
    SEED_MINT_AUTHORITY,
    SEED_CONDITIONAL_MINT,
    SEED_ORDER_STATUS,
    SEED_USER_NONCE,
    SEED_POSITION,
    SEED_EVENT_AUTHORITY,
)
```

### Account Discriminators

```python
from lightcone_sdk.program import (
    EXCHANGE_DISCRIMINATOR,
    MARKET_DISCRIMINATOR,
    ORDER_STATUS_DISCRIMINATOR,
    POSITION_DISCRIMINATOR,
    USER_NONCE_DISCRIMINATOR,
)
```

### Sizes and Limits

```python
from lightcone_sdk.program import (
    # Account sizes
    EXCHANGE_SIZE,      # 216 bytes
    MARKET_SIZE,        # 216 bytes
    ORDER_STATUS_SIZE,  # 32 bytes
    USER_NONCE_SIZE,    # 16 bytes
    POSITION_SIZE,      # 80 bytes
    GLOBAL_DEPOSIT_TOKEN_SIZE, # 47 bytes

    # Order sizes
    FULL_ORDER_SIZE,    # 233 bytes
    COMPACT_ORDER_SIZE, # 37 bytes
    SIGNATURE_SIZE,     # 64 bytes
    ORDER_HASH_SIZE,    # 32 bytes

    # Limits
    MAX_OUTCOMES,       # 6
    MIN_OUTCOMES,       # 2
    MAX_MAKERS,         # 5
    MAX_DEPOSIT_MINTS_PER_MARKET,  # 8
    MAX_DEPOSIT_MINTS_PER_IX,      # 8
)
```

### Event Transport Trailer

The program authenticates its accounting events with one PDA-signed self-CPI per
successful public instruction. To make that possible, every public instruction
must end with two read-only, non-signer accounts:

1. the event-authority PDA (`get_event_authority_pda(program_id)`, seed `__event_authority`)
2. the executable program account

The program pops both before dispatch and fails closed (on-chain error 21 or 68)
if either is missing, wrong, or writable, so a legacy instruction without the
trailer never mutates state. Public instructions must also be transaction-level:
invoking one through another program's CPI fails with error 73. Every
`build_*_instruction` in this module appends the trailer automatically, so
consumers that enumerate instruction accounts should expect the two trailing
entries. The SDK never builds or decodes the event batch itself
(`INSTRUCTION_EVENT_BATCH = 255` is reserved), and it does not attach a
compute-budget instruction for the self-CPI.

```python
from lightcone_sdk.program import PROGRAM_ID, get_event_authority_pda

event_authority, bump = get_event_authority_pda(PROGRAM_ID)
```

## Errors

```python
from lightcone_sdk.program import (
    LightconeError,           # Base error class
    InvalidDiscriminatorError, # Account discriminator mismatch
    AccountNotFoundError,      # Account does not exist
    InvalidAccountDataError,   # Account data malformed
    InvalidOrderError,         # Order validation failed
    InvalidSignatureError,     # Signature verification failed
    OrderExpiredError,         # Order has expired
    InsufficientBalanceError,  # Insufficient funds
    MarketNotActiveError,      # Market not in active state
    ExchangePausedError,       # Exchange is paused
    InvalidOutcomeError,       # Invalid outcome index
    TooManyMakersError,        # Exceeds MAX_MAKERS
    OrdersDoNotCrossError,     # Orders don't match
    TooManyDepositMintsError,  # Exceeds MAX_DEPOSIT_MINTS_PER_IX (on-chain error 75)
    InvalidEventAuthorityError,  # On-chain error 68: bad event transport trailer
    PublicInstructionMustBeTopLevelError,  # On-chain error 73: public ix invoked via CPI
)
```

## Utilities

### Hashing

```python
from lightcone_sdk.program import keccak256, derive_condition_id

# Hash arbitrary data
hash_bytes = keccak256(b"data to hash")  # Returns 32 bytes

# Derive condition ID for a market
condition_id = derive_condition_id(oracle_pubkey, question_id, num_outcomes)
```

### Associated Token Addresses

```python
from lightcone_sdk.program import (
    get_associated_token_address,
    get_associated_token_address_2022,
)

# SPL Token ATA
ata = get_associated_token_address(owner_pubkey, mint_pubkey)

# Token-2022 ATA
ata = get_associated_token_address_2022(owner_pubkey, mint_pubkey)
```

### Order Crossing

```python
from lightcone_sdk.program import orders_cross

# Check if a bid and ask order would match
crosses = orders_cross(
    bid_maker_amount, bid_taker_amount,
    ask_maker_amount, ask_taker_amount
)  # Returns bool
```

## PDA Derivation

```python
from lightcone_sdk.program import (
    get_exchange_pda,
    get_event_authority_pda,
    get_market_pda,
    get_vault_pda,
    get_mint_authority_pda,
    get_conditional_mint_pda,
    get_order_status_pda,
    get_user_nonce_pda,
    get_position_pda,
    get_all_conditional_mints,
    PROGRAM_ID,
)

# Exchange PDA
exchange_pda, bump = get_exchange_pda(PROGRAM_ID)

# Event-authority PDA (appended to every public instruction by the builders)
event_authority, bump = get_event_authority_pda(PROGRAM_ID)

# Market PDA
market_pda, bump = get_market_pda(market_id, PROGRAM_ID)

# Vault PDA (token account)
vault_pda, bump = get_vault_pda(deposit_mint, market_pubkey, PROGRAM_ID)

# Mint authority PDA
mint_auth_pda, bump = get_mint_authority_pda(market_pubkey, PROGRAM_ID)

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
    BidOrderParams,
    AskOrderParams,
)
from solders.keypair import Keypair

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

# Create and sign in one step
keypair = Keypair()
rules = await client.orderbooks().decimals(orderbook_id)
signed_bid = create_signed_bid_order(bid_params, keypair, rules)

# Signing helpers require fetched OrderbookRules and preflight the exact price
# ratio, size quantum, and durable field ranges before producing a signature.

# Hash order (keccak256)
order_hash = hash_order(order)  # 32 bytes

# Verify signature
is_valid = verify_order_signature(order)

# Serialize/deserialize
order_bytes = serialize_full_order(order)     # 233 bytes
order = deserialize_full_order(order_bytes)

# Validation
validate_order(order)         # Raises if invalid
validate_signed_order(order)  # Also verifies signature
```

## Transaction Builders

All transaction builders return a `Transaction` ready for signing.

### Exchange Administration & Market Lifecycle

Exchange administration and market lifecycle instructions (initialize, create/activate/settle market, pause, operator changes) are exposed as raw encoders for advanced usage:

```python
from lightcone_sdk.program import (
    ScalarResolutionParams,
    build_create_market_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    scalar_to_payout_numerators,
)

# Scalar settlement uses integer fixed-point metadata off-chain.
payout_numerators = scalar_to_payout_numerators(ScalarResolutionParams(
    min_value=0,
    max_value=100,
    resolved_value=25,
    lower_outcome_index=0,
    upper_outcome_index=1,
    num_outcomes=2,
))
```

### Position Management

```python
from lightcone_sdk.program import (
    MintCompleteSetParams,
    MergeCompleteSetParams,
    RedeemWinningsParams,
    WithdrawConditionalFromPositionParams,
    WithdrawFromPositionParams,
)

# Mint complete set (deposit collateral, receive outcome tokens)
tx = await client.mint_complete_set(
    MintCompleteSetParams(
        user=user_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        amount=1_000_000,
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
    outcome_index=0,
)

# Withdraw conditional tokens from position account.
# WithdrawFromPositionParams is a compatibility alias for the same layout.
tx = client.positions().withdraw_conditional_from_position_tx(
    WithdrawConditionalFromPositionParams(
        user=user_pubkey,
        market=market_pubkey,
        deposit_mint=usdc_mint,
        amount=500_000,
        outcome_index=0,
    ),
)
```

### Order Matching

Three strategies with different transaction size/verification tradeoffs:

```python
from lightcone_sdk.program import MakerFill

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
tx = await client.match_orders_multi_with_verify(...)

# 3. With cross-reference Ed25519 (smallest transaction size)
tx = await client.match_orders_multi_cross_ref(...)
```

## Client Utility Methods

The client provides convenience methods for address derivation and order management:

### Address Derivation

```python
# Get PDA addresses without making RPC calls
exchange_addr = client.get_exchange_address()
market_addr = client.get_market_address(market_id=0)
position_addr = client.get_position_address(owner_pubkey, market_pubkey)
order_status_addr = client.get_order_status_address(order_hash_bytes)
user_nonce_addr = client.get_user_nonce_address(user_pubkey)

# Get market by address (makes RPC call)
market = await client.get_market_by_address(market_pubkey)
```

| Method | Returns | Description |
|--------|---------|-------------|
| `get_exchange_address()` | Pubkey | Exchange PDA |
| `get_market_address(market_id)` | Pubkey | Market PDA for given ID |
| `get_position_address(owner, market)` | Pubkey | Position PDA |
| `get_order_status_address(order_hash)` | Pubkey | Order status PDA |
| `get_user_nonce_address(user)` | Pubkey | User nonce PDA |
| `get_market_by_address(address)` | Market | Fetch and deserialize market |

### Order Helpers

These are low-level wire/signature helpers. Every signing operation requires
fetched rules and applies the same exact preflight as the envelope path.

```python
from lightcone_sdk.program import BidOrderParams, AskOrderParams
from solders.keypair import Keypair

keypair = Keypair()
rules = await client.orderbooks().decimals(orderbook_id)

# Create unsigned orders
bid = client.create_bid_order(BidOrderParams(...))
ask = client.create_ask_order(AskOrderParams(...))

# Create and sign in one step
signed_bid = client.create_signed_bid_order(BidOrderParams(...), keypair, rules)
signed_ask = client.create_signed_ask_order(AskOrderParams(...), keypair, rules)

# Manual hash and sign
order_hash = client.hash_order(order)  # Returns 32 bytes
signature = client.sign_order(order, keypair, rules)  # Returns 64 bytes
```

| Method | Returns | Description |
|--------|---------|-------------|
| `create_bid_order(params)` | FullOrder | Create unsigned bid |
| `create_ask_order(params)` | FullOrder | Create unsigned ask |
| `create_signed_bid_order(params, keypair, rules)` | FullOrder | Preflight and sign bid |
| `create_signed_ask_order(params, keypair, rules)` | FullOrder | Preflight and sign ask |
| `hash_order(order)` | bytes | Keccak256 hash (32 bytes) |
| `sign_order(order, keypair, rules)` | bytes | Exact-preflight Ed25519 signature (64 bytes) |

### Condition Helpers

```python
# Derive condition ID
condition_id = client.derive_condition_id(
    oracle=oracle_pubkey,
    question_id=question_id_bytes,  # 32 bytes
    num_outcomes=2,
)

# Get all conditional mint addresses
mints = client.get_conditional_mints(
    market=market_pubkey,
    deposit_mint=usdc_mint,
    num_outcomes=2,
)  # Returns list[Pubkey]
```

## Ed25519 Verification

```python
from lightcone_sdk.program import (
    build_ed25519_verify_instruction,
    build_ed25519_verify_instruction_for_order,
    build_ed25519_batch_verify_instruction,
    create_cross_ref_ed25519_instructions,
    CrossRefEd25519Params,
    MatchIxOffsets,
)

# Individual verification
ix = build_ed25519_verify_instruction_for_order(order)

# Batch verification
orders = [taker_order, maker_order_1, maker_order_2]
ix = build_ed25519_batch_verify_instruction(orders)

# Cross-reference verification (smallest tx size)
ed25519_ixs = create_cross_ref_ed25519_instructions(num_makers, match_ix_index)
```

### Ed25519 Types

#### CrossRefEd25519Params

Parameters for cross-reference Ed25519 verification:

| Field | Type | Description |
|-------|------|-------------|
| `num_signatures` | int | Number of signatures to verify |
| `signature_offset` | int | Offset to signature in instruction data |
| `signature_instruction_index` | int | Index of instruction containing signature |
| `public_key_offset` | int | Offset to public key in instruction data |
| `public_key_instruction_index` | int | Index of instruction containing public key |
| `message_data_offset` | int | Offset to message in instruction data |
| `message_data_size` | int | Size of message data |
| `message_instruction_index` | int | Index of instruction containing message |

#### MatchIxOffsets

Offsets for signature data within match instruction:

| Field | Type | Description |
|-------|------|-------------|
| `taker_pubkey` | int | Offset to taker public key |
| `taker_signature` | int | Offset to taker signature |
| `taker_message` | int | Offset to taker message |
| `taker_message_len` | int | Length of taker message |
| `maker_offsets` | list | List of maker offset tuples |

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

## Instruction Builders

Low-level instruction construction:

```python
from lightcone_sdk.program import (
    build_initialize_instruction,
    build_create_market_instruction,
    build_add_deposit_mint_instruction,
    build_mint_complete_set_instruction,
    build_merge_complete_set_instruction,
    build_cancel_order_instruction,
    build_increment_nonce_instruction,
    build_settle_market_instruction,
    build_redeem_winnings_instruction,
    build_set_paused_instruction,
    build_set_operator_instruction,
    build_withdraw_conditional_from_position_instruction,
    build_withdraw_from_position_instruction,
    build_activate_market_instruction,
    build_match_orders_multi_instruction,
    build_create_orderbook_instruction,
    build_refresh_orderbook_alt_instruction,
    build_set_authority_instruction,
    build_set_manager_instruction,
    build_accept_authority_instruction,
    build_accept_manager_instruction,
    build_accept_operator_instruction,
    build_set_oracle_instruction,
    build_set_fee_receiver_instruction,
    build_set_fee_receiver_with_atas_instruction,
    build_whitelist_deposit_token_instruction,
    build_set_deposit_token_status_instruction,
    build_deposit_to_global_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_deposit_and_swap_instruction,
    build_extend_position_tokens_instruction,
    build_withdraw_from_global_instruction,
    build_close_position_alt_instruction,
    build_close_order_status_instruction,
    build_close_position_token_accounts_instruction,
    build_close_orderbook_alt_instruction,
    build_close_orderbook_instruction,
)
```
