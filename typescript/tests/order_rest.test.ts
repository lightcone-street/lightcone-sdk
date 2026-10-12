import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { LightconeClient } from "../src/client";
import {
  CancelStatus,
  cancelAllSuccessFromWire,
  cancelSuccessFromWire,
  InitialCohortState,
  limitOrderFromUserOrder,
  OrderStatus,
  orderFillEventPrice,
  Role,
  submitFilledBase,
  submitOrderResponseFromWire,
  SubmitOrderStatus,
  userOrderFillsResponseFromWire,
  userOrderFromWire,
  userOrdersResponseFromWire,
} from "../src/domain/order";
import { SdkError } from "../src/error";
import { ProgramSdkError } from "../src/program/error";
import { cancelAllBodyFromBase58, cancelBodyFromBase58 } from "../src/domain/order/client";
import bs58 from "bs58";
import { FundingSource, TimeInForce } from "../src/shared";

const HASH = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001";
const U64_MAX = (1n << 64n) - 1n;
const I128_MIN = -(1n << 127n);

function restUserOrder(): Record<string, unknown> {
  return {
    order_hash: HASH,
    market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
    orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
    side: "bid",
    amount_in: "5.500000",
    amount_out: "10.00000000",
    price: "0.5500",
    created_at: 1790685521784,
    expiration: 0,
    base_mint: "BaseMint111111111111111111111111111111111111",
    quote_mint: "QuoteMint11111111111111111111111111111111111",
    outcome_index: -1,
    state: {
      original_base: "100",
      confirmed_base: "40",
      pending_base: "20",
      open_base: "40",
      cancelled_base: "0",
      closed_reason: null,
      committed_revision: "7",
      accepted_seq: "0",
    },
    tif: "FOK",
    source: "global",
    order_type: "limit",
  };
}

describe("order submission response", () => {
  it("is accepted_pending without a committed view", () => {
    const response = submitOrderResponseFromWire({
      order_hash: HASH,
      status: "accepted_pending",
      state: null,
      initial_cohort: null,
      fills: [],
      fills_complete: false,
      fills_next_cursor: null,
    });
    assert.equal(response.status, SubmitOrderStatus.AcceptedPending);
    assert.equal(response.state, undefined);
    assert.equal(response.initial_cohort, undefined);
    assert.equal(submitFilledBase(response), undefined);
  });

  it("decodes statuses and keeps the hash of an unknown one", () => {
    for (const [wire, status] of [
      ["filled", SubmitOrderStatus.Filled],
      ["accepted", SubmitOrderStatus.Accepted],
      ["accepted_pending", SubmitOrderStatus.AcceptedPending],
      ["partial_fill", SubmitOrderStatus.Unknown],
      ["accepted_partial", SubmitOrderStatus.Unknown],
    ] as const) {
      const response = submitOrderResponseFromWire({
        order_hash: HASH,
        status: wire,
        fills: [],
        fills_complete: true,
      });
      assert.equal(response.status, status, wire);
      assert.equal(response.order_hash, HASH);
    }
  });

  it("decodes committed state, the initial cohort, and fills", () => {
    const response = submitOrderResponseFromWire({
      order_hash: HASH,
      status: "accepted_pending",
      state: {
        original_base: "10",
        confirmed_base: "4",
        pending_base: "2",
        open_base: "4",
        executable_base: "4",
        cancelled_base: "0",
        ready: true,
        closed_reason: "",
        committed_revision: "812",
        accepted_seq: "44",
      },
      initial_cohort: {
        state: "pending",
        selected_base: "6",
        known_confirmed_base: "4",
        known_failed_unfilled_base: "0",
        unresolved_base: "2",
      },
      fills: [
        {
          fill_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:1",
          execution_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11",
          counterparty: "Maker111111111111111111111111111111111111111",
          counterparty_order_hash: HASH,
          base_amount: "4",
          quote_amount: "2.2",
          is_maker: false,
          fee_estimate_atoms: "-12",
          fee_bps: -5,
          fee_mint: "QuoteMint11111111111111111111111111111111111",
        },
      ],
      fills_complete: false,
      fills_next_cursor: "cursor",
    });
    assert.equal(response.state?.committed_revision, 812n);
    assert.equal(response.state?.closed_reason, undefined);
    assert.equal(submitFilledBase(response), "4");
    assert.equal(response.initial_cohort?.state, InitialCohortState.Pending);
    assert.equal(response.fills[0]!.fee_estimate_atoms, -12n);
    assert.equal(response.fills_next_cursor, "cursor");
  });
});

describe("cancellation request bodies", () => {
  it("require a 64-byte base58 signature, as Rust's from_base58 does", () => {
    const wallet = "Wallet1111111111111111111111111111111111111" as never;
    const good = bs58.encode(Buffer.alloc(64, 7));
    assert.equal(cancelBodyFromBase58(HASH, wallet, good).signature, "07".repeat(64));
    const isInvalidSignature = (error: unknown) =>
      error instanceof ProgramSdkError && error.variant === "InvalidSignature";
    for (const signature of [bs58.encode(Buffer.alloc(63, 7)), bs58.encode(Buffer.alloc(65, 7)), "0OIl"]) {
      assert.throws(() => cancelBodyFromBase58(HASH, wallet, signature), isInvalidSignature);
      assert.throws(
        () => cancelAllBodyFromBase58(wallet, "" as never, 1, "salt", signature),
        isInvalidSignature,
      );
    }
  });
});

describe("cancellation responses", () => {
  it("decodes raw base atoms and closed reasons", () => {
    const response = cancelSuccessFromWire({
      status: "cancelled",
      order_hash: HASH,
      quantities: {
        newly_cancelled_base: "18446744073709551615",
        confirmed_base: "0",
        pending_base: "40",
        remaining_open_base: "0",
      },
      quantity_unit: "base_atoms",
      revision: "812",
      closed_reason: "",
    });
    assert.equal(response.status, CancelStatus.Cancelled);
    assert.equal(response.quantities?.newly_cancelled_base, U64_MAX);
    assert.equal(response.quantities?.pending_base, 40n);
    assert.equal(response.quantity_unit, "base_atoms");
    assert.equal(response.revision, 812n);
    assert.equal(response.closed_reason, undefined);

    const already = cancelSuccessFromWire({
      status: "already_closed",
      order_hash: HASH,
      quantities: null,
      quantity_unit: "base_atoms",
      revision: "813",
      closed_reason: "expired",
    });
    assert.equal(already.status, CancelStatus.AlreadyClosed);
    assert.equal(already.quantities, undefined);
    assert.equal(already.closed_reason, "expired");

    assert.equal(
      cancelSuccessFromWire({ ...already, status: "superseded", revision: "1" }).status,
      CancelStatus.Unknown,
    );
  });

  it("decodes the cancel-all closure", () => {
    const response = cancelAllSuccessFromWire({
      status: "success",
      user_pubkey: "Wallet1111111111111111111111111111111111111",
      orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
      closure: {
        operation_id: "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2c",
        committed_revision: "900",
        scope:
          "WalletBook:Wallet1111111111111111111111111111111111111:j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
        accepted_seq_cutoff: "44",
        cleanup_pending: true,
      },
      message: "Accepted-order cutoff committed",
    });
    assert.equal(response.closure?.committed_revision, 900n);
    assert.equal(response.closure?.accepted_seq_cutoff, 44n);
    assert.ok(response.closure?.cleanup_pending);
    assert.ok(response.closure?.scope.startsWith("WalletBook:"));

    const walletWide = cancelAllSuccessFromWire({
      status: "success",
      user_pubkey: "Wallet1111111111111111111111111111111111111",
      orderbook_id: "",
      closure: {
        operation_id: "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2d",
        committed_revision: "901",
        scope: "Wallet:Wallet1111111111111111111111111111111111111",
        accepted_seq_cutoff: null,
        cleanup_pending: false,
      },
      message: "Accepted-order cutoff committed",
    });
    assert.equal(walletWide.orderbook_id, "");
    assert.equal(walletWide.closure?.accepted_seq_cutoff, undefined);
  });
});

describe("user orders", () => {
  it("decodes orders and the funding page", () => {
    const response = userOrdersResponseFromWire({
      user_pubkey: "Wallet1111111111111111111111111111111111111",
      orders: [{ ...restUserOrder(), side: "ask", state: { ...(restUserOrder().state as object), open_base: "4.00000000", accepted_seq: "45", committed_revision: "812" }, source: "conditional" }],
      funding_accounts: [
        {
          account: "Acct1111111111111111111111111111111111111111",
          mint: "Cond1111111111111111111111111111111111111111",
          source: "conditional",
          market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
          deposit_mint: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
          raw_observed: "4.00000000",
          order_reserved: "340282366920938463463374607431768.211455",
          execution_reserved: "0.00000000",
          signed_remaining_atoms: "0",
          accepted_boundary: "boundary",
          observed_slot: "123",
          observed_blockhash: "hash",
          observation_state: "present",
        },
      ],
      next_cursor: null,
      has_more: false,
      next_funding_cursor: null,
      funding_has_more: false,
      committed_revision: "812",
      projection_generation: "1",
    });
    assert.equal(response.committed_revision, 812n);
    const order = limitOrderFromUserOrder(response.orders[0]!);
    assert.ok(order);
    assert.equal(order.remainingSize, "4.00000000");
    assert.equal(order.acceptedSeq, 45n);
    assert.equal(order.fundingSource, FundingSource.Conditional);
    const account = response.funding_accounts[0]!;
    assert.equal(account.order_reserved, "340282366920938463463374607431768.211455");
    assert.equal(account.observed_slot, 123n);
  });

  it("can return an empty page that still has more", () => {
    const response = userOrdersResponseFromWire({
      user_pubkey: "Wallet1111111111111111111111111111111111111",
      orders: [],
      funding_accounts: [],
      next_cursor: "45:4f1a",
      has_more: true,
      next_funding_cursor: null,
      funding_has_more: false,
      committed_revision: "812",
      projection_generation: "1",
    });
    assert.equal(response.orders.length, 0);
    assert.ok(response.has_more);
    assert.equal(response.next_cursor, "45:4f1a");
  });

  it("decodes a REST order's recorded state", () => {
    const order = userOrderFromWire(restUserOrder());
    assert.equal(order.state?.pending_base, "20");
    assert.equal(order.state?.committed_revision, 7n);
    assert.equal(order.tif, TimeInForce.Fok);
    assert.equal(order.source, FundingSource.Global);
    assert.equal(order.outcome_index, -1);
  });

  it("does not fail the payload on unknown wire values", () => {
    const order = userOrderFromWire({ ...restUserOrder(), tif: "unsupported", source: "vault" });
    assert.equal(order.tif, undefined);
    assert.equal(order.source, FundingSource.Unknown);
    assert.equal(limitOrderFromUserOrder(order)?.timeInForce, undefined);

    const { tif: _tif, ...withoutTif } = restUserOrder();
    assert.equal(userOrderFromWire(withoutTif).tif, undefined);
  });

  it("does not convert an order without recorded state", () => {
    assert.equal(limitOrderFromUserOrder(userOrderFromWire({ ...restUserOrder(), state: null })), undefined);
  });

  it("names the field of a malformed order", () => {
    assert.throws(
      () => userOrderFromWire({ ...restUserOrder(), price: 0.55 }),
      (error) => error instanceof SdkError && error.variant === "Serde" && /order\.price/.test(error.message),
    );
  });
});

describe("order fills", () => {
  it("decodes the backend shape", () => {
    const response = userOrderFillsResponseFromWire({
      orders: [
        {
          order_hash: HASH,
          market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
          orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
          side: "bid",
          base_mint: "BaseMint111111111111111111111111111111111111",
          quote_mint: "QuoteMint11111111111111111111111111111111111",
          original_base: "10.00000000",
          confirmed_base: "4.00000000",
          pending_base: "0.00000000",
          open_base: "0.00000000",
          cancelled_base: "6.00000000",
          confirmed_quote: "2.200000",
          status: "closed",
          closed_reason: "expired",
          created_at: 1790685521784,
          fills: [
            {
              fill_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:1",
              counterparty: "Maker111111111111111111111111111111111111111",
              counterparty_order_hash: HASH,
              role: "taker",
              base_amount: "4.00000000",
              quote_amount: "2.200000",
              fee_estimate_atoms: "-170141183460469231731687303715884105728",
              fee_mint: "QuoteMint11111111111111111111111111111111111",
              maker_fee_bps: -5,
              taker_fee_bps: 30,
              tx_signature:
                "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
              filled_at: 1790685522000,
            },
          ],
          fills_has_more: false,
          fills_next_cursor: null,
        },
      ],
      next_cursor: "eyJhdCI6IjIwMjYtMDktMjlUMTI6MDA6MDBaIiwia2V5IjoiYWIifQ",
      has_more: true,
      committed_revision: "812",
    });
    assert.equal(response.committed_revision, 812n);
    const order = response.orders[0]!;
    assert.equal(order.status, OrderStatus.Closed);
    assert.equal(order.closed_reason, "expired");
    const fill = order.fills[0]!;
    assert.equal(fill.role, Role.Taker);
    assert.equal(fill.fee_estimate_atoms, I128_MIN);
    assert.equal(orderFillEventPrice(fill), "0.55");
  });

  it("decodes unknown roles and statuses as unknown", () => {
    const fill = {
      fill_id: "f",
      counterparty: "Maker111111111111111111111111111111111111111",
      counterparty_order_hash: HASH,
      role: "arbiter",
      base_amount: "1",
      quote_amount: "1",
      fee_estimate_atoms: "0",
      fee_mint: "QuoteMint11111111111111111111111111111111111",
      maker_fee_bps: 0,
      taker_fee_bps: 0,
      tx_signature: "sig",
      filled_at: 1,
    };
    const response = userOrderFillsResponseFromWire({
      orders: [
        {
          order_hash: HASH,
          market_pubkey: "m",
          orderbook_id: "ob",
          side: "ask",
          base_mint: "b",
          quote_mint: "q",
          original_base: "1",
          confirmed_base: "1",
          pending_base: "0",
          open_base: "0",
          cancelled_base: "0",
          confirmed_quote: "1",
          status: "settling",
          created_at: 1,
          fills: [fill],
        },
      ],
      has_more: false,
      committed_revision: 1,
    });
    assert.equal(response.orders[0]!.status, OrderStatus.Unknown);
    assert.equal(response.orders[0]!.fills[0]!.role, Role.Unknown);
    assert.equal(response.orders[0]!.fills_has_more, false);
    assert.equal(response.next_cursor, undefined);
  });
});

describe("orders client", () => {
  function mockedClient(response: unknown) {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    const urls: string[] = [];
    (client.http as unknown as { get: (url: string) => Promise<unknown> }).get = async (url) => {
      urls.push(url);
      return response;
    };
    return { client, urls };
  }

  it("sends only the given order-fills parameters", async () => {
    const { client, urls } = mockedClient({ orders: [], has_more: false, committed_revision: "1" });
    await client.orders().getUserOrderFills(undefined, 20, "c", 0);
    await client.orders().getOrderFillPage(HASH, "f");
    await client.orders().getUserOrderFillsByWallet("Wallet1111111111111111111111111111111111111");
    assert.deepEqual(urls, [
      "https://example.test/api/users/order-fills?limit=20&cursor=c&fills=0",
      `https://example.test/api/users/order-fills?order_hash=${HASH}&fill_cursor=f`,
      "https://example.test/api/users/Wallet1111111111111111111111111111111111111/order-fills",
    ]);
  });

  it("decodes the user orders route", async () => {
    const { client, urls } = mockedClient({
      user_pubkey: "Wallet1111111111111111111111111111111111111",
      orders: [restUserOrder()],
      has_more: false,
      committed_revision: "9",
      projection_generation: "1",
    });
    const response = await client.orders().getUserOrders(50, "45:4f1a");
    assert.equal(urls[0], "https://example.test/api/users/orders?limit=50&cursor=45%3A4f1a");
    assert.equal(response.funding_accounts.length, 0);
    assert.equal(response.orders[0]!.state?.accepted_seq, 0n);
  });
});
