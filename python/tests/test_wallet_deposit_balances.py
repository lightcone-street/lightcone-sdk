import json
import time

import pytest
from solders.hash import Hash
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.system_program import decode_transfer
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
from lightcone_sdk.shared.signing import SigningStrategy
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
        occupied_temporary_attempts: int = 0,
        fees: list[int | None] | None = None,
        rent_lamports: int = 2_039_280,
        blockhashes: list[Hash] | None = None,
    ) -> None:
        """Configure canonical presence, collision count, and ordered live values."""
        self.canonical = get_associated_token_address(wallet, WRAPPED_SOL_MINT)
        self.canonical_exists = canonical_exists
        self.occupied_temporary_attempts = occupied_temporary_attempts
        self.fees = list(fees or [5_000])
        self.rent_lamports = rent_lamports
        self.blockhashes = list(blockhashes or [Hash.default()])
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


def planning_harness(
    *,
    native: str = "2.000000000",
    wrapped: str = "0.500000000",
    credentials: AuthCredentials | None = None,
    canonical_exists: bool = True,
    occupied_temporary_attempts: int = 0,
    fees: list[int | None] | None = None,
    blockhashes: list[Hash] | None = None,
    signing_wallet_mismatch: bool = False,
) -> tuple[Positions, WalletDepositBalancesState, FakeRpc, Pubkey]:
    """Build complete wallet state and a deterministic planner dependency graph."""
    keypair = Keypair()
    wallet = keypair.pubkey()
    if credentials is None:
        credentials = AuthCredentials(
            user_id="user-a",
            wallet_address=str(wallet),
            expires_at=int(time.time()) + 60,
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
    rpc = FakeRpc(
        wallet,
        canonical_exists=canonical_exists,
        occupied_temporary_attempts=occupied_temporary_attempts,
        fees=fees,
        blockhashes=blockhashes,
    )
    signing_keypair = Keypair() if signing_wallet_mismatch else keypair
    client = FakePlanningClient(
        credentials, rpc, SigningStrategy.native(signing_keypair)
    )
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
    """Expand one compiled instruction so SPL/system decoders can inspect it."""
    message = transaction.message
    compiled = message.instructions[index]
    keys = message.account_keys
    return Instruction(
        keys[compiled.program_id_index],
        bytes(compiled.data),
        [AccountMeta(keys[i], False, True) for i in bytes(compiled.accounts)],
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
