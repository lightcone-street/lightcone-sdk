import { SdkError } from "../../error";
import { deriveOrderStatus, type LimitOrder } from "./model";
import { UserOpenLimitOrders } from "./state";
import { orderUpdatePrice, type OrderUpdate, type UserOrder, type UserSnapshotOrder } from "./wire";

/** Convert a live `order` fact. Fails when its signed amounts define no limit price. */
export function limitOrderFromUpdate(update: OrderUpdate): LimitOrder {
  const price = orderUpdatePrice(update);
  if (price === undefined) {
    throw SdkError.validation(`order ${update.order_hash}: signed amounts define no limit price`);
  }
  return {
    status: deriveOrderStatus(
      update.closed_reason,
      update.original_base,
      update.confirmed_base,
      update.pending_base,
    ),
    marketPubkey: update.market_pubkey,
    orderbookId: update.orderbook_id,
    baseMint: update.base_mint,
    quoteMint: update.quote_mint,
    orderHash: update.order_hash,
    side: update.side,
    price,
    size: update.original_base,
    filledSize: update.confirmed_base,
    pendingSize: update.pending_base,
    remainingSize: update.open_base,
    cancelledSize: update.cancelled_base,
    timeInForce: update.tif,
    fundingSource: update.funding_source,
    closedReason: update.closed_reason,
    acceptedSeq: update.accepted_seq,
    committedRevision: update.commit.committed_revision,
    createdAt: update.accepted_at,
    expiration: update.expiration,
  };
}

/** Convert one order of the WS `user` snapshot. */
export function limitOrderFromSnapshot(order: UserSnapshotOrder): LimitOrder {
  return {
    status: deriveOrderStatus(
      order.closed_reason,
      order.original_base,
      order.confirmed_base,
      order.pending_base,
    ),
    marketPubkey: order.market_pubkey,
    orderbookId: order.orderbook_id,
    baseMint: order.base_mint,
    quoteMint: order.quote_mint,
    orderHash: order.order_hash,
    side: order.side,
    price: order.price,
    size: order.original_base,
    filledSize: order.confirmed_base,
    pendingSize: order.pending_base,
    remainingSize: order.open_base,
    cancelledSize: order.cancelled_base,
    timeInForce: order.tif,
    fundingSource: order.funding_source,
    closedReason: order.closed_reason,
    acceptedSeq: order.accepted_seq,
    committedRevision: order.committed_revision,
    createdAt: order.created_at,
    expiration: order.expiration,
  };
}

/**
 * Convert a REST order; `undefined` when the backend omitted the recorded
 * quantities.
 */
export function limitOrderFromUserOrder(order: UserOrder): LimitOrder | undefined {
  const state = order.state;
  if (state === undefined) return undefined;
  return {
    status: deriveOrderStatus(
      state.closed_reason,
      state.original_base,
      state.confirmed_base,
      state.pending_base,
    ),
    marketPubkey: order.market_pubkey,
    orderbookId: order.orderbook_id,
    baseMint: order.base_mint,
    quoteMint: order.quote_mint,
    orderHash: order.order_hash,
    side: order.side,
    price: order.price,
    size: state.original_base,
    filledSize: state.confirmed_base,
    pendingSize: state.pending_base,
    remainingSize: state.open_base,
    cancelledSize: state.cancelled_base,
    timeInForce: order.tif,
    fundingSource: order.source,
    closedReason: state.closed_reason,
    acceptedSeq: state.accepted_seq,
    committedRevision: state.committed_revision,
    createdAt: order.created_at,
    expiration: order.expiration,
  };
}

/**
 * Build the open-order container from a WS `user` snapshot page.
 *
 * Orders that are neither resting nor awaiting fill confirmation are skipped.
 * The snapshot is the first bounded page only; merge later REST pages with
 * `UserOpenLimitOrders.applyOrder`.
 */
export function convertSnapshotOrders(orders: UserSnapshotOrder[]): UserOpenLimitOrders {
  const openOrders = new UserOpenLimitOrders();
  for (const order of orders) {
    openOrders.applyOrder(limitOrderFromSnapshot(order));
  }
  return openOrders;
}
