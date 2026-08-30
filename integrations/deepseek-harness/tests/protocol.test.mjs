import assert from 'node:assert/strict'
import test from 'node:test'
import { buildPlanningEnvelope, parsePlanningResponse } from '../src/protocol.js'

test('builds the bounded B.O.B. planning envelope without extra host authority', () => {
  const params = {
    activeId: 'current',
    items: [{ id: 'current', kind: 'task', priority: 'normal', due: null, status: 'planned' }],
  }

  const envelope = buildPlanningEnvelope(params, 'request-1')

  assert.deepEqual(envelope, {
    protocolVersion: 1,
    requestId: 'request-1',
    method: 'planRemainingWork',
    params,
  })
  assert.equal('shell' in envelope, false)
  assert.equal('filesystem' in envelope, false)
  assert.equal('credentials' in envelope, false)
})

test('accepts only the matching successful typed response', () => {
  const result = parsePlanningResponse(JSON.stringify({
    protocolVersion: 1,
    requestId: 'request-1',
    ok: true,
    result: { nextId: 'current', focusIds: ['current'] },
  }), 'request-1')

  assert.deepEqual(result, { nextId: 'current', focusIds: ['current'] })

  assert.throws(
    () => parsePlanningResponse(JSON.stringify({
      protocolVersion: 1,
      requestId: 'other',
      ok: true,
      result: { nextId: null, focusIds: [] },
    }), 'request-1'),
    /requestId did not match/,
  )
})
