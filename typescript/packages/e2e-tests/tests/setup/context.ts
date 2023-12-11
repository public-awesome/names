import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import chainConfig, { denom } from '../../configs/chain_config.json'
import testAccounts from '../../configs/test_accounts.json'
import { getSigningClient } from '../utils/client'
import fs from 'fs'
import _ from 'lodash'
import path from 'path'

export const CONTRACT_MAP = {
  // core artifacts
  MARKETPLACE: 'name_marketplace',
  SG721_NAME: 'sg721_name',
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

  private sg721ContractAddress: string = ''

  private extractAttribute = (instantiateNameMinter: any, attr: string) => {
    const { events } = instantiateNameMinter
    for (let i = 0; i < events.length; i++) {
      const event = events[i]
      if (event.type === 'wasm') {
        const { attributes } = event
        for (let j = 0; j < attributes.length; j++) {
          const attribute = attributes[j]
          if (attribute.key === attr) {
            return attribute.value
          }
        }
      }
    }
  }

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

    const inistantiateMarketpace = await this.instantiateContract(client, sender, CONTRACT_MAP.MARKETPLACE, {
      trading_fee_bps: 200,
      min_price: '5000000',
      ask_interval: 60,
    })

    const instantiateWhitelistUpdatable = await this.instantiateContract(
      client,
      sender,
      CONTRACT_MAP.WHITELIST_UPDATABLE,
      {
        addresses: [this.getTestUser('user1').address],
        per_address_limit: 10,
        mint_discount_bps: 0,
      },
    )

    const instantiateNameMinter = await this.instantiateContract(client, sender, CONTRACT_MAP.NAME_MINTER, {
      admin: this.getTestUser('user1').address,
      collection_code_id: this.codeIds[CONTRACT_MAP.SG721_NAME],
      marketplace_addr: this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      min_name_length: 3,
      max_name_length: 63,
      base_price: '100000000',
      fair_burn_bps: 6666,
      whitelists: [this.extractAttribute(instantiateWhitelistUpdatable, 'whitelist_addr')],
    })

    const setupMarketplaceMsg = {
      setup: {
        minter: this.getContractAddress(CONTRACT_MAP.NAME_MINTER),
        collection: this.extractAttribute(instantiateNameMinter, 'sg721_names_addr'),
      },
    }

    this.sg721ContractAddress = this.extractAttribute(instantiateNameMinter, 'sg721_names_addr')

    const setupResult = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      setupMarketplaceMsg,
      'auto',
      undefined,
      [],
    )
// here
    const updateWhitelistMsg = {
      add_whitelist: {
        address: this.getTestUser('user1').address,
      },
    }

    await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.NAME_MINTER),
      updateWhitelistMsg,
      'auto',
      undefined,
      [],
    )

    const approveMarketplaceMsg = {
      approve_all: {
        operator: this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      },
    }

    await client.execute(
      sender,
      this.extractAttribute(instantiateNameMinter, 'sg721_names_addr'),
      approveMarketplaceMsg,
      'auto',
      undefined,
      [],
    )

    // mint name
    let mintNameMsg = {
      mint_and_list: {
        name: 'testname2',
      },
    }

    let mintNameResult = await client.execute(
      this.getTestUser('user1').address,
      this.getContractAddress(CONTRACT_MAP.NAME_MINTER),
      mintNameMsg,
      'auto',
      undefined,
      [{ denom, amount: '100000000' }],
    )
    // console.log(`Minted name ${mintNameResult.transactionHash}`)
    // console.log(mintNameResult)
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

  getAsks = async () => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      asks: {},
    }

    const result = await client.queryContractSmart(this.getContractAddress(CONTRACT_MAP.MARKETPLACE), queryMsg)
    return result
  }

  countAsks = async () => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      ask_count: {},
    }

    const result = await client.queryContractSmart(this.getContractAddress(CONTRACT_MAP.MARKETPLACE), queryMsg)
    return result
  }

  placeBid = async (name: string, price: string, bidder: string) => {
    const { client, address: sender } = this.getTestUser(bidder)
    const queryMsg = {
      set_bid: {
        token_id: name,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [{ denom, amount: price }],
    )
    return result
  }
  getBids = async (token_id: string, bidder: string) => {
    const { client, address: sender } = this.getTestUser(bidder)
    const queryMsg = {
      bids: {
        token_id,
      },
    }

    const result = await client.queryContractSmart(this.getContractAddress(CONTRACT_MAP.MARKETPLACE), queryMsg)
    return result
  }

  removeBid = async (name: string, bidder: string) => {
    const { client, address: sender } = this.getTestUser(bidder)
    const queryMsg = {
      remove_bid: {
        token_id: name,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [],
    )
    return result
  }
  acceptBid = async (token_id: string, bidder: string) => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      accept_bid: {
        token_id,
        bidder: this.getTestUser(bidder).address,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [],
    )
    return result
  }
  fundRenewal = async (token_id: string, amount: string) => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      fund_renewal: {
        token_id,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [{ denom, amount }],
    )
    return result
  }

  refundRenewal = async (token_id: string, bidder: string) => {
    const { client, address: sender } = this.getTestUser(bidder)
    const queryMsg = {
      refund_renewal: {
        token_id,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [],
    )
    return result
  }

  mintName = async (name: string, user: string) => {
    const { client, address: sender } = this.getTestUser(user)
    const queryMsg = {
      mint_and_list: {
        name,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.NAME_MINTER),
      queryMsg,
      'auto',
      undefined,
      [{ denom, amount: '100000000' }],
    )
    return result
  }

  updateWhitelist = async (address: string, user: string) => {
    const { client, address: sender } = this.getTestUser(user)
    const updateWhitelistMsg = {
      add_whitelist: {
        address: this.getTestUser(address).address,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.NAME_MINTER),
      updateWhitelistMsg,
      'auto',
      undefined,
      [],
    )
    return result
  }

  getRenewalQueue = async (time: string) => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      renewal_queue: {
        time,
      },
    }

    const result = await client.queryContractSmart(this.getContractAddress(CONTRACT_MAP.MARKETPLACE), queryMsg)
    return result
  }

  processRenewal = async (time: string) => {
    const { client, address: sender } = this.getTestUser('user1')
    const queryMsg = {
      process_renewals: {
        time,
      },
    }

    const result = await client.execute(
      sender,
      this.getContractAddress(CONTRACT_MAP.MARKETPLACE),
      queryMsg,
      'auto',
      undefined,
      [],
    )
    return result
    }
}
