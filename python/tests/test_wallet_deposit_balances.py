import json
import time

import pytest
from solders.hash import Hash
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.system_program import (
    ID as SYSTEM_PROGRAM_ID,
)
from solders.system_program import (
    decode_transfer,
)
from spl.token.constants import (
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    WRAPPED_SOL_MINT,
)
from spl.token.instructions import (
    decode_close_account,
    decode_sync_native,
)
from spl.token.instructions import (
    decode_transfer as decode_token_transfer,
)

from lightcone_sdk.auth import AuthCredentials
from lightcone_sdk.domain.market import Market
from lightcone_sdk.domain.position import (
    DepositTokenBalance,
    DepositTokenBalancesSnapshot,
    SolActionCosts,
    SolActionKind,
    SolBalanceAvailability,
    SolBalanceComponents,
    WalletDepositBalancesApplyResult,
    WalletDepositBalanceSnapshot,
    WalletDepositBalancesState,
    WalletDepositBalanceStatus,
    WalletDepositBalanceStatusEvent,
    WalletDepositBalanceUpdate,
    WalletNativeSolBalanceUpdate,
)
from lightcone_sdk.domain.position.client import Positions, native_withdraw_seed
from lightcone_sdk.error import SdkError
from lightcone_sdk.program import InvalidOutcomeIndexError, get_associated_token_address
from lightcone_sdk.rpc import CanonicalWsolAccountInfo
from lightcone_sdk.shared.signing import (
    ExternalSigner,
    SigningStrategy,
    SigningStrategyKind,
)
from lightcone_sdk.ws import parse_message_in

WSOL_MINT = str(WRAPPED_SOL_MINT)


def balance(mint: str, idle: str) -> DepositTokenBalance:
    return DepositTokenBalance(
        mint=mint,
        idle=idle,
        symbol="TOKEN",
        name="Token",
    )


def initialized_state(wallet: str) -> WalletDepositBalancesState:
    state = WalletDepositBalancesState()
    state.apply_rest_snapshot(
        wallet,
        DepositTokenBalancesSnapshot(
            context_slot=100,
            native_sol_balance="2.000000000",
            balances={WSOL_MINT: balance(WSOL_MINT, "0.500000000")},
        ),
    )
    return state


def test_nested_wallet_balance_wire_variants_are_strictly_decoded() -> None:
    snapshot = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_snapshot",
                    "wallet_address": "wallet-a",
                    "context_slot": 10,
                    "native_sol_balance": "1.000000000",
                    "balances": {
                        "MintA": {
                            "mint": "MintA",
                            "idle": "1.000000000",
                            "symbol": "TOKEN",
                            "name": "Token",
                            "icon_url_low": None,
                        }
                    },
                },
            }
        )
    ).data
    native = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_native_sol_balance_update",
                    "wallet_address": "wallet-a",
                    "context_slot": 11,
                    "native_sol_balance": "1.000000001",
                },
            }
        )
    ).data
    balance_update = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_update",
                    "wallet_address": "wallet-a",
                    "context_slot": 12,
                    "balance": {
                        "mint": "MintA",
                        "idle": "2.000000000",
                        "symbol": "TOKEN",
                        "name": "Token",
                    },
                },
            }
        )
    ).data
    status = parse_message_in(
        json.dumps(
            {
                "type": "wallet_deposit_balances",
                "data": {
                    "event_type": "wallet_deposit_balance_status",
                    "wallet_address": "wallet-a",
                    "status": "reconnecting",
                    "code": "RPC_UNAVAILABLE",
                },
            }
        )
    ).data

    assert isinstance(snapshot, WalletDepositBalanceSnapshot)
    assert snapshot.balances["MintA"].icon_url_low is None
    assert isinstance(native, WalletNativeSolBalanceUpdate)
    assert isinstance(balance_update, WalletDepositBalanceUpdate)
    assert balance_update.balance.mint == "MintA"
    assert isinstance(status, WalletDepositBalanceStatusEvent)
    assert status.status is WalletDepositBalanceStatus.RECONNECTING

    malformed = {
        "type": "wallet_deposit_balances",
        "data": {
            "event_type": "wallet_native_sol_balance_update",
            "wallet_address": "wallet-a",
            "context_slot": 11,
            "native_sol_balance": "1.0",
        },
    }
    with pytest.raises(TypeError, match="exactly nine decimal places"):
        parse_message_in(json.dumps(malformed))


@pytest.mark.parametrize(
    "message",
    [
        {"type": "wallet_deposit_balances"},
        {"type": "wallet_deposit_balances", "data": None},
        {"type": "wallet_deposit_balances", "data": []},
        {"type": "wallet_deposit_balances", "data": "invalid"},
        {"type": "wallet_deposit_balances", "data": 1},
    ],
)
def test_wallet_balance_wire_requires_object_data(message: dict[str, object]) -> None:
    with pytest.raises(TypeError, match="data must be an object"):
        parse_message_in(json.dumps(message))


def test_state_replacement_component_updates_zero_removal_and_exact_combined_sol() -> (
    None
):
    state = initialized_state("wallet-a")
    assert state.combined_sol_balance() == "2.500000000"

    assert (
        state.apply_event(
            WalletNativeSolBalanceUpdate(
                event_type="wallet_native_sol_balance_update",
                wallet_address="wallet-b",
                context_slot=101,
                native_sol_balance="9.000000000",
            )
        )
        is WalletDepositBalancesApplyResult.IGNORED
    )
    assert (
        state.apply_event(
            WalletNativeSolBalanceUpdate(
                event_type="wallet_native_sol_balance_update",
                wallet_address="wallet-a",
                context_slot=101,
                native_sol_balance="2.000000001",
            )
        )
        is WalletDepositBalancesApplyResult.APPLIED
    )
    assert state.combined_sol_balance() == "2.500000001"

    state.apply_event(
        WalletDepositBalanceUpdate(
            event_type="wallet_deposit_balance_update",
            wallet_address="wallet-a",
            context_slot=102,
            balance=balance("MintHighPrecision", "0.000000000001"),
        )
    )
    assert "MintHighPrecision" in state.balances
    state.apply_event(
        WalletDepositBalanceUpdate(
            event_type="wallet_deposit_balance_update",
            wallet_address="wallet-a",
            context_slot=103,
            balance=balance("MintHighPrecision", "0.000000000000"),
        )
    )
    assert "MintHighPrecision" not in state.balances

    before_slot = state.context_slot
    before_balances = dict(state.balances)
    assert (
        state.apply_event(
            WalletDepositBalanceUpdate(
                event_type="wallet_deposit_balance_update",
                wallet_address="wallet-a",
                context_slot=104,
                balance=balance("MintHighPrecision", "-1"),
            )
        )
        is WalletDepositBalancesApplyResult.REJECTED
    )
    assert state.context_slot == before_slot
    assert state.balances == before_balances

    state.apply_event(
        WalletDepositBalanceSnapshot(
            event_type="wallet_deposit_balance_snapshot",
            wallet_address="wallet-c",
            context_slot=90,
            native_sol_balance="3.000000000",
            balances={},
        )
    )
    assert state.wallet_address == "wallet-c"
    assert state.context_slot == 90
    assert state.combined_sol_balance() == "3.000000000"

    assert (
        state.apply_event(
            WalletDepositBalanceStatusEvent(
                event_type="wallet_deposit_balance_status",
                wallet_address="wallet-c",
                status=WalletDepositBalanceStatus.METADATA_UNAVAILABLE,
                code="METADATA_UNAVAILABLE",
            )
        )
        is WalletDepositBalancesApplyResult.IGNORED
    )


def test_transaction_components_reject_u64_overflow() -> None:
    """Keep broad display arithmetic while rejecting transaction-range overflow."""
    state = WalletDepositBalancesState()
    state.apply_rest_snapshot(
        "wallet-a",
        DepositTokenBalancesSnapshot(
            context_slot=1,
            native_sol_balance="18446744073.709551616",
            balances={},
        ),
    )
    assert state.combined_sol_balance() == "18446744073.709551616"
    with pytest.raises(SdkError, match="transaction u64 range"):
        state.sol_components()


class FakeAuth:
    """Expose cached credentials at the same planning boundary as real auth."""

    def __init__(self, credentials: AuthCredentials | None) -> None:
        """Store credentials without synthesizing freshness or identity."""
        self._credentials = credentials

    def credentials(self) -> AuthCredentials | None:
        """Return the exact cached credentials supplied by the test."""
        return self._credentials


class FakeRpc:
    """Deterministic account, blockhash, rent, and fee authority for planners."""

    def __init__(
        self,
        wallet: Pubkey,
        *,
        canonical_exists: bool = True,
        invalid_canonical_account: bool = False,
        occupied_temporary_attempts: int = 0,
        fees: list[int | None] | None = None,
        rent_lamports: int = 2_039_280,
        blockhashes: list[Hash] | None = None,
        canonical_token_amount_lamports: int = 500_000_000,
        canonical_account_lamports: int = 502_039_280,
        canonical_native_reserve_lamports: int = 2_039_280,
    ) -> None:
        """Configure canonical presence, collision count, and ordered live values."""
        self.canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        self.canonical_exists = canonical_exists
        self.invalid_canonical_account = invalid_canonical_account
        self.occupied_temporary_attempts = occupied_temporary_attempts
        self.fees = list(fees or [5_000])
        self.rent_lamports = rent_lamports
        self.blockhashes = list(blockhashes or [Hash.default()])
        self.canonical_token_amount_lamports = canonical_token_amount_lamports
        self.canonical_account_lamports = canonical_account_lamports
        self.canonical_native_reserve_lamports = canonical_native_reserve_lamports
        self.account_lookups: list[Pubkey] = []
        self.fee_calls = 0

    async def account_exists(self, address: Pubkey) -> bool:
        """Record reads and model canonical presence or temporary collisions."""
        self.account_lookups.append(address)
        if address == self.canonical:
            return self.canonical_exists
        if self.occupied_temporary_attempts > 0:
            self.occupied_temporary_attempts -= 1
            return True
        return False

    async def canonical_wsol_account_exists(
        self, address: Pubkey, _wallet: Pubkey
    ) -> bool:
        """Preserve ordinary planners' boolean compatibility surface."""
        return await self.canonical_wsol_account_info(address, _wallet) is not None

    async def canonical_wsol_account_info(
        self, address: Pubkey, _wallet: Pubkey
    ) -> CanonicalWsolAccountInfo | None:
        """Model exact canonical validation and close-return facts."""
        exists = await self.account_exists(address)
        if exists and self.invalid_canonical_account:
            raise SdkError("canonical WSOL token account is invalid")
        if not exists:
            return None
        return CanonicalWsolAccountInfo(
            account_lamports=self.canonical_account_lamports,
            token_amount_lamports=self.canonical_token_amount_lamports,
            native_reserve_lamports=self.canonical_native_reserve_lamports,
        )

    async def minimum_balance_for_rent_exemption(self, _data_len: int) -> int:
        """Return the configured rent-exempt minimum in lamports."""
        return self.rent_lamports

    async def get_latest_blockhash(self) -> Hash:
        """Return ordered freshness while retaining the final fallback value."""
        return (
            self.blockhashes.pop(0)
            if len(self.blockhashes) > 1
            else self.blockhashes[0]
        )

    async def prepare_and_estimate_transaction_fee(self, transaction) -> int:
        """Attach deterministic blockhash authority before fee estimation."""
        transaction.partial_sign([], await self.get_latest_blockhash())
        return await self.estimate_prepared_transaction_fee(transaction)

    async def estimate_prepared_transaction_fee(self, _transaction) -> int:
        """Return ordered live fees or fail when authority is unavailable."""
        self.fee_calls += 1
        fee = self.fees.pop(0) if len(self.fees) > 1 else self.fees[0]
        if fee is None:
            raise SdkError("transaction fee estimate is unavailable")
        return fee


class FakePlanningClient:
    """Provide only the auth, program, and RPC surfaces consumed by planners."""

    def __init__(
        self,
        credentials: AuthCredentials | None,
        rpc: FakeRpc,
        strategy: SigningStrategy,
    ) -> None:
        """Bind deterministic identity and chain authority to one client."""
        self.program_id = Pubkey.default()
        self._auth = FakeAuth(credentials)
        self._rpc = rpc
        self._strategy = strategy

    def auth(self) -> FakeAuth:
        """Return the cached-identity facade."""
        return self._auth

    def rpc(self) -> FakeRpc:
        """Return deterministic chain authority."""
        return self._rpc

    def _require_signing_strategy(self) -> SigningStrategy:
        """Return the identity-bound signer used by planner preflight."""
        return self._strategy


class FakeExternalSigner(ExternalSigner):
    """Identity-only external signer used to prove conversion rejection order."""

    def __init__(self, wallet: Pubkey) -> None:
        """Expose the wallet without allowing any signer call."""
        self.wallet_address = str(wallet)

    async def sign_message(self, _message: bytes) -> bytes:
        """Fail if a planner unexpectedly reaches signing."""
        raise AssertionError("planner must not sign messages")

    async def sign_transaction(self, _tx_bytes: bytes) -> bytes:
        """Fail if a planner unexpectedly reaches signing."""
        raise AssertionError("planner must not sign transactions")


def planning_harness(
    *,
    native: str = "2.000000000",
    wrapped: str = "0.500000000",
    credentials: AuthCredentials | None = None,
    canonical_exists: bool = True,
    invalid_canonical_account: bool = False,
    occupied_temporary_attempts: int = 0,
    fees: list[int | None] | None = None,
    blockhashes: list[Hash] | None = None,
    signing_wallet_mismatch: bool = False,
    signing_kind: SigningStrategyKind = SigningStrategyKind.NATIVE,
    canonical_token_amount_lamports: int | None = None,
    canonical_account_lamports: int | None = None,
    canonical_native_reserve_lamports: int = 2_039_280,
    credential_expires_at: int | None = None,
) -> tuple[Positions, WalletDepositBalancesState, FakeRpc, Pubkey]:
    """Build complete wallet state and a deterministic planner dependency graph."""
    keypair = Keypair()
    wallet = keypair.pubkey()
    if credentials is None:
        credentials = AuthCredentials(
            user_id="user-a",
            wallet_address=str(wallet),
            expires_at=(
                int(time.time()) + 60
                if credential_expires_at is None
                else credential_expires_at
            ),
        )
    state = WalletDepositBalancesState()
    state.apply_rest_snapshot(
        str(wallet),
        DepositTokenBalancesSnapshot(
            context_slot=100,
            native_sol_balance=native,
            balances={WSOL_MINT: balance(WSOL_MINT, wrapped)},
        ),
    )
    live_token_amount = (
        state.canonical_wsol_lamports()
        if canonical_token_amount_lamports is None
        else canonical_token_amount_lamports
    )
    live_account_lamports = (
        live_token_amount + 2_039_280
        if canonical_account_lamports is None
        else canonical_account_lamports
    )
    rpc = FakeRpc(
        wallet,
        canonical_exists=canonical_exists,
        invalid_canonical_account=invalid_canonical_account,
        occupied_temporary_attempts=occupied_temporary_attempts,
        fees=fees,
        blockhashes=blockhashes,
        canonical_token_amount_lamports=live_token_amount,
        canonical_account_lamports=live_account_lamports,
        canonical_native_reserve_lamports=canonical_native_reserve_lamports,
    )
    signing_keypair = Keypair() if signing_wallet_mismatch else keypair
    if signing_kind is SigningStrategyKind.NATIVE:
        strategy = SigningStrategy.native(signing_keypair)
    elif signing_kind is SigningStrategyKind.WALLET_ADAPTER:
        strategy = SigningStrategy.wallet_adapter(FakeExternalSigner(wallet))
    else:
        strategy = SigningStrategy.privy("wallet-id", str(wallet))
    client = FakePlanningClient(credentials, rpc, strategy)
    return Positions(client), state, rpc, wallet  # type: ignore[arg-type]


def market() -> Market:
    """Return the smallest active market shape accepted by split planning."""
    return Market(
        id=1,
        pubkey=str(Pubkey.new_unique()),
        name="Market",
        definition="Test market",
        num_outcomes=2,
    )


def compiled_instruction(transaction, index: int) -> Instruction:
    """Expand one compiled instruction with its effective message privileges."""
    message = transaction.message
    compiled = message.instructions[index]
    keys = message.account_keys
    header = message.header
    writable_signed = (
        header.num_required_signatures - header.num_readonly_signed_accounts
    )
    writable_unsigned_end = len(keys) - header.num_readonly_unsigned_accounts

    def account_meta(account_index: int) -> AccountMeta:
        """Recover signer/writable flags after message-wide privilege promotion."""
        is_signer = message.is_signer(account_index)
        is_writable = (
            account_index < writable_signed
            if is_signer
            else account_index < writable_unsigned_end
        )
        return AccountMeta(keys[account_index], is_signer, is_writable)

    return Instruction(
        keys[compiled.program_id_index],
        bytes(compiled.data),
        [account_meta(i) for i in bytes(compiled.accounts)],
    )


def test_sol_action_availability_uses_live_costs_and_reserve_floors() -> None:
    """Use live costs above each floor and only honor explicit sponsorship."""
    components = SolBalanceComponents(10_000_000, 5_000_000)
    existing = SolBalanceAvailability.from_costs(
        components, SolActionCosts(5_000, 0, False, False)
    )
    assert existing.reserve_lamports == 1_000_000
    assert existing.spendable_lamports == 14_000_000

    account_creation = SolBalanceAvailability.from_costs(
        components, SolActionCosts(1_000_000, 3_000_000, True, False)
    )
    assert account_creation.reserve_lamports == 4_000_000
    sponsored = SolBalanceAvailability.from_costs(
        components, SolActionCosts(20_000_000, 20_000_000, True, True)
    )
    assert sponsored.reserve_lamports == 0

    with pytest.raises(SdkError, match="transaction reserve"):
        SolBalanceAvailability.from_costs(
            SolBalanceComponents(999_999, 10_000_000),
            SolActionCosts(5_000, 0, False, False),
        )


@pytest.mark.parametrize(
    ("fee_lamports", "rent_lamports"),
    [(-1, 0), (2**64, 0), (2**64 - 1, 1)],
)
def test_sol_action_availability_rejects_invalid_costs(
    fee_lamports: int, rent_lamports: int
) -> None:
    """Reject negative, overflowing, and sum-overflowing transaction costs."""
    with pytest.raises(SdkError, match="u64"):
        SolBalanceAvailability.from_costs(
            SolBalanceComponents(10_000_000, 5_000_000),
            SolActionCosts(fee_lamports, rent_lamports, False, True),
        )


def test_sol_action_availability_rejects_displayed_u64_overflow() -> None:
    """Reject an aggregate amount that no Solana instruction can represent."""
    with pytest.raises(SdkError, match="displayed SOL exceeds"):
        SolBalanceAvailability.from_costs(
            SolBalanceComponents(2**64 - 1, 1),
            SolActionCosts(0, 0, False, True),
        )


@pytest.mark.parametrize(
    "components",
    [SolBalanceComponents(-1, 0), SolBalanceComponents(0, 2**64)],
)
def test_sol_action_availability_rejects_invalid_components(
    components: SolBalanceComponents,
) -> None:
    """Reject negative or overflowing authoritative balance components."""
    with pytest.raises(SdkError, match="non-negative u64"):
        SolBalanceAvailability.from_costs(
            components,
            SolActionCosts(0, 0, False, True),
        )


def test_unwrap_all_availability_reserves_only_the_exact_live_fee() -> None:
    """Preserve components after validating the complete unwrap cost tuple."""
    components = SolBalanceComponents(5_000, 500_000_000)
    costs = SolActionCosts(5_000, 0, False, False)
    availability = SolBalanceAvailability.from_unwrap_all_costs(components, costs)

    assert availability.components is components
    assert availability.displayed_lamports == 500_005_000
    assert availability.reserve_lamports == 5_000
    assert availability.spendable_lamports == 500_000_000


def test_unwrap_all_availability_fails_closed_on_fee_and_display_errors() -> None:
    """Require native fee funding and checked common-u64 displayed arithmetic."""
    with pytest.raises(SdkError, match="unwrap-all transaction fee"):
        SolBalanceAvailability.from_unwrap_all_costs(
            SolBalanceComponents(4_999, 500_000_000),
            SolActionCosts(5_000, 0, False, False),
        )
    with pytest.raises(SdkError, match="displayed SOL exceeds"):
        SolBalanceAvailability.from_unwrap_all_costs(
            SolBalanceComponents(2**64 - 1, 1),
            SolActionCosts(0, 0, False, False),
        )
    with pytest.raises(SdkError, match="non-negative u64"):
        SolBalanceAvailability.from_unwrap_all_costs(
            SolBalanceComponents(5_000, 0),
            SolActionCosts(True, 0, False, False),
        )


@pytest.mark.parametrize(
    ("costs", "message"),
    [
        (SolActionCosts(5_000, 1, False, False), "zero upfront rent"),
        (SolActionCosts(5_000, 0, True, False), "must not create"),
        (SolActionCosts(5_000, 0, False, True), "must be unsponsored"),
    ],
)
def test_unwrap_all_availability_rejects_non_close_cost_tuples(
    costs: SolActionCosts, message: str
) -> None:
    """Reject rent, account creation, or sponsorship before fee-only math."""
    with pytest.raises(SdkError, match=message):
        SolBalanceAvailability.from_unwrap_all_costs(
            SolBalanceComponents(10_000, 500_000_000), costs
        )


def test_temporary_native_withdraw_seed_and_address_match_other_sdks() -> None:
    """Pin byte-exact seed and legacy-token address derivation across SDKs."""
    wallet = Pubkey(bytes([1]) * 32)
    recipient = Pubkey(bytes([2]) * 32)
    seed = native_withdraw_seed(
        Hash.default(), wallet, recipient, 0x0102_0304_0506_0708, 7
    )
    assert seed == "4dce744c636478f024df5aefd987f933"
    assert (
        str(Pubkey.create_with_seed(wallet, seed, TOKEN_PROGRAM_ID))
        == "71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6"
    )


@pytest.mark.asyncio
async def test_wrap_sol_reuses_canonical_account_with_exact_instruction_and_costs() -> (
    None
):
    """Transfer and sync an exact amount after matching live canonical state."""
    positions, state, _rpc, wallet = planning_harness(fees=[7_500])
    plan = await positions.plan_wrap_sol(250_000_000, state)
    canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)

    assert plan.kind is SolActionKind.WRAP
    assert len(plan.transaction.message.instructions) == 2
    transfer_params = decode_transfer(compiled_instruction(plan.transaction, 0))
    assert transfer_params["from_pubkey"] == wallet
    assert transfer_params["to_pubkey"] == canonical
    assert transfer_params["lamports"] == 250_000_000
    sync = compiled_instruction(plan.transaction, 1)
    assert decode_sync_native(sync).program_id == TOKEN_PROGRAM_ID
    assert sync.accounts[0].pubkey == canonical
    assert plan.costs == SolActionCosts(7_500, 0, False, False)
    assert plan.availability.reserve_lamports == 1_000_000
    assert plan.expected_delta.native_lamports == -250_007_500
    assert plan.expected_delta.canonical_wsol_lamports == 250_000_000


@pytest.mark.asyncio
async def test_wrap_sol_strictly_creates_missing_account_before_transfer_and_sync() -> (
    None
):
    """Create the wallet's exact Tokenkeg ATA and include live rent in delta."""
    positions, state, _rpc, wallet = planning_harness(
        native="1.000000000",
        wrapped="0.000000000",
        canonical_exists=False,
        fees=[5_000],
    )
    plan = await positions.plan_wrap_sol(500_000_000, state)
    canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)

    assert len(plan.transaction.message.instructions) == 3
    create = compiled_instruction(plan.transaction, 0)
    assert create.program_id == ASSOCIATED_TOKEN_PROGRAM_ID
    assert create.data == b""
    assert [
        (meta.pubkey, meta.is_signer, meta.is_writable) for meta in create.accounts
    ] == [
        (wallet, True, True),
        (canonical, False, True),
        (wallet, True, True),
        (WRAPPED_SOL_MINT, False, False),
        (SYSTEM_PROGRAM_ID, False, False),
        (TOKEN_PROGRAM_ID, False, False),
    ]
    transfer_params = decode_transfer(compiled_instruction(plan.transaction, 1))
    assert transfer_params["to_pubkey"] == canonical
    assert transfer_params["lamports"] == 500_000_000
    assert compiled_instruction(plan.transaction, 2).accounts[0].pubkey == canonical
    assert plan.costs == SolActionCosts(5_000, 2_039_280, True, False)
    assert plan.availability.reserve_lamports == 3_500_000
    assert plan.expected_delta.native_lamports == -502_044_280
    assert plan.expected_delta.canonical_wsol_lamports == 500_000_000


@pytest.mark.asyncio
async def test_wrap_sol_uses_live_cost_above_the_ordinary_floor() -> None:
    """Reserve an exact high fee rather than truncating it to the configured floor."""
    positions, state, _rpc, _wallet = planning_harness(fees=[2_000_000])
    plan = await positions.plan_wrap_sol(1, state)

    assert plan.costs.fee_lamports == 2_000_000
    assert plan.availability.reserve_lamports == 2_000_000
    assert plan.expected_delta.native_lamports == -2_000_001


@pytest.mark.asyncio
@pytest.mark.parametrize("amount", [-1, 0, 1.5, True, 2**64])
async def test_wrap_sol_rejects_invalid_amounts_before_rpc(amount: object) -> None:
    """Reject non-integer, non-positive, and overflowing exact wrap amounts."""
    positions, state, rpc, _wallet = planning_harness()
    with pytest.raises(SdkError, match="integer|greater than zero|fit u64"):
        await positions.plan_wrap_sol(amount, state)  # type: ignore[arg-type]
    assert rpc.account_lookups == []
    assert rpc.fee_calls == 0


@pytest.mark.asyncio
async def test_wrap_sol_requires_native_amount_plus_reserve() -> None:
    """Do not let existing WSOL mask insufficient native conversion funding."""
    positions, state, _rpc, _wallet = planning_harness(
        native="0.250000000", wrapped="1.000000000"
    )
    with pytest.raises(SdkError, match="wrap amount and transaction reserve"):
        await positions.plan_wrap_sol(250_000_000, state)


@pytest.mark.asyncio
async def test_wrap_sol_rejects_amount_plus_reserve_u64_overflow() -> None:
    """Reject an otherwise valid amount when amount plus reserve exceeds u64."""
    positions, state, _rpc, _wallet = planning_harness(
        native="18446744073.709551615",
        wrapped="0.000000000",
        canonical_exists=False,
        canonical_token_amount_lamports=0,
    )
    with pytest.raises(SdkError, match="exceed u64"):
        await positions.plan_wrap_sol(2**64 - 1, state)


@pytest.mark.asyncio
async def test_wrap_sol_rejects_missing_invalid_and_stale_canonical_state() -> None:
    """Fail closed instead of wrapping against absent, invalid, or stale authority."""
    positions, state, _rpc, _wallet = planning_harness(canonical_exists=False)
    with pytest.raises(SdkError, match="positive but its account is unavailable"):
        await positions.plan_wrap_sol(1, state)

    positions, state, _rpc, _wallet = planning_harness(invalid_canonical_account=True)
    with pytest.raises(SdkError, match="canonical WSOL token account is invalid"):
        await positions.plan_wrap_sol(1, state)

    positions, state, _rpc, _wallet = planning_harness(
        canonical_token_amount_lamports=499_999_999
    )
    with pytest.raises(SdkError, match="does not match wallet balance state"):
        await positions.plan_wrap_sol(1, state)


@pytest.mark.asyncio
async def test_wrap_sol_rejects_unsynchronized_donated_lamports_before_fee() -> None:
    """Prevent SyncNative from silently increasing canonical WSOL beyond intent."""
    positions, state, rpc, wallet = planning_harness(
        canonical_account_lamports=503_039_280,
    )

    with pytest.raises(SdkError, match="unsynchronized excess lamports"):
        await positions.plan_wrap_sol(1, state)

    assert rpc.account_lookups == [
        get_associated_token_address(wallet, WRAPPED_SOL_MINT)
    ]
    assert rpc.fee_calls == 0


@pytest.mark.asyncio
async def test_wrap_sol_rejects_existing_canonical_u64_overflow_before_fee() -> None:
    """Reject a transfer that would overflow the synchronized token account."""
    token_lamports = 9_000_000_000_000_000
    positions, state, rpc, _wallet = planning_harness(
        native="18446744073.709551615",
        wrapped="9000000.000000000",
        canonical_token_amount_lamports=token_lamports,
        canonical_account_lamports=token_lamports + 2_039_280,
    )

    with pytest.raises(SdkError, match="canonical WSOL token or account u64 range"):
        await positions.plan_wrap_sol(2**64 - token_lamports, state)

    assert rpc.fee_calls == 0


@pytest.mark.asyncio
async def test_unwrap_all_accepts_unsynchronized_donation_and_credits_it() -> None:
    """Close returns token amount, rent, and donated excess to the same wallet."""
    positions, state, _rpc, wallet = planning_harness(
        native="0.000005000",
        canonical_account_lamports=503_039_280,
        fees=[5_000],
    )
    plan = await positions.plan_unwrap_wsol_all(state)
    canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)

    assert plan.kind is SolActionKind.UNWRAP_ALL
    assert len(plan.transaction.message.instructions) == 1
    close_instruction = compiled_instruction(plan.transaction, 0)
    close = decode_close_account(close_instruction)
    assert close.account == canonical
    assert close.dest == wallet
    assert close.owner == wallet
    assert [
        (meta.pubkey, meta.is_signer, meta.is_writable)
        for meta in close_instruction.accounts
    ] == [
        (canonical, False, True),
        (wallet, True, True),
        (wallet, True, True),
    ]
    assert plan.costs == SolActionCosts(5_000, 0, False, False)
    assert plan.availability.components == SolBalanceComponents(5_000, 500_000_000)
    assert plan.availability.displayed_lamports == 500_005_000
    assert plan.availability.reserve_lamports == 5_000
    assert plan.availability.spendable_lamports == 500_000_000
    assert plan.expected_delta.native_lamports == 503_034_280
    assert plan.expected_delta.canonical_wsol_lamports == -500_000_000


@pytest.mark.asyncio
async def test_unwrap_all_uses_exact_high_fee_without_the_ordinary_floor() -> None:
    """Interpret the factual zero-rent cost tuple through fee-only availability."""
    positions, state, _rpc, _wallet = planning_harness(
        native="0.002000000", fees=[2_000_000]
    )
    plan = await positions.plan_unwrap_wsol_all(state)

    assert plan.costs == SolActionCosts(2_000_000, 0, False, False)
    assert plan.availability.reserve_lamports == 2_000_000
    assert plan.expected_delta.native_lamports == 500_039_280


@pytest.mark.asyncio
async def test_unwrap_all_rejects_zero_missing_mismatched_and_invalid_accounts() -> (
    None
):
    """Require positive complete state and one matching valid live account."""
    positions, state, rpc, _wallet = planning_harness(
        wrapped="0.000000000",
        canonical_token_amount_lamports=0,
    )
    with pytest.raises(SdkError, match="greater than zero"):
        await positions.plan_unwrap_wsol_all(state)
    assert rpc.account_lookups == []

    positions, state, _rpc, _wallet = planning_harness(canonical_exists=False)
    with pytest.raises(SdkError, match="unavailable for unwrap-all"):
        await positions.plan_unwrap_wsol_all(state)

    positions, state, _rpc, _wallet = planning_harness(
        canonical_token_amount_lamports=500_000_001
    )
    with pytest.raises(SdkError, match="does not match wallet balance state"):
        await positions.plan_unwrap_wsol_all(state)

    positions, state, _rpc, _wallet = planning_harness(invalid_canonical_account=True)
    with pytest.raises(SdkError, match="canonical WSOL token account is invalid"):
        await positions.plan_unwrap_wsol_all(state)


@pytest.mark.asyncio
async def test_unwrap_all_requires_native_fee_and_checked_destination_balance() -> None:
    """Reject insufficient fee funds and a close return that would overflow native."""
    positions, state, _rpc, _wallet = planning_harness(
        native="0.000004999", fees=[5_000]
    )
    with pytest.raises(SdkError, match="unwrap-all transaction fee"):
        await positions.plan_unwrap_wsol_all(state)

    positions, state, _rpc, _wallet = planning_harness(
        native="18446744073.709551614",
        wrapped="0.000000001",
        fees=[0],
        canonical_token_amount_lamports=1,
        canonical_account_lamports=2_039_281,
    )
    with pytest.raises(SdkError, match="projected native SOL exceeds"):
        await positions.plan_unwrap_wsol_all(state)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "signing_kind", [SigningStrategyKind.WALLET_ADAPTER, SigningStrategyKind.PRIVY]
)
async def test_conversion_planners_reject_non_native_signers_before_rpc(
    signing_kind: SigningStrategyKind,
) -> None:
    """Keep explicit conversion native-only without tightening ordinary plans."""
    for action in ("wrap", "unwrap"):
        positions, state, rpc, _wallet = planning_harness(signing_kind=signing_kind)
        with pytest.raises(SdkError, match="native signing strategy"):
            if action == "wrap":
                await positions.plan_wrap_sol(1, state)
            else:
                await positions.plan_unwrap_wsol_all(state)
        assert rpc.account_lookups == []
        assert rpc.fee_calls == 0

    positions, state, rpc, _wallet = planning_harness(signing_kind=signing_kind)
    ordinary = await positions.plan_native_sol_withdrawal(
        Pubkey.new_unique(), 1, state, False
    )
    assert ordinary.kind is SolActionKind.NATIVE_WITHDRAW
    assert rpc.fee_calls == 1


@pytest.mark.asyncio
async def test_conversion_planners_reject_expired_and_wrong_native_wallets_before_rpc() -> (
    None
):
    """Reuse complete authenticated identity checks after native admission."""
    positions, state, rpc, _wallet = planning_harness(
        credential_expires_at=int(time.time()) - 1
    )
    with pytest.raises(SdkError, match="credentials have expired"):
        await positions.plan_wrap_sol(1, state)
    assert rpc.account_lookups == []

    positions, state, rpc, _wallet = planning_harness(signing_wallet_mismatch=True)
    with pytest.raises(SdkError, match="does not control authenticated wallet"):
        await positions.plan_unwrap_wsol_all(state)
    assert rpc.account_lookups == []

    positions, state, rpc, _wallet = planning_harness()
    state.wallet_address = str(Pubkey.new_unique())
    with pytest.raises(SdkError, match="does not match wallet balance state"):
        await positions.plan_wrap_sol(1, state)
    assert rpc.account_lookups == []


@pytest.mark.asyncio
async def test_conversion_planners_fail_closed_when_live_fee_is_unavailable() -> None:
    """Never synthesize a zero fee for either prepared conversion message."""
    for action in ("wrap", "unwrap"):
        positions, state, rpc, _wallet = planning_harness(fees=[None])
        with pytest.raises(SdkError, match="fee estimate is unavailable"):
            if action == "wrap":
                await positions.plan_wrap_sol(1, state)
            else:
                await positions.plan_unwrap_wsol_all(state)
        assert rpc.fee_calls == 1


@pytest.mark.asyncio
async def test_split_wraps_only_the_shortfall_before_deposit() -> None:
    """Consume canonical WSOL first and wrap only the exact split shortfall."""
    positions, state, _rpc, wallet = planning_harness(
        native="1.500000000", wrapped="0.200000000"
    )
    plan = await positions.plan_sol_split(market(), 500_000_000, state, False)

    assert plan.kind is SolActionKind.SPLIT
    assert len(plan.transaction.message.instructions) == 3
    transfer_params = decode_transfer(compiled_instruction(plan.transaction, 0))
    assert transfer_params["lamports"] == 300_000_000
    assert transfer_params["to_pubkey"] == get_associated_token_address(
        wallet, WRAPPED_SOL_MINT
    )
    assert decode_sync_native(compiled_instruction(plan.transaction, 1)).program_id == (
        TOKEN_PROGRAM_ID
    )
    assert plan.expected_delta.native_lamports == -300_005_000
    assert plan.expected_delta.canonical_wsol_lamports == -200_000_000


@pytest.mark.asyncio
async def test_split_creates_missing_canonical_account_and_reserves_rent() -> None:
    """Create a missing canonical ATA atomically and reserve its live rent."""
    positions, state, _rpc, _wallet = planning_harness(
        native="1.000000000",
        wrapped="0.000000000",
        canonical_exists=False,
    )
    plan = await positions.plan_sol_split(market(), 500_000_000, state, False)

    assert len(plan.transaction.message.instructions) == 4
    assert compiled_instruction(plan.transaction, 0).program_id == (
        ASSOCIATED_TOKEN_PROGRAM_ID
    )
    assert plan.availability.reserve_lamports == 3_500_000
    assert plan.expected_delta.native_lamports == -502_044_280
    assert plan.expected_delta.canonical_wsol_lamports == 0


@pytest.mark.asyncio
async def test_merge_and_redeem_keep_proceeds_in_the_canonical_account() -> None:
    """Retain receive-side proceeds without any canonical close instruction."""
    for action in ("merge", "redeem"):
        positions, state, _rpc, _wallet = planning_harness(
            native="1.000000000",
            wrapped="0.000000000",
            canonical_exists=False,
        )
        if action == "merge":
            plan = await positions.plan_sol_merge(market(), 250_000_000, state, False)
        else:
            plan = await positions.plan_sol_redeem(
                Pubkey.new_unique(), 250_000_000, 0, 2, state, False
            )

        assert len(plan.transaction.message.instructions) == 2
        assert all(
            compiled_instruction(plan.transaction, index).data != bytes([9])
            for index in range(2)
        )
        assert plan.expected_delta.native_lamports == -2_044_280
        assert plan.expected_delta.canonical_wsol_lamports == 250_000_000


@pytest.mark.asyncio
async def test_native_withdrawal_uses_direct_transfer_when_native_is_sufficient() -> (
    None
):
    """Prefer one native transfer when native SOL covers amount and reserve."""
    positions, state, _rpc, wallet = planning_harness(
        native="1.000000000", wrapped="1.000000000"
    )
    recipient = Pubkey.new_unique()
    plan = await positions.plan_native_sol_withdrawal(
        recipient, 500_000_000, state, False
    )

    assert plan.kind is SolActionKind.NATIVE_WITHDRAW
    assert len(plan.transaction.message.instructions) == 1
    transfer_params = decode_transfer(compiled_instruction(plan.transaction, 0))
    assert transfer_params["from_pubkey"] == wallet
    assert transfer_params["to_pubkey"] == recipient
    assert transfer_params["lamports"] == 500_000_000
    assert plan.expected_delta.native_lamports == -500_005_000
    assert plan.expected_delta.canonical_wsol_lamports == 0


@pytest.mark.asyncio
async def test_native_withdrawal_closes_only_a_seeded_temporary_account() -> None:
    """Convert only the shortfall and close the seeded account, not canonical WSOL."""
    direct_blockhash = Hash.new_unique()
    planned_blockhash = Hash.new_unique()
    replacement_blockhash = Hash.new_unique()
    positions, state, _rpc, wallet = planning_harness(
        native="0.010000000",
        wrapped="1.000000000",
        blockhashes=[direct_blockhash, planned_blockhash, replacement_blockhash],
    )
    recipient = Pubkey.new_unique()
    plan = await positions.plan_native_sol_withdrawal(
        recipient, 500_000_000, state, False
    )

    assert len(plan.transaction.message.instructions) == 5
    assert plan.transaction.message.recent_blockhash == planned_blockhash
    expected_seed = native_withdraw_seed(
        plan.transaction.message.recent_blockhash,
        wallet,
        recipient,
        500_000_000,
        0,
    )
    expected_temporary = Pubkey.create_with_seed(
        wallet, expected_seed, TOKEN_PROGRAM_ID
    )
    create_instruction = compiled_instruction(plan.transaction, 0)
    assert create_instruction.accounts[1].pubkey == expected_temporary
    token_transfer = decode_token_transfer(compiled_instruction(plan.transaction, 2))
    assert token_transfer.amount == 492_044_280
    assert token_transfer.source == get_associated_token_address(
        wallet, WRAPPED_SOL_MINT
    )
    close_params = decode_close_account(compiled_instruction(plan.transaction, 3))
    assert close_params.account != get_associated_token_address(
        wallet, WRAPPED_SOL_MINT
    )
    assert close_params.dest == wallet
    native_transfer = decode_transfer(compiled_instruction(plan.transaction, 4))
    assert native_transfer["to_pubkey"] == recipient
    assert native_transfer["lamports"] == 500_000_000
    assert plan.availability.reserve_lamports == 2_044_280
    assert plan.expected_delta.native_lamports == -7_960_720
    assert plan.expected_delta.canonical_wsol_lamports == -492_044_280


@pytest.mark.asyncio
async def test_every_ordinary_plan_scans_all_instructions_without_canonical_close() -> (
    None
):
    """Inspect every ordinary instruction, including split's final market call."""
    ordinary_plans = []

    positions, state, _rpc, wallet = planning_harness(
        native="1.500000000", wrapped="0.200000000"
    )
    split = await positions.plan_sol_split(market(), 500_000_000, state, False)
    ordinary_plans.append(("split", split, wallet))

    positions, state, _rpc, wallet = planning_harness()
    ordinary_plans.append(
        (
            "merge",
            await positions.plan_sol_merge(market(), 250_000_000, state, False),
            wallet,
        )
    )

    positions, state, _rpc, wallet = planning_harness()
    ordinary_plans.append(
        (
            "redeem",
            await positions.plan_sol_redeem(
                Pubkey.new_unique(), 250_000_000, 0, 2, state, False
            ),
            wallet,
        )
    )

    positions, state, _rpc, wallet = planning_harness()
    ordinary_plans.append(
        (
            "native-direct",
            await positions.plan_native_sol_withdrawal(
                Pubkey.new_unique(), 500_000_000, state, False
            ),
            wallet,
        )
    )

    positions, state, _rpc, wallet = planning_harness(
        native="0.010000000",
        wrapped="1.000000000",
        blockhashes=[Hash.new_unique(), Hash.new_unique(), Hash.new_unique()],
    )
    ordinary_plans.append(
        (
            "native-temporary",
            await positions.plan_native_sol_withdrawal(
                Pubkey.new_unique(), 500_000_000, state, False
            ),
            wallet,
        )
    )

    split_final = compiled_instruction(
        split.transaction, len(split.transaction.message.instructions) - 1
    )
    assert split_final.program_id == Pubkey.default()
    assert split_final.data != bytes([9])

    scanned = 0
    for name, plan, plan_wallet in ordinary_plans:
        canonical = get_associated_token_address(plan_wallet, WRAPPED_SOL_MINT)
        for index in range(len(plan.transaction.message.instructions)):
            instruction = compiled_instruction(plan.transaction, index)
            scanned += 1
            if (
                instruction.program_id == TOKEN_PROGRAM_ID
                and instruction.data == bytes([9])
            ):
                assert instruction.accounts[0].pubkey != canonical, name
    assert scanned == sum(
        len(plan.transaction.message.instructions)
        for _name, plan, _wallet in ordinary_plans
    )


@pytest.mark.asyncio
async def test_native_withdrawal_fails_after_eight_seed_collisions() -> None:
    """Bound collision probing so planning cannot loop on hostile account state."""
    positions, state, rpc, _wallet = planning_harness(
        native="0.010000000",
        wrapped="1.000000000",
        occupied_temporary_attempts=8,
    )
    with pytest.raises(SdkError, match="seed attempts are exhausted"):
        await positions.plan_native_sol_withdrawal(
            Pubkey.new_unique(), 500_000_000, state, False
        )
    assert len(rpc.account_lookups) == 9


@pytest.mark.asyncio
@pytest.mark.parametrize("amount", [-1, 0, 2**64])
async def test_sol_action_plans_reject_amounts_outside_u64_before_rpc(
    amount: int,
) -> None:
    """Reject invalid instruction amounts before any fee or account read."""
    positions, state, rpc, _wallet = planning_harness()
    with pytest.raises(SdkError, match="greater than zero|fit u64"):
        await positions.plan_native_sol_withdrawal(
            Pubkey.new_unique(), amount, state, False
        )
    assert rpc.fee_calls == 0

    assert rpc.account_lookups == []


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "fees",
    ([20_000_000, 0], [20_000_000, 15_000_000, 0]),
)
async def test_native_withdrawal_rejects_negative_temporary_transfer(
    fees: list[int],
) -> None:
    """Reject fee changes that would make either transfer calculation negative."""
    positions, state, _rpc, _wallet = planning_harness(native="0.110000000", fees=fees)

    with pytest.raises(SdkError, match="invalid temporary withdrawal requirement"):
        await positions.plan_native_sol_withdrawal(
            Pubkey.new_unique(), 100_000_000, state, False
        )


@pytest.mark.asyncio
async def test_sol_action_plans_fail_closed_on_identity_and_fee_errors() -> None:
    """Refuse stale identity and unavailable fee authority without guessing."""
    positions, state, rpc, _wallet = planning_harness()
    state.wallet_address = str(Pubkey.new_unique())
    with pytest.raises(SdkError, match="does not match wallet balance state"):
        await positions.plan_native_sol_withdrawal(Pubkey.new_unique(), 1, state, False)
    assert rpc.fee_calls == 0

    positions, state, rpc, _wallet = planning_harness(fees=[None])
    with pytest.raises(SdkError, match="fee estimate is unavailable"):
        await positions.plan_native_sol_withdrawal(Pubkey.new_unique(), 1, state, False)
    assert rpc.fee_calls == 1


@pytest.mark.asyncio
async def test_sol_action_plans_reject_a_mismatched_signing_wallet_before_rpc() -> None:
    """Reject a signer that cannot control the authenticated planning wallet."""
    positions, state, rpc, _wallet = planning_harness(signing_wallet_mismatch=True)

    with pytest.raises(
        SdkError, match="signing strategy does not control authenticated wallet"
    ):
        await positions.plan_native_sol_withdrawal(Pubkey.new_unique(), 1, state, False)
    assert rpc.fee_calls == 0
    assert rpc.account_lookups == []


@pytest.mark.asyncio
async def test_planners_reject_unsupported_sponsorship_and_invalid_redeem_outcomes() -> (
    None
):
    """Reject unsupported or invalid intent before touching chain authority."""
    positions, state, rpc, _wallet = planning_harness()

    with pytest.raises(
        SdkError, match="sponsored SOL action planning is not supported"
    ):
        await positions.plan_sol_split(market(), 1, state, True)
    with pytest.raises(InvalidOutcomeIndexError):
        await positions.plan_sol_redeem(Pubkey.new_unique(), 1, 2, 2, state, False)
    assert rpc.account_lookups == []


@pytest.mark.asyncio
async def test_planner_rejects_an_occupied_invalid_canonical_account() -> None:
    """Reject an occupied canonical address that is not a valid WSOL token account."""
    positions, state, _rpc, _wallet = planning_harness(invalid_canonical_account=True)

    with pytest.raises(SdkError, match="canonical WSOL token account is invalid"):
        await positions.plan_sol_split(market(), 1, state, False)
