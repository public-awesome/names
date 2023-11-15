import Context, { CONTRACT_MAP } from '../setup/context'

describe('Names Renewal', () => {
  let context: Context

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
  })

  test('is initialized', () => {
    expect(context.getContractAddress(CONTRACT_MAP.MARKETPLACE)).toBeTruthy()
    // expect(context.getContractAddress(CONTRACT_MAP.NAME_MINTER)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.SG721_NAME)).toBeTruthy()
    // expect(context.getContractAddress(CONTRACT_MAP.WHITELIST_UPDATABLE)).toBeTruthy()
  })

  test('STUB: true is true', () => {
    expect(true).toBe(true)
  })
})
