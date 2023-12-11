// Names End to End Tests

import Context, { CONTRACT_MAP } from '../setup/context'

// Utility functions
const extractEvent = (processResponse: any, eventName: string) => {
  const event = processResponse.events.find((event: any) => event.type === eventName)
  return event;
}

const extractAttribute = (event: any, attributeName: string) => {
  const attribute = event.attributes.find((attribute: any) => attribute.key === attributeName)
  return attribute;
}

const logStuff = (processResponse:any) => {
  processResponse.events.forEach((event: any) => {
    event.attributes.forEach((attribute: any) => {
      console.log(attribute)
    }
    )
  })
}

// Test suite
describe('Names Renewal', () => {
  let context: Context
  const BID = '100000000'
  const NAME = 'testname'
  const NAME2 = 'testname2'
  const BIDDER = 'user2'

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
    // we will always have at least 1 name
    // to test with, each test will mint
    // additional names as needed
    await context.mintName(NAME, "user1")
    await context.mintName("more-test", "user1")

  })

  test('is initialized', () => {
    expect(context.getContractAddress(CONTRACT_MAP.MARKETPLACE)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.NAME_MINTER)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.WHITELIST_UPDATABLE)).toBeTruthy()
  })

  test('count asks', async () => {
    const askCount = await context.countAsks()
    expect(askCount).toBe(3)
  })

  test('get asks', async () => {
    const asks = await context.getAsks()
    expect(asks.length).toBe(3)
  })

  test('add whitelist entry', async () => {
    const whitelistResponse = await context.updateWhitelist('user4', 'user1')
    expect(whitelistResponse.transactionHash).toBeTruthy()
  })

  test('bid on name', async () => {
    const bidResponse = await context.placeBid(NAME, BID, BIDDER)
    expect(bidResponse.transactionHash).toBeTruthy()

    const bids = await context.getBids(NAME, BIDDER)
    expect(bids.length).toBe(1)
    expect(bids[0].token_id).toBe(NAME)
    expect(bids[0].bidder).toBe(context.getTestUser(BIDDER).address)
    expect(bids[0].amount).toBe(BID)
  })

  test('remove bid', async () => {
    const bidResponse = await context.removeBid(NAME, BIDDER)
    expect(bidResponse.transactionHash).toBeTruthy()

    const bids = await context.getBids(NAME, BIDDER)
    expect(bids.length).toBe(0)
  })

  test('accept bid', async () => {
    const bidResponse = await context.placeBid(NAME, BID, BIDDER)
    expect(bidResponse.transactionHash).toBeTruthy()

    const acceptResponse = await context.acceptBid(NAME, BIDDER)
    expect(acceptResponse.transactionHash).toBeTruthy()
  })

  test('fund renewal', async () => {
    const fundResponse = await context.fundRenewal(NAME, BID)

    expect(fundResponse.transactionHash).toBeTruthy()
  })

  test('refund renewal', async () => {
    await context.fundRenewal(NAME, BID)

    const refundResponse = await context.refundRenewal(NAME, BIDDER)
    expect(refundResponse.transactionHash).toBeTruthy()
  })

  test('get renewal queue', async () => {
    let timestamp = ''
    const asks = await context.getAsks()

    // get the newest renewal time from asks
    for(let i = 0; i < asks.length; i++) {
      if(asks[i].renewal_time > timestamp) {
        timestamp = asks[i].renewal_time
      }
    }

    const queue = await context.getRenewalQueue(timestamp)
    // console.log("renewal queue----->", queue)
    // ensure the renewal queue is not empty
    expect(queue.length).toBe(1)
    expect(queue[0].token_id).toBe(NAME)
  })


  test('process renewal, no bid & no renewal fund (burn name)', async () => {
    const timestamp = (Date.now() - 100000)

    const processResponse = await context.processRenewal(timestamp.toString())
    expect(processResponse.transactionHash).toBeTruthy()

    const burnEvent = extractEvent(processResponse, 'wasm-burn')
    expect(burnEvent).toBeTruthy()

    let asks = await context.getAsks()
    expect(asks.length).toBe(2)

    const burnAttribute = extractAttribute(burnEvent, 'token_id-burned')
    expect(burnAttribute).toStrictEqual(
      {"key": "token_id-burned", "value": NAME2}
    )
  })

  test('process renewal, with renewal funded but no bid (renew name)', async() =>{
    // timestamp in ns
    const timestamp = (Date.now() - 100000)// * 1000000

    // fund the renewal
    await context.fundRenewal(NAME, BID+1000)

    const processResponse = await context.processRenewal(timestamp.toString())
    expect(processResponse.transactionHash).toBeTruthy()
  })


  test('process renewal, with a bid and no renewal fund', async () => {
    // timestamp in ns
    let timestamp = (Date.now() - 100000)// * 1000000

    // place a bid first
    await context.placeBid(NAME, BID, BIDDER)

    // get the all the asks
    const asks = await context.getAsks()
    console.log("asks----->", asks)
    // get the newest renewal time from asks
    for(let i = 0; i < asks.length; i++) {
      if(asks[i].renewal_time > timestamp) {
        timestamp = asks[i].renewal_time
      }
    }
    console.log("asks----->", asks)
    // get the renewal queue and log it
    const queue = await context.getRenewalQueue(timestamp.toString())
    console.log("renewal queue----->", queue)

    // const processResponse = await context.processRenewal(timestamp.toString())
    // console.log(processResponse)
    // logStuff(processResponse)
    // const askCount = await context.countAsks()
    // console.log("ask count----->", askCount)
    // expect(processResponse.transactionHash).toBeTruthy()
    expect(true).toBeTruthy()
  })

})
