import { SigningCosmWasmClient, instantiate2Address } from '@cosmjs/cosmwasm-stargate'
import { fromHex } from '@cosmjs/encoding'
import chainConfig from '../../configs/chain_config.json'
import testAccounts from '../../configs/test_accounts.json'
import { getSigningClient } from '../utils/client'
import { readChecksumFile } from '../utils/file'
import { InstantiateMsg as RoyaltyRegistryInstantiateMsg } from '@stargazezone/core-types/lib/RoyaltyRegistry.types'
import { InstantiateMsg as InfinityBuilderInstantiateMsg } from '@stargazezone/infinity-types/lib/codegen/InfinityBuilder.types'
import { InstantiateMsg as VendingFactoryInstantiateMsg } from '@stargazezone/launchpad/src/VendingFactory.types'
import assert from 'assert'
import fs from 'fs'
import _ from 'lodash'
import path from 'path'

export const CONTRACT_MAP = {
  // core artifacts
  MARKETPLACE: 'name_marketplace',
  SG721_NAME: 'name_sg721',
  NAME_MINTER: 'name_minter',
  WHITELIST_UPDATABLE: 'whitelist_updatable',
}

export type TestUser = {
  name: string
  address: string
  client: SigningCosmWasmClient
}

export type TestUserMap = { [name: string]: TestUser }

export default class Context {
  private codeIds: { [key: string]: number } = {}
  private contracts: { [key: string]: string[] } = {}
  private testCachePath: string = path.join(__dirname, '../../tmp/test_cache.json')
  private testUserMap: TestUserMap = {}

  private initializeTestUsers = async () => {
    for (let i = 0; i < testAccounts.length; i++) {
      const mnemonic = testAccounts[i].mnemonic
      const signingClient = await getSigningClient(mnemonic)
      const testAccount = testAccounts[i]
      this.testUserMap[testAccount.name] = {
        name: testAccount.name,
        address: testAccounts[i].address,
        client: signingClient.client,
      }
    }
  }

  private hydrateContext = async () => {
    let testCache = JSON.parse(fs.readFileSync(this.testCachePath, 'utf8'))
    this.codeIds = testCache.codeIds
    this.contracts = testCache.contracts
  }

  private uploadContracts = async () => {
    let { client, address: sender } = this.getTestUser('user1')

    let fileNames = fs.readdirSync(chainConfig.artifacts_path)
    let wasmFileNames = _.filter(fileNames, (fileName) => _.endsWith(fileName, '.wasm'))

    for (const idx in wasmFileNames) {
      let wasmFileName = wasmFileNames[idx]
      let wasmFilePath = path.join(chainConfig.artifacts_path, wasmFileName)
      let wasmFile = fs.readFileSync(wasmFilePath, { encoding: null })
      let uploadResult = await client.upload(sender, wasmFile, 'auto')
      let codeIdKey = wasmFileName.replace('-aarch64', '').replace('.wasm', '')
      this.codeIds[codeIdKey] = uploadResult.codeId
      console.log(`Uploaded ${codeIdKey} contract with codeId ${uploadResult.codeId}`)
    }
  }

  private instantiateContract = async (
    client: SigningCosmWasmClient,
    sender: string,
    contractKey: string,
    msg: any,
  ) => {
    let instantiateResult = await client.instantiate(sender, this.codeIds[contractKey], msg, contractKey, 'auto')
    this.addContractAddress(contractKey, instantiateResult.contractAddress)
    console.log(`Instantiated ${contractKey} contract with address ${instantiateResult.contractAddress}`)
    return instantiateResult
  }

  private instantiateContracts = async () => {
    let { client, address: sender } = this.getTestUser('user1')

    let inistantiateMarketpace = await this.instantiateContract(client, sender, CONTRACT_MAP.MARKETPLACE, {
      trading_fee_bps: 100,
      min_price: '1',
      ask_interval: 60,
    })

    let intantiateSG721Name = await this.instantiateContract(client, sender, CONTRACT_MAP.SG721_NAME, {
      verifier: null,
      base_init_msg: {
        /*
      pub name: String,
    pub symbol: String,
    pub minter: String,
    pub collection_info: CollectionInfo<RoyaltyInfoResponse>,
  */
        name: 'Stargaze',
        symbol: 'SGZ',
        minter: this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
        collection_info: {
          /*
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<T>,
*/

          creator: sender,
          description: 'Stargaze NFTs',
          image: 'https://stargaze.zone/images/sgz_logo.png',
          external_link: null,
          explicit_content: null,
          start_trading_time: null,
          royalty_info: null,
        },
      },
    })

    // let instantiateNameMinter = await this.instantiateContract(client, sender, CONTRACT_MAP.NAME_MINTER, {
    //   /*
    //       /// Temporary admin for managing whitelists
    // pub admin: Option<String>,
    // /// Oracle for verifying text records
    // pub verifier: Option<String>,
    // pub collection_code_id: u64,
    // pub marketplace_addr: String,
    // pub min_name_length: u32,
    // pub max_name_length: u32,
    // pub base_price: Uint128,
    // pub fair_burn_bps: u64,
    // pub whitelists: Vec<String>,*/
    //   admin: null,
    //   verifier: null,
    //   collection_code_id: this.codeIds[CONTRACT_MAP.SG721_NAME],
    //   marketplace_addr: this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
    //   min_name_length: 5,
    //   max_name_length: 20,
    //   base_price: '1000000',
    //   fair_burn_bps: 100,
    //   whitelists: [],
    // })
  }

  private writeContext = () => {
    const dir = path.dirname(this.testCachePath)

    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true })
    }

    fs.writeFileSync(
      this.testCachePath,
      JSON.stringify({
        codeIds: this.codeIds,
        contracts: this.contracts,
      }),
    )
  }

  initialize = async (hydrate: boolean) => {
    await this.initializeTestUsers()

    if (hydrate) {
      await this.hydrateContext()
    } else {
      await this.uploadContracts()
      await this.instantiateContracts()
      this.writeContext()
    }
  }

  getTestUser = (userName: string) => {
    return this.testUserMap[userName]
  }

  getCodeId = (codeIdKey: string) => {
    return this.codeIds[codeIdKey]
  }

  getContractKeyByCodeId = (codeId: number) => {
    return _.findKey(this.codeIds, (value, key) => value === codeId)
  }

  getContractAddress = (contractKey: string, index: number = 0) => {
    try {
      return this.contracts[contractKey][index]
    } catch {
      console.log(`error ${contractKey} ${index} ${JSON.stringify(this.contracts)}}`)
    }
    return this.contracts[contractKey][index]
  }

  addContractAddress = (contractKey: string, contractAddress: string) => {
    this.contracts[contractKey] = _.extend([], this.contracts[contractKey], [contractAddress])
  }
}
