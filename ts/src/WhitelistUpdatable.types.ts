/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  addresses: string[];
  mint_discount_bps?: number | null;
  per_address_limit: number;
}
export type ExecuteMsg = {
  update_admin: {
    new_admin: string;
  };
} | {
  add_addresses: {
    addresses: string[];
  };
} | {
  remove_addresses: {
    addresses: string[];
  };
} | {
  process_address: {
    address: string;
  };
} | {
  update_per_address_limit: {
    limit: number;
  };
} | {
  purge: {};
};
export type QueryMsg = {
  config: {};
} | {
  includes_address: {
    address: string;
  };
} | {
  mint_count: {
    address: string;
  };
} | {
  is_processable: {
    address: string;
  };
} | {
  admin: {};
} | {
  address_count: {};
} | {
  per_address_limit: {};
} | {
  mint_discount_percent: {};
};
export type Uint64 = number;
export interface AdminResponse {
  admin?: string | null;
}
export type Addr = string;
export interface Config {
  admin: Addr;
  mint_discount_bps?: number | null;
  per_address_limit: number;
}
export type Boolean = boolean;
export type Decimal = string;