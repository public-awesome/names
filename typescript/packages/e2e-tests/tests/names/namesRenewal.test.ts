import Context, { CONTRACT_MAP } from '../setup/context'

describe('Names Renewal', () => {
  let context: Context
  const BID = '100000000'
  const NAME = 'testname'
  const BIDDER = 'user2'

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
  })

  test('is initialized', () => {
    expect(context.getContractAddress(CONTRACT_MAP.MARKETPLACE)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.NAME_MINTER)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.WHITELIST_UPDATABLE)).toBeTruthy()
  })

  test('count asks', async () => {
    const askCount = await context.countAsks()
    expect(askCount).toBe(1)
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

  test('process renewal', async () => {
    const timestamp = new Date().toISOString()

    const processResponse = await context.processRenewal(timestamp)
    console.log(processResponse)
    expect(processResponse.transactionHash).toBeTruthy()
  })

  test('STUB: true is true', () => {
    expect(true).toBe(true)
  })
})
