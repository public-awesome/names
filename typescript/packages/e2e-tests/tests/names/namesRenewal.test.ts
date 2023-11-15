import Context from '../setup/context'

describe('Names Renewal', () => {
  let context: Context

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
  });

  test('STUB: true is true', () => {
    expect(true).toBe(true)
  })
})
