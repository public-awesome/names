/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { Coin } from "@cosmjs/amino";
import { MsgExecuteContractEncodeObject } from "cosmwasm";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Uint128, InstantiateMsg, ExecuteMsg, Timestamp, Uint64, QueryMsg, Addr, BidOffset, Nullable_Ask, Ask, HooksResponse, ArrayOfAsk, Nullable_Bid, Bid, ArrayOfBid, ConfigResponse, Decimal, SudoParams } from "./NameMarketplace.types";
export interface NameMarketplaceMessage {
  contractAddress: string;
  sender: string;
  setAsk: ({
    seller,
    tokenId
  }: {
    seller: string;
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  removeAsk: ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateAsk: ({
    seller,
    tokenId
  }: {
    seller: string;
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  setBid: ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  removeBid: ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  acceptBid: ({
    bidder,
    tokenId
  }: {
    bidder: string;
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  fundRenewal: ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  refundRenewal: ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  processRenewals: ({
    time
  }: {
    time: Timestamp;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
  setup: ({
    collection,
    minter
  }: {
    collection: string;
    minter: string;
  }, funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class NameMarketplaceMessageComposer implements NameMarketplaceMessage {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.setAsk = this.setAsk.bind(this);
    this.removeAsk = this.removeAsk.bind(this);
    this.updateAsk = this.updateAsk.bind(this);
    this.setBid = this.setBid.bind(this);
    this.removeBid = this.removeBid.bind(this);
    this.acceptBid = this.acceptBid.bind(this);
    this.fundRenewal = this.fundRenewal.bind(this);
    this.refundRenewal = this.refundRenewal.bind(this);
    this.processRenewals = this.processRenewals.bind(this);
    this.setup = this.setup.bind(this);
  }

  setAsk = ({
    seller,
    tokenId
  }: {
    seller: string;
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          set_ask: {
            seller,
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  removeAsk = ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          remove_ask: {
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  updateAsk = ({
    seller,
    tokenId
  }: {
    seller: string;
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_ask: {
            seller,
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  setBid = ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          set_bid: {
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  removeBid = ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          remove_bid: {
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  acceptBid = ({
    bidder,
    tokenId
  }: {
    bidder: string;
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          accept_bid: {
            bidder,
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  fundRenewal = ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          fund_renewal: {
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  refundRenewal = ({
    tokenId
  }: {
    tokenId: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          refund_renewal: {
            token_id: tokenId
          }
        })),
        funds
      })
    };
  };
  processRenewals = ({
    time
  }: {
    time: Timestamp;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          process_renewals: {
            time
          }
        })),
        funds
      })
    };
  };
  setup = ({
    collection,
    minter
  }: {
    collection: string;
    minter: string;
  }, funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          setup: {
            collection,
            minter
          }
        })),
        funds
      })
    };
  };
}