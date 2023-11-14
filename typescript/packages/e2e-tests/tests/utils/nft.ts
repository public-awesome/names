import { CosmWasmClient, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { toUtf8 } from '@cosmjs/encoding'
import Context, { CONTRACT_MAP, TestUser } from '../setup/context'
import { Sg2ExecuteMsgForVendingMinterInitMsgExtension } from '../types/vendingFactory'
import { getQueryClient, getSigningClient } from './client'
import { getFutureTimestamp, nanoToMs, waitUntil } from './datetime'
import { sleep } from './sleep'
import { GlobalConfigForAddr } from '@stargazezone/infinity-types/lib/codegen/InfinityGlobal.types'
import { ExecuteMsg as BaseFactoryExecuteMsg } from '@stargazezone/launchpad/src/BaseFactory.types'
import assert from 'assert'
import { MsgExecuteContract } from 'cosmjs-types/cosmwasm/wasm/v1/tx'
import _ from 'lodash'

export const createMinter = async (context: Context) => {
  const queryClient = await getQueryClient()

  let vendingFactoryAddress = context.getContractAddress(CONTRACT_MAP.VENDING_FACTORY)
  let { params: factoryParams } = await queryClient.queryContractSmart(vendingFactoryAddress, {
    params: {},
  })

  const { client: signingClient, address: sender } = context.getTestUser('user1')
  let msg: Sg2ExecuteMsgForVendingMinterInitMsgExtension = {
    create_minter: {
      init_msg: {
        base_token_uri: 'ipfs://bafybeiek33kk3js27dhodwadtmrf3p6b64netr6t3xzi3sbfyxovxe36qe',
        payment_address: sender,
        start_time: getFutureTimestamp(8),
        num_tokens: 10_000,
        mint_price: { amount: '1000000', denom: 'ustars' },
        per_address_limit: 100,
        whitelist: null,
      },
      collection_params: {
        code_id: context.getCodeId(CONTRACT_MAP.SG721_BASE),
        name: 'Test Collection',
        symbol: 'TC',
        info: {
          creator: sender,
          description: 'This is the collection description',
          image: 'ipfs://bafybeiek33kk3js27dhodwadtmrf3p6b64netr6t3xzi3sbfyxovxe36qe/1.png',
          start_trading_time: getFutureTimestamp(8),
          royalty_info: {
            payment_address: sender,
            share: '0.05',
          },
        },
      },
    },
  }
  let executeResult = await signingClient.execute(
    sender,
    vendingFactoryAddress,
    msg,
    'auto',
    'instantiate-vending-minter',
    [factoryParams.creation_fee],
  )

  let instantiateEvents = _.filter(executeResult.events, (event) => {
    return event.type === 'instantiate'
  })

  let minterAddress = instantiateEvents[0].attributes[0].value
  let collectionAddress = instantiateEvents[1].attributes[0].value

  context.addContractAddress(CONTRACT_MAP.VENDING_MINTER, minterAddress)
  context.addContractAddress(CONTRACT_MAP.SG721_BASE, collectionAddress)

  await waitForMinter(queryClient, minterAddress)

  return collectionAddress
}

export const waitForMinter = async (queryClient: CosmWasmClient, vendingMinterAddress: string) => {
  let minterConfig = await queryClient.queryContractSmart(vendingMinterAddress, {
    config: {},
  })
  await waitUntil(new Date(nanoToMs(minterConfig.start_time) + 2000))
}

export const mintNfts = async (
  context: Context,
  globalConfig: GlobalConfigForAddr,
  numNfts: number,
  recipient: TestUser,
  approveAddress?: string,
): Promise<string[]> => {
  let queryClient = await getQueryClient()
  let creator = context.getTestUser('user1')

  let vendingMinterAddress = context.getContractAddress(CONTRACT_MAP.VENDING_MINTER)
  let minterConfig = await queryClient.queryContractSmart(vendingMinterAddress, {
    config: {},
  })

  let collectionAddress = minterConfig.sg721_address

  let encodedMessages: any[] = []

  for (let i = 0; i < numNfts; i++) {
    let mintMsg = { mint: {} }
    encodedMessages.push({
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: MsgExecuteContract.fromPartial({
        sender: creator.address,
        contract: vendingMinterAddress,
        msg: toUtf8(JSON.stringify(mintMsg)),
        funds: [minterConfig.mint_price],
      }),
    })
  }

  let deliverTxResponse = await creator.client.signAndBroadcast(creator.address, encodedMessages, 'auto')

  let tokenIds: Set<string> = new Set()
  _.forEach(
    _.filter(deliverTxResponse.events, (event) => event.type === 'wasm'),
    (event) => {
      _.forEach(event.attributes, (attribute) => {
        if (attribute.key == 'token_id') {
          tokenIds.add(attribute.value)
        }
      })
    },
  )

  encodedMessages = []

  tokenIds.forEach((tokenId) => {
    let transferMsg = { transfer_nft: { recipient: recipient.address, token_id: tokenId } }
    encodedMessages.push({
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: MsgExecuteContract.fromPartial({
        sender: creator.address,
        contract: collectionAddress,
        msg: toUtf8(JSON.stringify(transferMsg)),
        funds: [],
      }),
    })
  })

  if (encodedMessages.length > 0) {
    await creator.client.signAndBroadcast(creator.address, encodedMessages, 'auto')
  }

  encodedMessages = []

  if (approveAddress) {
    tokenIds.forEach((tokenId) => {
      let approveMsg = { approve: { spender: approveAddress, token_id: tokenId } }
      encodedMessages.push({
        typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
        value: MsgExecuteContract.fromPartial({
          sender: recipient.address,
          contract: collectionAddress,
          msg: toUtf8(JSON.stringify(approveMsg)),
          funds: [],
        }),
      })
    })
  }

  if (encodedMessages.length > 0) {
    await recipient.client.signAndBroadcast(recipient.address, encodedMessages, 'auto')
  }

  return [...tokenIds]
}

export const mintNft = async (
  context: Context,
  globalConfig: GlobalConfigForAddr,
  recipient: TestUser,
  approveAddress?: string,
): Promise<string> => {
  let tokenIds = await mintNfts(context, globalConfig, 1, recipient, approveAddress)
  return tokenIds[0]
}

export const approveNft = async (
  signingClient: SigningCosmWasmClient,
  sender: string,
  collectionAddress: string,
  tokenId: string,
  approveAddress: string,
) => {
  let msg = { approve: { spender: approveAddress, token_id: tokenId } }
  let executeResult = await signingClient.execute(sender, collectionAddress, msg, 'auto', 'approve-nft')
  return executeResult
}
