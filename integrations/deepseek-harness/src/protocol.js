import { randomUUID } from 'node:crypto'
import { spawn } from 'node:child_process'
import { isAbsolute } from 'node:path'

const PROTOCOL_VERSION = 1
const PLAN_METHOD = 'planRemainingWork'
const MAX_RESPONSE_BYTES = 64 * 1024

export function buildPlanningEnvelope(planningRequest, requestId = randomUUID()) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    method: PLAN_METHOD,
    params: planningRequest,
  }
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
  if (!response.result || !Array.isArray(response.result.focusIds)) {
    throw new Error('B.O.B. capability host returned an invalid planning result')
  }

  return response.result
}

export async function runPlanningRequest(hostPath, planningRequest, signal) {
  if (typeof hostPath !== 'string' || !isAbsolute(hostPath)) {
    throw new Error('B.O.B. hostPath must be an absolute executable path')
  }

  const envelope = buildPlanningEnvelope(planningRequest)
  const child = spawn(hostPath, [], {
    shell: false,
    stdio: ['pipe', 'pipe', 'pipe'],
    windowsHide: true,
  })

  let stdout = ''
  let stderr = ''
  let settled = false

  return new Promise((resolve, reject) => {
    const finish = (fn, value) => {
      if (settled) return
      settled = true
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

    child.on('error', (error) => fail(new Error(`Unable to start B.O.B. capability host: ${error.message}`)))

    child.stdout.setEncoding('utf8')
    child.stdout.on('data', (chunk) => {
      stdout += chunk
      if (Buffer.byteLength(stdout, 'utf8') > MAX_RESPONSE_BYTES) {
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

    child.stdin.end(`${JSON.stringify(envelope)}\n`)
  })
}
