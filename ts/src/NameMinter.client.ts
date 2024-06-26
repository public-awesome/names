/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { Uint128, InstantiateMsg, ExecuteMsg, Timestamp, Uint64, Config, QueryMsg, AdminResponse, Addr, Decimal, SudoParams, ArrayOfAddr } from "./NameMinter.types";
export interface NameMinterReadOnlyInterface {
  contractAddress: string;
  admin: () => Promise<AdminResponse>;
  whitelists: () => Promise<ArrayOfAddr>;
  collection: () => Promise<Addr>;
  params: () => Promise<SudoParams>;
  config: () => Promise<Config>;
}
export class NameMinterQueryClient implements NameMinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.admin = this.admin.bind(this);
    this.whitelists = this.whitelists.bind(this);
    this.collection = this.collection.bind(this);
    this.params = this.params.bind(this);
    this.config = this.config.bind(this);
  }

  admin = async (): Promise<AdminResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      admin: {}
    });
  };
  whitelists = async (): Promise<ArrayOfAddr> => {
    return this.client.queryContractSmart(this.contractAddress, {
      whitelists: {}
    });
  };
  collection = async (): Promise<Addr> => {
    return this.client.queryContractSmart(this.contractAddress, {
      collection: {}
    });
  };
  params = async (): Promise<SudoParams> => {
    return this.client.queryContractSmart(this.contractAddress, {
      params: {}
    });
  };
  config = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
}
export interface NameMinterInterface extends NameMinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  mintAndList: ({
    name
  }: {
    name: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateAdmin: ({
    admin
  }: {
    admin?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  pause: ({
    pause
  }: {
    pause: boolean;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  addWhitelist: ({
    address,
    whitelistType
  }: {
    address: string;
    whitelistType: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  removeWhitelist: ({
    address
  }: {
    address: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateConfig: ({
    config
  }: {
    config: Config;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class NameMinterClient extends NameMinterQueryClient implements NameMinterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.mintAndList = this.mintAndList.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.pause = this.pause.bind(this);
    this.addWhitelist = this.addWhitelist.bind(this);
    this.removeWhitelist = this.removeWhitelist.bind(this);
    this.updateConfig = this.updateConfig.bind(this);
  }

  mintAndList = async ({
    name
  }: {
    name: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_and_list: {
        name
      }
    }, fee, memo, funds);
  };
  updateAdmin = async ({
    admin
  }: {
    admin?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_admin: {
        admin
      }
    }, fee, memo, funds);
  };
  pause = async ({
    pause
  }: {
    pause: boolean;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      pause: {
        pause
      }
    }, fee, memo, funds);
  };
  addWhitelist = async ({
    address,
    whitelistType
  }: {
    address: string;
    whitelistType: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_whitelist: {
        address,
        whitelist_type: whitelistType
      }
    }, fee, memo, funds);
  };
  removeWhitelist = async ({
    address
  }: {
    address: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      remove_whitelist: {
        address
      }
    }, fee, memo, funds);
  };
  updateConfig = async ({
    config
  }: {
    config: Config;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_config: {
        config
      }
    }, fee, memo, funds);
  };
}