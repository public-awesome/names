/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.20.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { Coin } from "@cosmjs/amino";
import { MsgExecuteContractEncodeObject } from "cosmwasm";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { InstantiateMsg, ExecuteMsg, QueryMsg, Uint64, AdminResponse, Addr, ConfigResponse, Config, Boolean, IsProcessableResponse, Decimal, PerAddressLimitResponse } from "./WhitelistUpdatable.types";
export interface WhitelistUpdatableMessage {
  contractAddress: string;
  sender: string;
  updateAdmin: ({
    newAdmin
  }: {
    newAdmin: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  addAddresses: ({
    addresses
  }: {
    addresses: string[];
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  removeAddresses: ({
    addresses
  }: {
    addresses: string[];
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  processAddress: ({
    address
  }: {
    address: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updatePerAddressLimit: ({
    limit
  }: {
    limit: number;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  purge: (funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class WhitelistUpdatableMessageComposer implements WhitelistUpdatableMessage {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.updateAdmin = this.updateAdmin.bind(this);
    this.addAddresses = this.addAddresses.bind(this);
    this.removeAddresses = this.removeAddresses.bind(this);
    this.processAddress = this.processAddress.bind(this);
    this.updatePerAddressLimit = this.updatePerAddressLimit.bind(this);
    this.purge = this.purge.bind(this);
  }

  updateAdmin = ({
    newAdmin
  }: {
    newAdmin: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_admin: {
            new_admin: newAdmin
          }
        })),
        funds
      })
    };
  };
  addAddresses = ({
    addresses
  }: {
    addresses: string[];
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          add_addresses: {
            addresses
          }
        })),
        funds
      })
    };
  };
  removeAddresses = ({
    addresses
  }: {
    addresses: string[];
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          remove_addresses: {
            addresses
          }
        })),
        funds
      })
    };
  };
  processAddress = ({
    address
  }: {
    address: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          process_address: {
            address
          }
        })),
        funds
      })
    };
  };
  updatePerAddressLimit = ({
    limit
  }: {
    limit: number;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_per_address_limit: {
            limit
          }
        })),
        funds
      })
    };
  };
  purge = (funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          purge: {}
        })),
        funds
      })
    };
  };
}