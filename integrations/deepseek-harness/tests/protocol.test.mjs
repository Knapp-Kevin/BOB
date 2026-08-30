import assert from 'node:assert/strict'
import { resolve } from 'node:path'
import test from 'node:test'
import {
  buildHostSpawnOptions,
  buildPlanningEnvelope,
  parsePlanningResponse,
  runPlanningRequest,
} from '../src/protocol.js'

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

test('spawns the B.O.B. sidecar without inheriting the DeepSeek Harness environment', () => {
  const options = buildHostSpawnOptions()

  assert.deepEqual(options.env, {})
  assert.equal(options.shell, false)
  assert.deepEqual(options.stdio, ['pipe', 'pipe', 'pipe'])
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

test('rejects malformed planning results even when the envelope claims success', () => {
  const invalidResults = [
    { nextId: 7, focusIds: [] },
    { nextId: 'one', focusIds: ['one', 'two', 'three', 'four'] },
    { nextId: 'one', focusIds: ['one', 2] },
    { nextId: 'one', focusIds: ['two'] },
    { nextId: null, focusIds: ['one'] },
    { nextId: 'one', focusIds: ['one', 'one'] },
  ]

  for (const result of invalidResults) {
    assert.throws(
      () => parsePlanningResponse(JSON.stringify({
        protocolVersion: 1,
        requestId: 'request-1',
        ok: true,
        result,
      }), 'request-1'),
      /invalid planning result/,
    )
  }
})

test('rejects oversized requests before attempting to spawn a sidecar', async () => {
  const impossibleHost = resolve('definitely-does-not-exist-bob-capability-host')
  const oversizedRequest = {
    activeId: null,
    items: [{
      id: 'x'.repeat(70_000),
      kind: 'task',
      priority: 'normal',
      due: null,
      status: 'planned',
    }],
  }

  await assert.rejects(
    runPlanningRequest(impossibleHost, oversizedRequest),
    /planning request exceeded 64 KiB/,
  )
})
