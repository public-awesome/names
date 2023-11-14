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
  FAIR_BURN: 'stargaze_fair_burn',
  ROYALTY_REGISTRY: 'stargaze_royalty_registry',

  // launchpad artifacts
  VENDING_MINTER: 'vending_minter',
  VENDING_FACTORY: 'vending_factory',
  SG721_BASE: 'sg721_base',

  // // marketplace artifacts
  // MARKETPLACE: 'sg_marketplace',
  // MARKETPLACE_V2: 'stargaze_marketplace_v2',
  // RESERVE_AUCTION: 'stargaze_reserve_auction',

  // infinity artifacts
  INFINITY_BUILDER: 'infinity_builder',
  INFINITY_FACTORY: 'infinity_factory',
  INFINITY_GLOBAL: 'infinity_global',
  INFINITY_INDEX: 'infinity_index',
  INFINITY_PAIR: 'infinity_pair',
  INFINITY_ROUTER: 'infinity_router',
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

  private instantiateInfinityBuilder = async (
    client: SigningCosmWasmClient,
    sender: string,
    fairBurnAddress: string,
    marketplaceAddress: string,
    royaltyRegistryAddress: string,
  ) => {
    let infinityBuilderInstantiateMsg: InfinityBuilderInstantiateMsg = {
      code_ids: {
        infinity_factory: this.codeIds[CONTRACT_MAP.INFINITY_FACTORY],
        infinity_global: this.codeIds[CONTRACT_MAP.INFINITY_GLOBAL],
        infinity_index: this.codeIds[CONTRACT_MAP.INFINITY_INDEX],
        infinity_pair: this.codeIds[CONTRACT_MAP.INFINITY_PAIR],
        infinity_router: this.codeIds[CONTRACT_MAP.INFINITY_ROUTER],
      },
      fair_burn: fairBurnAddress,
      fair_burn_fee_percent: '0.005',
      marketplace: marketplaceAddress,
      default_royalty_fee_percent: '0.005',
      max_royalty_fee_percent: '0.05',
      max_swap_fee_percent: '0.10',
      min_prices: [{ amount: '1000000', denom: 'ustars' }],
      pair_creation_fee: { amount: '100000000', denom: 'ustars' },
      royalty_registry: royaltyRegistryAddress,
    }

    const checksumFilePath = path.join(chainConfig.artifacts_path, 'checksums.txt')
    const checksum = await readChecksumFile(checksumFilePath, 'infinity_builder.wasm')
    const checksumUint8Array = fromHex(checksum)
    const salt = fromHex('00')
    const address2 = instantiate2Address(checksumUint8Array, sender, salt, 'stars')

    const instantiateInfinityBuilderResult = await client.instantiate2(
      sender,
      this.codeIds[CONTRACT_MAP.INFINITY_BUILDER],
      salt,
      infinityBuilderInstantiateMsg,
      CONTRACT_MAP.INFINITY_BUILDER,
      'auto',
    )

    this.addContractAddress(CONTRACT_MAP.INFINITY_BUILDER, instantiateInfinityBuilderResult.contractAddress)
    console.log(
      `Instantiated ${CONTRACT_MAP.INFINITY_BUILDER} contract with address ${instantiateInfinityBuilderResult.contractAddress}`,
    )
    assert(address2 === instantiateInfinityBuilderResult.contractAddress, 'address2 does not match')

    _.forEach(instantiateInfinityBuilderResult.events, (event) => {
      if (event.type === 'instantiate') {
        let codeId = parseInt(event.attributes[1].value, 10)
        let contractKey = this.getContractKeyByCodeId(codeId)
        assert(contractKey, 'contractKey not found in wasm event attributes')
        this.addContractAddress(contractKey, event.attributes[0].value)
      }
    })
  }

  private instantiateContracts = async () => {
    let { client, address: sender } = this.getTestUser('user1')

    // Instantiate stargaze_fair_burn
    let instantiateFairBurnResult = await this.instantiateContract(client, sender, CONTRACT_MAP.FAIR_BURN, {
      fee_bps: 5000,
    })

    // Instantiate stargaze_royalty_registry
    let royaltyRegistryInstantiateMsg: RoyaltyRegistryInstantiateMsg = {
      config: {
        max_share_delta: '0.10',
        update_wait_period: 12,
      },
    }
    let instantiateRoyaltyRegistryResult = await this.instantiateContract(
      client,
      sender,
      CONTRACT_MAP.ROYALTY_REGISTRY,
      royaltyRegistryInstantiateMsg,
    )

    // Instantiate vending_factory
    let vendingFactoryInstantiateMsg: VendingFactoryInstantiateMsg = {
      params: {
        allowed_sg721_code_ids: [this.codeIds[CONTRACT_MAP.SG721_BASE]],
        code_id: this.codeIds[CONTRACT_MAP.VENDING_MINTER],
        creation_fee: { amount: '1000000', denom: 'ustars' },
        frozen: false,
        max_trading_offset_secs: 60 * 60,
        min_mint_price: { amount: '1000000', denom: 'ustars' },
        mint_fee_bps: 200,
        extension: {
          airdrop_mint_fee_bps: 200,
          airdrop_mint_price: { amount: '1000000', denom: 'ustars' },
          max_per_address_limit: 10_000,
          max_token_limit: 10_000,
          shuffle_fee: { amount: '1000000', denom: 'ustars' },
        },
      },
    }
    await this.instantiateContract(client, sender, CONTRACT_MAP.VENDING_FACTORY, vendingFactoryInstantiateMsg)

    await this.instantiateInfinityBuilder(
      client,
      sender,
      instantiateFairBurnResult.contractAddress,
      instantiateFairBurnResult.contractAddress,
      instantiateRoyaltyRegistryResult.contractAddress,
    )
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
