"""Program-client known answers for the instruction builders.

Every vector below is the account list and data that the program team's
``lightcone-client`` produces at the nonce-free program commit (f1092ae7, ABI
unchanged from 9702f231), pinned by the Rust SDK's instruction tests and
reproduced byte for byte by the TypeScript SDK. Wallets are
``Keypair.from_seed(bytes([seed]) * 32)``; amounts are raw atoms.
"""

from dataclasses import replace

import pytest
from nacl.signing import SigningKey
from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.program.constants import MAX_MAKERS
from lightcone_sdk.program.errors import (
    DepositMintMismatchError,
    InvalidOrderbookError,
    InvalidOutcomeCountError,
    InvalidSideError,
    MissingFieldError,
    SerializationError,
    TooManyMakersError,
)
from lightcone_sdk.program.instructions import (
    build_cancel_order_instruction,
    build_close_orderbook_instruction,
    build_close_position_token_accounts_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_deposit_to_global_instruction,
    build_global_to_market_deposit_instruction,
    build_init_position_tokens_instruction,
    build_match_orders_multi_instruction,
    build_mint_complete_set_instruction,
)
from lightcone_sdk.program.orders import hash_order_hex
from lightcone_sdk.program.pda import (
    get_conditional_mint_pda,
    get_orderbook_pda,
    get_position_pda,
)
from lightcone_sdk.program.types import (
    CloseOrderbookParams,
    ClosePositionTokenAccountsParams,
    MakerFill,
    OrderSide,
    SignedOrder,
)

Account = tuple[str, bool, bool]

CLIENT_PROGRAM_ID = Pubkey.from_string("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai")


def wallet(seed: int) -> Keypair:
    """Deterministic Ed25519 wallet, so order signatures are reproducible."""
    return Keypair.from_seed(bytes([seed]) * 32)


MARKET = Pubkey.from_bytes(bytes([0x4D]) * 32)
COLLATERAL_A = Pubkey.from_bytes(bytes([0xA1]) * 32)
COLLATERAL_B = Pubkey.from_bytes(bytes([0x3C]) * 32)
OPERATOR = wallet(1).pubkey()
FEE_RECEIVER = wallet(2).pubkey()
MANAGER = wallet(6).pubkey()
USER = wallet(42).pubkey()


def mint(collateral: Pubkey, outcome: int) -> Pubkey:
    """The conditional mint for one collateral and outcome."""
    return get_conditional_mint_pda(MARKET, collateral, outcome, CLIENT_PROGRAM_ID)[0]


def signed_by(
    seed: int,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    *,
    salt: int,
    side: OrderSide,
    amount_in: int,
    amount_out: int,
    expiration: int,
) -> SignedOrder:
    """Make wallet ``seed`` the maker of an order on one pair and sign its order-id hex."""
    order = SignedOrder(
        salt=salt,
        maker=wallet(seed).pubkey(),
        market=MARKET,
        base_mint=base_mint,
        quote_mint=quote_mint,
        side=side,
        amount_in=amount_in,
        amount_out=amount_out,
        expiration=expiration,
    )
    message = hash_order_hex(order).encode("ascii")
    order.signature = SigningKey(bytes([seed]) * 32).sign(message).signature
    return order


def assert_ix(ix: Instruction, accounts: list[Account], data_hex: str) -> None:
    """Assert the program id, every account meta in order, and the data bytes."""
    assert ix.program_id == CLIENT_PROGRAM_ID
    assert [
        (str(meta.pubkey), meta.is_signer, meta.is_writable) for meta in ix.accounts
    ] == accounts
    assert bytes(ix.data).hex() == data_hex


MINT_COMPLETE_SET_ACCOUNTS = [
    ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", True, True),  # user
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", False, False),  # collateral A
    ("CcJ9vwR45b6DNPChMaS8JG3ua6VhyrGU1zAmSvvkhaRD", False, True),  # vault A
    (
        "FCvj66xiYGJ6SMsZ91UUhN4xvS4Hu7ouRdwzMeECKWjL",
        False,
        True,
    ),  # user collateral A ATA
    ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", False, True),  # user position
    ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", False, False),  # mint authority
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
    ("11111111111111111111111111111111", False, False),  # system program
    ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", False, True),  # mint A0
    ("C2mDQ76H3X73j8sRj8QtZF9RXSyN4VUdJKvGQ6DTKk76", False, True),  # position A0 ATA
    ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", False, True),  # mint A1
    ("5jgjy7yUFwQUcnDMDXKR3yL9wcGJQdDhE9L3SQ9uCH1n", False, True),  # position A1 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

MINT_COMPLETE_SET_DATA = "03141a99be1c000000"

CLOSE_ORDERBOOK_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # operator
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", False, True),  # orderbook
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

CLOSE_ORDERBOOK_DATA = "1b"

DEPOSIT_TO_GLOBAL_ACCOUNTS = [
    ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", True, True),  # user
    ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", False, False),  # GDT A
    ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", False, False),  # collateral A
    (
        "3MWB38PADzqKzPnVTL3Z8YTwyLHpTmKaF3Aj4AU4j8jJ",
        False,
        True,
    ),  # user global deposit A
    (
        "FCvj66xiYGJ6SMsZ91UUhN4xvS4Hu7ouRdwzMeECKWjL",
        False,
        True,
    ),  # user collateral A ATA
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("11111111111111111111111111111111", False, False),  # system program
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

DEPOSIT_TO_GLOBAL_DATA = "11feffffffffffffff"

GLOBAL_TO_MARKET_ACCOUNTS = [
    ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", True, True),  # user
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", False, False),  # collateral B
    ("9Ni78X3DJWYiXpymPmA1N8hVdiFPCetaeYsoxaxuF3vz", False, True),  # vault B
    ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", False, False),  # GDT B
    (
        "EBijBMb6kV1e7jNpjCqgQuAQgX9oefqTBTHu4M4qtj1d",
        False,
        True,
    ),  # user global deposit B
    ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", False, True),  # user position
    ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", False, False),  # mint authority
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
    ("11111111111111111111111111111111", False, False),  # system program
    ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", False, True),  # mint B0
    ("FZoibJ9Bek51d82eKoryEo6GuW9nmYaPicZym3RawkXC", False, True),  # position B0 ATA
    ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", False, True),  # mint B1
    ("7jjbtKwm4p3gcTT2NYmdnNtfw5vPRRvg9B7nKz19y5g1", False, True),  # position B1 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

GLOBAL_TO_MARKET_DATA = "1220a1070000000000"

INIT_POSITION_TOKENS_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # payer
    ("2iXtA8oeZqUU5pofxK971TCEvFGfems2AcDRaZHKD2pQ", False, False),  # user
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", False, True),  # user position
    ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", False, False),  # mint authority
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
    ("11111111111111111111111111111111", False, False),  # system program
    ("5KovAGoer61Vvo1Uv7sod2PpdATt74wUm7ezjKsLpKeF", False, False),  # collateral G0
    ("FPaub4kYnsuF2nhEzkGrzruZwGU7qKQGAcSKaisjsd31", False, False),  # vault G0
    ("G8KpaNysbSRbPd2oKtuwnqpaz9yNhbDCtqXqdaLfPNZC", False, False),  # GDT G0
    ("7xiscy1E21eAJN9B8jL8KcJL5vSBEFoYfBowvAUBxLn5", False, False),  # mint G00
    ("Ap7pkFebmQzwJNuhtce4ps7yqoq2E2RPy3GxAfqZXdqP", False, True),  # position G00 ATA
    ("DJgBqQz9rCFoztQUBRUv5iXYUahaQfN8Rwd6PcUf8FqV", False, False),  # mint G01
    ("63GjdPDfRJWzbbxnU1nXXEfZLq2DEotUkxLC735FMZYC", False, True),  # position G01 ATA
    ("5PjDJaGfSPJj4tFzMRCiuuAasKg5n8dJKXKenhuwZexx", False, False),  # collateral G1
    ("3vmGXu4dC4zTJQybJsBfpcxqqTcqEUM5hHN5Nr7w46yj", False, False),  # vault G1
    ("2bzkpDhvuTJxZCPgmqwwrx3uezdLZEidQPdpLPeqYvoB", False, False),  # GDT G1
    ("9KH9qQqJ3Poka7oKTyd2ejDwYfjk5U5nzkfjjNxM2BaZ", False, False),  # mint G10
    ("DDPLSXHzmcSEKyS5HAzQZtKZkmC2vg6cvYATtRuBePZ9", False, True),  # position G10 ATA
    ("6UMX23ZUbMDeH3S9FHW3thYPvTowzMDUrQsVsjb54zxF", False, False),  # mint G11
    ("Ain499vM3dfWTeRaqNdpr5dxfXPiBXAL4VPVWSj4wSSq", False, True),  # position G11 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

INIT_POSITION_TOKENS_DATA = "1302"

CLOSE_POSITION_TOKEN_ACCOUNTS_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # operator
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("4CzHZWutNwxxUyV3TigWtWZhBqmRibWuzZqYNvAwYV1p", False, False),  # user position
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("5KovAGoer61Vvo1Uv7sod2PpdATt74wUm7ezjKsLpKeF", False, False),  # collateral G0
    ("7xiscy1E21eAJN9B8jL8KcJL5vSBEFoYfBowvAUBxLn5", False, False),  # mint G00
    ("Ap7pkFebmQzwJNuhtce4ps7yqoq2E2RPy3GxAfqZXdqP", False, True),  # position G00 ATA
    ("DJgBqQz9rCFoztQUBRUv5iXYUahaQfN8Rwd6PcUf8FqV", False, False),  # mint G01
    ("63GjdPDfRJWzbbxnU1nXXEfZLq2DEotUkxLC735FMZYC", False, True),  # position G01 ATA
    ("5PjDJaGfSPJj4tFzMRCiuuAasKg5n8dJKXKenhuwZexx", False, False),  # collateral G1
    ("9KH9qQqJ3Poka7oKTyd2ejDwYfjk5U5nzkfjjNxM2BaZ", False, False),  # mint G10
    ("DDPLSXHzmcSEKyS5HAzQZtKZkmC2vg6cvYATtRuBePZ9", False, True),  # position G10 ATA
    ("6UMX23ZUbMDeH3S9FHW3thYPvTowzMDUrQsVsjb54zxF", False, False),  # mint G11
    ("Ain499vM3dfWTeRaqNdpr5dxfXPiBXAL4VPVWSj4wSSq", False, True),  # position G11 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

CLOSE_POSITION_TOKEN_ACCOUNTS_DATA = "19"

CANCEL_ORDER_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # operator
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("CZyZ59NJHBRNoDKfndnNggqYHGbZ3xdDF75ikuHHJxDw", False, True),  # order status
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

CANCEL_ORDER_DATA = (
    "0576c704f365c47163d7443660ada391788ffd280a9c1e0d5a26af5394f25ad0"
    "4308070605040302015e212c0980e4b39fc09721134aa02109374edfd260c0d3"
    "d03cb501c8d65457a94d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d4d"
    "4d4d4d4d4d4d4d4d4d4d87e7c084828730227139c14226f040f48038ba06198b"
    "fb0ffeb9514acaea02aa599aca0ddeca9f522c30f07b7c0ba0a4a46c37c10c83"
    "73b27ffdfbba4af8280040e2010000000000f1fb09000000000000b33f710000"
    "000058fc3a9edb7464fb467a78e4603ae1238953860e55c7ed32f9ba9aa98afe"
    "a114578bdeb6f4c35f3b55138f41f6059918a267df3348a9576fb668cd943653"
    "0506"
)

MATCH_ORDERS_MULTI_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # operator
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("AcefzzBzYiYcLKmg9ktNNyBnPSxagC9JRNMweSN1kYF5", False, False),  # orderbook
    ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", False, False),  # GDT A
    ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", False, False),  # GDT B
    ("D6hX8jPNF1a2ZJApa9duWbGMneXN5XB5wATXTdj5U5Ja", False, True),  # taker status
    ("FBTMbey9FRyKTJ8bzGK8wokrwyVXjdnnaaBAiAtHTJCm", False, False),  # taker position
    ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", False, False),  # mint A0 (base)
    ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", False, False),  # mint B0 (quote)
    ("BKUo9SY9CjBD3WycwT96RPHzsqriXsgwZu4JZ31KnKpQ", False, True),  # taker A0 ATA
    ("BJTYzqmaZKc8U91F8nfqwfiFb6xGCToLii3mGFh9xbkL", False, True),  # taker B0 ATA
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    ("11111111111111111111111111111111", False, False),  # system program
    (
        "6DwXxNbBGFceSZbin2oLSP1DnPSXVD4er4U8ZBVW9Pxq",
        False,
        True,
    ),  # fee receiver B0 ATA
    ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", False, False),  # fee receiver
    ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
    ("AswUFYjEDQi4cXqmde8M1H4cZRD4BnmvKfbQoTqFBLB2", False, False),  # maker 0 position
    ("7tJKnMHYLKz7sk3xyDh41PQZbTV7vmSchtMZJAE7G81j", False, True),  # maker 0 A0 ATA
    ("4nV6kAnVDp5xLzzcyqFxpiWknSuF5YFZCtx7kZrr4p9R", False, True),  # maker 0 B0 ATA
    ("HbPnPSmp428fK7dnAtKwDdLoFu6mpQdHrz5b85Zyj2Em", False, True),  # maker 1 status
    ("DagzdYGfpRUgMmsnYZbzHPea9RkPjqJvj41mSg1Erx7w", False, False),  # maker 1 position
    ("8TAgt4BvP2aMFJ5T5or4dxgPmHNuGLx2c5HqSbg1U9vc", False, True),  # maker 1 A0 ATA
    ("EFiBM1hPrxAdfzKjZMhiGQDqfHeshPqcN38oiFfD6Twv", False, True),  # maker 1 B0 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

MATCH_ORDERS_MULTI_DATA = (
    "0d01000000efbeadde007800000000000000c800000000000000000000000000"
    "000065e302ae869d94a86e237bfb06dfa084adefa4f461ad90dce13c12d778ac"
    "2c26c480c307132f45a10ad75cdef22c8206d76397a3c0e737195d720682956b"
    "2009020100ffffffffffffffff0164000000000000003c0000000000000000b3"
    "3f71000000009b29d9b4d5fbcca95d70100554932d394e3136a883dfd71afe77"
    "d7583ffe7b6c0846996b6082b1426b937eaf07899db41dd1c9b01c8bc180aed3"
    "cf7bf0fb770164000000000000003c0000000000000007000000000000000196"
    "000000000000005a00000000000000ffffffffffffff7f3f32e60f1428f7eff0"
    "611c334bdd0315987680e864ec6c8c0b5622fcac9cd535e651111c8805b5bdc6"
    "c6c128de2f8c25fc13d9f8ad5e86e36117fc971b17740f64000000000000003c"
    "00000000000000"
)

DEPOSIT_AND_SWAP_ACCOUNTS = [
    ("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9", True, True),  # operator
    ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
    ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
    ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", False, False),  # orderbook
    ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", False, False),  # GDT B
    ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", False, False),  # GDT A
    ("HwerbZ3uDMyHrs1xe1gUfKBHDbmqp4iMcBzgGDNgKPZG", False, False),  # mint authority
    ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
    (
        "4yzmLNrFRbmMUH9g5HaraxzLHq2CLbh1BWHYcGfesJFL",
        False,
        True,
    ),  # fee receiver B1 ATA
    ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", False, False),  # fee receiver
    ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
    ("Gwrh4QtYMroN5j7iuBj8qJ2isAqnM2FRbsw23JaxD3BJ", False, False),  # taker position
    ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", False, False),  # mint A1 (base)
    ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", False, False),  # mint B1 (quote)
    ("CgaNt6iMtiNk6kCL71sJDDPB3GLFY2b4xNN54KKK97jE", False, True),  # taker B1 ATA
    ("FtcEGDzZsu5V3ZoMGiRJf8VscXuWreDoSCFxHRBCc6Xd", False, True),  # taker A1 ATA
    ("11111111111111111111111111111111", False, False),  # system program
    ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", False, False),  # collateral A
    ("CcJ9vwR45b6DNPChMaS8JG3ua6VhyrGU1zAmSvvkhaRD", False, True),  # vault A
    ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", False, False),  # GDT A
    (
        "G39vUBiTQfZeh61obxAoD6ThrwjANLyVsopaYy45T9Bd",
        False,
        True,
    ),  # taker global deposit A
    ("6Deb7C6szQAGzATpea8mwdk98yKLShbep7HQaKeZ4sVB", False, True),  # mint A0
    ("9yFYW3w2B2RyvHgh7KQf7HHqqX3rD2txp32WSwsbB7pW", False, True),  # taker A0 ATA
    ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", False, True),  # mint A1
    ("FtcEGDzZsu5V3ZoMGiRJf8VscXuWreDoSCFxHRBCc6Xd", False, True),  # taker A1 ATA
    ("68cXjdwiAAhMg1i8oAopj6J6j7xZZ2NQi7cES74HUujW", False, True),  # maker status
    ("FHyZwUs9jJNaXMzvyWZG6gxhiRqVE4CbxRb92RWHfuxR", False, False),  # maker position
    ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", False, False),  # collateral B
    ("9Ni78X3DJWYiXpymPmA1N8hVdiFPCetaeYsoxaxuF3vz", False, True),  # vault B
    ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", False, False),  # GDT B
    (
        "DbQxKggBn8RTGCFTqTkn3qYcMPqEKafwpcitUEZXvVaD",
        False,
        True,
    ),  # maker global deposit B
    ("CTyZDSYMyGRFE2XoFxproKZqrRe6RyVSghupF1dznVmh", False, True),  # mint B0
    ("J1NHgXSKBU2wPDptRDnKT2ct163kDxqKAn5rcED6caB3", False, True),  # maker B0 ATA
    ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", False, True),  # mint B1
    ("EJKrAAFTCgHsPq3nXLL12Bd8xVDy2P6oN4rEpAfMSuwL", False, True),  # maker B1 ATA
    ("EJKrAAFTCgHsPq3nXLL12Bd8xVDy2P6oN4rEpAfMSuwL", False, True),  # maker B1 ATA
    ("Fzq9pPAncudmCZsYRQdKsQNtnUFooQwDTeLb7qfQTMfb", False, True),  # maker A1 ATA
    ("8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8", False, False),  # event authority
    ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
]

DEPOSIT_AND_SWAP_DATA = (
    "1400000000000000000164000000000000003c00000000000000000000000000"
    "000076d7c446df05eaffbaddfee27a939ab11cd999afc57b80434bc749388567"
    "c019677e9ac826ca8dd1499da1a397e90a4f74c78fb6f4620c1d3c78453052f2"
    "8d0101008001801817161514131211005a000000000000009600000000000000"
    "00f153650000000065743720dc9ef272128db330653a217f25daed211354835c"
    "0d806cc78f5b539c7e89332fb9dc1c93b201dbfbefbb50cf900618a24a9444a7"
    "1c26ff74359b14043c000000000000006400000000000000"
)


def create_orderbook_accounts(fee_receiver_quote_ata: str) -> list[Account]:
    return [
        ("AKkzLhjhyFtM9j7WAhbaqYpFe49cXeJBg2kzLRC2PnNa", True, True),  # manager
        ("6Ckm2BrnXxsSjyG5b17kQQRjoECVrts92RKXVGT8XeqS", False, False),  # market
        ("9Vm6zp8LJ7BLKA5WXMCrVp1XtPcucDxGSniwcibC4qnY", False, False),  # mint B1
        ("DxkG46NfTmZdeLf88KacJwSFeEo8jeta4pQLGvstaKsa", False, False),  # mint A1
        ("E11bpYzs4qWFn7yiN2rqwnhCidnUFvwxvDmAgwWaYznw", False, True),  # orderbook
        ("7GHv8nJvVXunMXBpQF5t1EbDJtfLxKqXAjLCAtfrzgmf", False, False),  # GDT B
        ("GjYEnbey6TFjN6wSf6ZVWzPSykh9GPVBAo26oiXC7KhY", False, False),  # GDT A
        ("3QpkVHKRzdzj2uXdSH6TYXBYfgd4cwKK6Dmx8V78gx37", False, False),  # exchange
        ("11111111111111111111111111111111", False, False),  # system program
        ("548jb4wcUtpbNS1TAva8TXJmeXd5QpCCXUzNVogwpzMR", False, False),  # collateral B
        ("Bswb3UyeD1pUTaGiE6WvqwFpJZsQSEY1xhJePCDTHdvp", False, False),  # collateral A
        ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", False, False),  # token program
        ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", False, False),  # ATA program
        ("9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu", False, False),  # fee receiver
        (fee_receiver_quote_ata, False, True),
        (
            "8VqAxBFSt2PzRu2VjitjGZhWFaKKkjkd3kZpb3jtSkN8",
            False,
            False,
        ),  # event authority
        ("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai", False, False),  # program
    ]


# Fee receiver ATAs for quote mints B1 and A1.
CREATE_ORDERBOOK_BASE_A1_ATA = "4yzmLNrFRbmMUH9g5HaraxzLHq2CLbh1BWHYcGfesJFL"
CREATE_ORDERBOOK_BASE_B1_ATA = "73zFJD8Vtz2ash9VDypWXgnToNE5TH2fuT7CygK3ZY5T"


def match_inputs() -> dict:
    """Two-maker MatchOrdersMulti on the A0/B0 book (base mint sorts first).

    The BUY taker keeps its status account, maker 0 fully fills its base and
    omits its status (mask bit 0), and maker 1 fills partially. Each maker
    gives 100 base atoms for 60 quote atoms.
    """
    base_mint = mint(COLLATERAL_A, 0)
    quote_mint = mint(COLLATERAL_B, 0)
    return {
        "operator": OPERATOR,
        "market": MARKET,
        "base_mint": base_mint,
        "quote_mint": quote_mint,
        "fee_receiver": FEE_RECEIVER,
        "taker_order": signed_by(
            100,
            base_mint,
            quote_mint,
            salt=0xDEADBEEF00000001,
            side=OrderSide.BID,
            amount_in=120,
            amount_out=200,
            expiration=0,
        ),
        "maker_orders": [
            signed_by(
                10,
                base_mint,
                quote_mint,
                salt=(1 << 64) - 1,
                side=OrderSide.ASK,
                amount_in=100,
                amount_out=60,
                expiration=1_900_000_000,
            ),
            signed_by(
                11,
                base_mint,
                quote_mint,
                salt=7,
                side=OrderSide.ASK,
                amount_in=150,
                amount_out=90,
                expiration=(1 << 63) - 1,
            ),
        ],
        "maker_fill_amounts": [100, 100],
        "taker_fill_amounts": [60, 60],
        "full_fill_bitmask": 0x0001,
        "program_id": CLIENT_PROGRAM_ID,
        "base_deposit_mint": COLLATERAL_A,
        "quote_deposit_mint": COLLATERAL_B,
    }


def deposit_and_swap_inputs() -> dict:
    """Single-maker DepositAndSwap on the A1/B1 book (GDT B precedes GDT A).

    The SELL taker fully fills (status omitted) from a collateral-A global
    deposit; the partial BUY maker keeps its status and deposits collateral B,
    giving 60 quote atoms for 100 base atoms.
    """
    base_mint = mint(COLLATERAL_A, 1)
    quote_mint = mint(COLLATERAL_B, 1)
    return {
        "operator": OPERATOR,
        "market": MARKET,
        "base_mint": base_mint,
        "quote_mint": quote_mint,
        "fee_receiver": FEE_RECEIVER,
        "taker_order": signed_by(
            101,
            base_mint,
            quote_mint,
            salt=0,
            side=OrderSide.ASK,
            amount_in=100,
            amount_out=60,
            expiration=0,
        ),
        "taker_is_full_fill": True,
        "taker_is_deposit": True,
        "taker_deposit_mint": COLLATERAL_A,
        "num_outcomes": 2,
        "makers": [
            MakerFill(
                order=signed_by(
                    12,
                    base_mint,
                    quote_mint,
                    salt=0x1112131415161718,
                    side=OrderSide.BID,
                    amount_in=90,
                    amount_out=150,
                    expiration=1_700_000_000,
                ),
                maker_fill_amount=60,
                taker_fill_amount=100,
                is_full_fill=False,
                is_deposit=True,
                deposit_mint=COLLATERAL_B,
            )
        ],
        "program_id": CLIENT_PROGRAM_ID,
        "base_deposit_mint": COLLATERAL_A,
        "quote_deposit_mint": COLLATERAL_B,
    }


class TestProgramClientKnownAnswers:
    def test_mint_complete_set(self) -> None:
        ix = build_mint_complete_set_instruction(
            USER, MARKET, COLLATERAL_A, 123_456_789_012, 2, CLIENT_PROGRAM_ID
        )
        assert_ix(ix, MINT_COMPLETE_SET_ACCOUNTS, MINT_COMPLETE_SET_DATA)

    def test_create_orderbook_canonicalizes_both_mint_orders(self) -> None:
        a, b = COLLATERAL_A, COLLATERAL_B
        mint_a1, mint_b1 = mint(a, 1), mint(b, 1)

        def build(mint_a, mint_a_deposit, mint_b, mint_b_deposit, base_index):
            return build_create_orderbook_instruction(
                MANAGER,
                MARKET,
                mint_a,
                mint_b,
                FEE_RECEIVER,
                mint_a_deposit,
                mint_b_deposit,
                base_index,
                1,
                CLIENT_PROGRAM_ID,
            )

        # The two outcome-1 books: base A1/quote B1 and base B1/quote A1. Mint
        # B1 sorts first, so the books share every account except the fee
        # receiver's quote ATA, and only the base index differs in the data.
        for ix in (build(mint_a1, a, mint_b1, b, 0), build(mint_b1, b, mint_a1, a, 1)):
            assert_ix(
                ix, create_orderbook_accounts(CREATE_ORDERBOOK_BASE_A1_ATA), "0f0101"
            )
        for ix in (build(mint_b1, b, mint_a1, a, 0), build(mint_a1, a, mint_b1, b, 1)):
            assert_ix(
                ix, create_orderbook_accounts(CREATE_ORDERBOOK_BASE_B1_ATA), "0f0001"
            )

    def test_close_orderbook(self) -> None:
        orderbook = get_orderbook_pda(
            mint(COLLATERAL_A, 1), mint(COLLATERAL_B, 1), CLIENT_PROGRAM_ID
        )[0]
        ix = build_close_orderbook_instruction(
            CloseOrderbookParams(operator=OPERATOR, orderbook=orderbook, market=MARKET),
            CLIENT_PROGRAM_ID,
        )
        assert_ix(ix, CLOSE_ORDERBOOK_ACCOUNTS, CLOSE_ORDERBOOK_DATA)

    def test_global_deposit_builders(self) -> None:
        deposit = build_deposit_to_global_instruction(
            USER, COLLATERAL_A, (1 << 64) - 2, CLIENT_PROGRAM_ID
        )
        to_market = build_global_to_market_deposit_instruction(
            USER, MARKET, COLLATERAL_B, 500_000, 2, CLIENT_PROGRAM_ID
        )
        assert_ix(deposit, DEPOSIT_TO_GLOBAL_ACCOUNTS, DEPOSIT_TO_GLOBAL_DATA)
        assert_ix(to_market, GLOBAL_TO_MARKET_ACCOUNTS, GLOBAL_TO_MARKET_DATA)

    def test_position_token_account_builders(self) -> None:
        # Two groups in registration order (GDT indices 0 and 3 in the client).
        groups = [
            Pubkey.from_bytes(bytes([0x40]) * 32),
            Pubkey.from_bytes(bytes([0x41]) * 32),
        ]
        init = build_init_position_tokens_instruction(
            OPERATOR, USER, MARKET, groups, 2, CLIENT_PROGRAM_ID
        )
        close = build_close_position_token_accounts_instruction(
            ClosePositionTokenAccountsParams(
                operator=OPERATOR,
                market=MARKET,
                position=get_position_pda(USER, MARKET, CLIENT_PROGRAM_ID)[0],
                deposit_mints=groups,
            ),
            2,
            CLIENT_PROGRAM_ID,
        )
        assert_ix(init, INIT_POSITION_TOKENS_ACCOUNTS, INIT_POSITION_TOKENS_DATA)
        assert_ix(
            close,
            CLOSE_POSITION_TOKEN_ACCOUNTS_ACCOUNTS,
            CLOSE_POSITION_TOKEN_ACCOUNTS_DATA,
        )

    def test_cancel_order(self) -> None:
        order = signed_by(
            50,
            mint(COLLATERAL_A, 0),
            mint(COLLATERAL_B, 0),
            salt=0x0102030405060708,
            side=OrderSide.BID,
            amount_in=123_456,
            amount_out=654_321,
            expiration=1_900_000_000,
        )
        ix = build_cancel_order_instruction(OPERATOR, MARKET, order, CLIENT_PROGRAM_ID)
        # Order id, then the 225-byte signed order.
        assert_ix(ix, CANCEL_ORDER_ACCOUNTS, CANCEL_ORDER_DATA)

    def test_match_orders_multi(self) -> None:
        ix = build_match_orders_multi_instruction(**match_inputs())
        assert_ix(ix, MATCH_ORDERS_MULTI_ACCOUNTS, MATCH_ORDERS_MULTI_DATA)

    def test_deposit_and_swap(self) -> None:
        # The SELL taker receives quote (B1) and gives base (A1); every
        # settlement ATA pair follows that orientation, so the maker's B1 ATA
        # closes its deposit block and then opens its settlement pair.
        ix = build_deposit_and_swap_instruction(**deposit_and_swap_inputs())
        assert_ix(ix, DEPOSIT_AND_SWAP_ACCOUNTS, DEPOSIT_AND_SWAP_DATA)


class TestLocalTradeRejections:
    def test_match_orders_multi_rejects_invalid_trades(self) -> None:
        valid = match_inputs()
        makers = valid["maker_orders"]

        def build(**overrides):
            return build_match_orders_multi_instruction(**{**valid, **overrides})

        with pytest.raises(MissingFieldError, match="maker_orders"):
            build(maker_orders=[])
        with pytest.raises(TooManyMakersError, match="12"):
            build(maker_orders=[makers[1]] * (MAX_MAKERS + 1))
        with pytest.raises(MissingFieldError, match="maker_fill_amounts"):
            build(maker_fill_amounts=[100])
        with pytest.raises(MissingFieldError, match="taker_fill_amounts"):
            build(taker_fill_amounts=[60, 60, 60])

        # A participant signed for another pair or market.
        with pytest.raises(InvalidOrderbookError):
            build(
                maker_orders=[
                    makers[0],
                    replace(makers[1], quote_mint=mint(COLLATERAL_B, 1)),
                ]
            )
        with pytest.raises(InvalidOrderbookError):
            build(
                taker_order=replace(
                    valid["taker_order"], market=Pubkey.from_bytes(bytes([0x4E]) * 32)
                )
            )

        # A maker on the taker's side.
        with pytest.raises(InvalidSideError, match=r": 0$"):
            build(maker_orders=[replace(makers[0], side=OrderSide.BID), makers[1]])

        # Mask bits beyond the two makers; the taker bit alone stays valid.
        for mask in (0x0004, 0x4000, 0x8004):
            with pytest.raises(SerializationError):
                build(full_fill_bitmask=mask)
        build(full_fill_bitmask=0x8003)

        with pytest.raises(DepositMintMismatchError):
            build(quote_deposit_mint=valid["base_deposit_mint"])

    def test_deposit_and_swap_rejects_invalid_trades(self) -> None:
        valid = deposit_and_swap_inputs()
        [maker] = valid["makers"]

        def build(**overrides):
            return build_deposit_and_swap_instruction(**{**valid, **overrides})

        with pytest.raises(MissingFieldError, match="makers"):
            build(makers=[])
        with pytest.raises(TooManyMakersError, match="12"):
            build(makers=[maker] * (MAX_MAKERS + 1))
        for num_outcomes in (1, 7):
            with pytest.raises(InvalidOutcomeCountError):
                build(num_outcomes=num_outcomes)

        # A maker signed for another pair.
        with pytest.raises(InvalidOrderbookError):
            build(
                makers=[
                    replace(
                        maker,
                        order=replace(maker.order, base_mint=mint(COLLATERAL_A, 0)),
                    )
                ]
            )

        # A maker on the taker's side.
        with pytest.raises(InvalidSideError, match=r": 1$"):
            build(
                makers=[replace(maker, order=replace(maker.order, side=OrderSide.ASK))]
            )

        with pytest.raises(DepositMintMismatchError):
            build(quote_deposit_mint=valid["base_deposit_mint"])

        # Depositors must split the collateral behind the asset their order
        # gives: base collateral for the SELL taker, quote for the BUY maker.
        with pytest.raises(DepositMintMismatchError):
            build(taker_deposit_mint=COLLATERAL_B)
        with pytest.raises(DepositMintMismatchError):
            build(makers=[replace(maker, deposit_mint=COLLATERAL_A)])

        # A non-depositing participant's deposit mint is not encoded or checked.
        build(makers=[replace(maker, is_deposit=False, deposit_mint=Pubkey.default())])
