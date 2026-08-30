import { randomUUID } from 'node:crypto'
import { spawn } from 'node:child_process'
import { isAbsolute } from 'node:path'

const PROTOCOL_VERSION = 1
const PLAN_METHOD = 'planRemainingWork'
const MAX_MESSAGE_BYTES = 64 * 1024
const MAX_FOCUS_ITEMS = 3
const MAX_ID_LENGTH = 128
const HOST_TIMEOUT_MS = 10_000

export function buildPlanningEnvelope(planningRequest, requestId = randomUUID()) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    method: PLAN_METHOD,
    params: planningRequest,
  }
}

export function buildHostSpawnOptions() {
  return {
    env: {},
    shell: false,
    stdio: ['pipe', 'pipe', 'pipe'],
    windowsHide: true,
  }
}

function isValidId(value) {
  return typeof value === 'string' && value.length > 0 && value.length <= MAX_ID_LENGTH
}

export function parsePlanningResponse(text, expectedRequestId) {
  let response
  try {
    response = JSON.parse(text)
  } catch {
    throw new Error('B.O.B. capability host returned invalid JSON')
  }

  if (response?.protocolVersion !== PROTOCOL_VERSION) {
    throw new Error('B.O.B. capability host returned an unsupported protocol version')
  }
  if (response?.requestId !== expectedRequestId) {
    throw new Error('B.O.B. capability host response requestId did not match the request')
  }
  if (response?.ok !== true) {
    const code = response?.error?.code ?? 'unknown_error'
    const message = response?.error?.message ?? 'request failed'
    throw new Error(`B.O.B. capability host rejected request (${code}): ${message}`)
  }

  const result = response.result
  if (!result || typeof result !== 'object' || !Array.isArray(result.focusIds)) {
    throw new Error('B.O.B. capability host returned an invalid planning result')
  }

  const { nextId, focusIds } = result
  if (
    (nextId !== null && !isValidId(nextId))
    || focusIds.length > MAX_FOCUS_ITEMS
    || !focusIds.every(isValidId)
    || new Set(focusIds).size !== focusIds.length
    || (focusIds.length === 0 && nextId !== null)
    || (focusIds.length > 0 && nextId !== focusIds[0])
  ) {
    throw new Error('B.O.B. capability host returned an invalid planning result')
  }

  return result
}

export async function runPlanningRequest(hostPath, planningRequest, signal) {
  if (typeof hostPath !== 'string' || !isAbsolute(hostPath)) {
    throw new Error('B.O.B. hostPath must be an absolute executable path')
  }

  const envelope = buildPlanningEnvelope(planningRequest)
  const serializedEnvelope = `${JSON.stringify(envelope)}\n`
  if (Buffer.byteLength(serializedEnvelope, 'utf8') > MAX_MESSAGE_BYTES) {
    throw new Error('B.O.B. planning request exceeded 64 KiB')
  }

  const child = spawn(hostPath, [], buildHostSpawnOptions())

  let stdout = ''
  let stderr = ''
  let settled = false
  let timeout

  return new Promise((resolve, reject) => {
    const finish = (fn, value) => {
      if (settled) return
      settled = true
      clearTimeout(timeout)
      signal?.removeEventListener('abort', onAbort)
      fn(value)
    }

    const fail = (error) => {
      if (!child.killed) child.kill()
      finish(reject, error)
    }

    const onAbort = () => fail(new Error('B.O.B. planning request was cancelled'))

    if (signal?.aborted) {
      onAbort()
      return
    }
    signal?.addEventListener('abort', onAbort, { once: true })

    timeout = setTimeout(
      () => fail(new Error('B.O.B. capability host timed out after 10 seconds')),
      HOST_TIMEOUT_MS,
    )

    child.on('error', (error) => fail(new Error(`Unable to start B.O.B. capability host: ${error.message}`)))
    child.stdin.on('error', (error) => {
      if (!settled) fail(new Error(`Unable to write to B.O.B. capability host: ${error.message}`))
    })

    child.stdout.setEncoding('utf8')
    child.stdout.on('data', (chunk) => {
      stdout += chunk
      if (Buffer.byteLength(stdout, 'utf8') > MAX_MESSAGE_BYTES) {
        fail(new Error('B.O.B. capability host response exceeded 64 KiB'))
      }
    })

    child.stderr.setEncoding('utf8')
    child.stderr.on('data', (chunk) => {
      if (Buffer.byteLength(stderr, 'utf8') < 4096) stderr += chunk
    })

    child.on('close', (code) => {
      if (settled) return
      if (code !== 0) {
        const detail = stderr.trim()
        finish(
          reject,
          new Error(`B.O.B. capability host exited with code ${code}${detail ? `: ${detail}` : ''}`),
        )
        return
      }

      try {
        finish(resolve, parsePlanningResponse(stdout.trim(), envelope.requestId))
      } catch (error) {
        finish(reject, error)
      }
    })

    child.stdin.end(serializedEnvelope)
  })
}
