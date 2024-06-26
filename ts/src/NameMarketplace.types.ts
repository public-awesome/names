/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export type Decimal = string;
export interface InstantiateMsg {
  ask_interval: number;
  max_renewals_per_block: number;
  min_price: Uint128;
  operator: string;
  renew_window: number;
  renewal_bid_percentage: Decimal;
  trading_fee_bps: number;
  valid_bid_query_limit: number;
}
export type ExecuteMsg = {
  set_ask: {
    seller: string;
    token_id: string;
  };
} | {
  remove_ask: {
    token_id: string;
  };
} | {
  update_ask: {
    seller: string;
    token_id: string;
  };
} | {
  set_bid: {
    token_id: string;
  };
} | {
  remove_bid: {
    token_id: string;
  };
} | {
  accept_bid: {
    bidder: string;
    token_id: string;
  };
} | {
  migrate_bids: {
    limit: number;
  };
} | {
  fund_renewal: {
    token_id: string;
  };
} | {
  refund_renewal: {
    token_id: string;
  };
} | {
  renew: {
    token_id: string;
  };
} | {
  process_renewals: {
    limit: number;
  };
} | {
  setup: {
    collection: string;
    minter: string;
  };
};
export type QueryMsg = {
  ask: {
    token_id: string;
  };
} | {
  asks: {
    limit?: number | null;
    start_after?: number | null;
  };
} | {
  ask_count: {};
} | {
  asks_by_seller: {
    limit?: number | null;
    seller: string;
    start_after?: string | null;
  };
} | {
  asks_by_renew_time: {
    limit?: number | null;
    max_time: Timestamp;
    start_after?: Timestamp | null;
  };
} | {
  ask_renew_price: {
    current_time: Timestamp;
    token_id: string;
  };
} | {
  ask_renewal_prices: {
    current_time: Timestamp;
    token_ids: string[];
  };
} | {
  bid: {
    bidder: string;
    token_id: string;
  };
} | {
  bids_by_bidder: {
    bidder: string;
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  bids: {
    limit?: number | null;
    start_after?: string | null;
    token_id: string;
  };
} | {
  legacy_bids: {
    limit?: number | null;
    start_after?: BidOffset | null;
  };
} | {
  bids_sorted_by_price: {
    limit?: number | null;
    start_after?: BidOffset | null;
  };
} | {
  reverse_bids_sorted_by_price: {
    limit?: number | null;
    start_before?: BidOffset | null;
  };
} | {
  bids_for_seller: {
    limit?: number | null;
    seller: string;
    start_after?: BidOffset | null;
  };
} | {
  highest_bid: {
    token_id: string;
  };
} | {
  ask_hooks: {};
} | {
  bid_hooks: {};
} | {
  sale_hooks: {};
} | {
  params: {};
} | {
  renewal_queue: {
    time: Timestamp;
  };
} | {
  config: {};
};
export type Timestamp = Uint64;
export type Uint64 = string;
export type Addr = string;
export interface BidOffset {
  bidder: Addr;
  price: Uint128;
  token_id: string;
}
export type NullableAsk = Ask | null;
export interface Ask {
  id: number;
  renewal_fund: Uint128;
  renewal_time: Timestamp;
  seller: Addr;
  token_id: string;
}
export interface HooksResponse {
  hooks: string[];
}
export type TupleOfNullable_CoinAndNullable_Bid = [Coin | null, Bid | null];
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface Bid {
  amount: Uint128;
  bidder: Addr;
  created_time: Timestamp;
  token_id: string;
}
export type ArrayOfAskRenewPriceResponse = AskRenewPriceResponse[];
export interface AskRenewPriceResponse {
  bid?: Bid | null;
  price: Coin;
  token_id: string;
}
export type ArrayOfAsk = Ask[];
export type NullableBid = Bid | null;
export type ArrayOfBid = Bid[];
export interface ConfigResponse {
  collection: Addr;
  minter: Addr;
}
export interface SudoParams {
  ask_interval: number;
  max_renewals_per_block: number;
  min_price: Uint128;
  operator: Addr;
  renew_window: number;
  renewal_bid_percentage: Decimal;
  trading_fee_percent: Decimal;
  valid_bid_query_limit: number;
}