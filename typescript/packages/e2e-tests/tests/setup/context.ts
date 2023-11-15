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
  MARKETPLACE: 'stargaze_fair_burn',
  SG721_NAME: 'stargaze_royalty_registry',
  NAME_MINTER: 'stargaze_name_minter',
  WHITELIST_UPDATABLE: 'stargaze_whitelist_updatable',
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
