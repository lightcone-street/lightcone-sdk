import Decimal from "decimal.js";
import { TimeInForce } from "../../shared";
import type { LimitOrder, TriggerOrder } from "./index";
import type { OrderUpdate, UserSnapshotOrder, UserSnapshotOrderCommon } from "./wire";
import { UserOpenLimitOrders, UserTriggerOrders } from "./state";

export function orderFromUpdate(update: OrderUpdate): LimitOrder {
  const size = new Decimal(update.order.filled).plus(update.order.remaining);

  return {
    marketPubkey: update.market_pubkey,
    orderbookId: update.orderbook_id,
    baseMint: update.order.base_mint,
    quoteMint: update.order.quote_mint,
    orderHash: update.order.order_hash,
    side: update.order.side,
    size: size.toString(),
    price: update.order.price,
    filledSize: update.order.filled,
    remainingSize: update.order.remaining,
    createdAt: new Date(update.order.created_at),
    txSignature: update.tx_signature,
    status: update.order.status,
    outcomeIndex: update.order.outcome_index,
  };
}

export function limitSnapshotToOrder(common: UserSnapshotOrderCommon, txSignature?: string): LimitOrder {
  const size = new Decimal(common.filled).plus(common.remaining);
  return {
    marketPubkey: common.market_pubkey,
    orderbookId: common.orderbook_id,
    orderHash: common.order_hash,
    baseMint: common.base_mint,
    quoteMint: common.quote_mint,
    side: common.side,
    size: size.toString(),
    price: common.price,
    filledSize: common.filled,
    remainingSize: common.remaining,
    createdAt: new Date(common.created_at),
    txSignature,
    status: common.status,
    outcomeIndex: common.outcome_index,
  };
}

export function triggerSnapshotToOrder(
  common: UserSnapshotOrderCommon,
  triggerOrderId: string,
  triggerPrice: string,
  triggerType: import("../../shared").TriggerType,
  timeInForce?: import("../../shared").TimeInForce
): TriggerOrder {
  return {
    triggerOrderId,
    orderHash: common.order_hash,
    marketPubkey: common.market_pubkey,
    orderbookId: common.orderbook_id,
    triggerPrice,
    triggerType,
    side: common.side,
    amountIn: common.amount_in,
    amountOut: common.amount_out,
    timeInForce: timeInForce ?? TimeInForce.Gtc,
    createdAt: new Date(common.created_at),
  };
}

export function splitSnapshotOrders(orders: UserSnapshotOrder[]): [UserOpenLimitOrders, UserTriggerOrders] {
  const openOrders = new UserOpenLimitOrders();
  const triggerOrders = new UserTriggerOrders();

  for (const snapshot of orders) {
    if (snapshot.order_type === "limit") {
      if (!new Decimal(snapshot.remaining).isZero()) {
        openOrders.insert(limitSnapshotToOrder(snapshot, snapshot.tx_signature));
      }
      continue;
    }

    triggerOrders.insert(
      triggerSnapshotToOrder(
        snapshot,
        snapshot.trigger_order_id,
        snapshot.trigger_price,
        snapshot.trigger_type,
        snapshot.time_in_force
      )
    );
  }

  return [openOrders, triggerOrders];
}
