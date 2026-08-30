import { defineTool } from '@deepseek-ai/dsh-tools'
import { runPlanningRequest } from './protocol.js'

export const name = 'bob-deepseek-host-tracer'
export const inject = ['tools']

export function apply(ctx, config = {}) {
  const hostPath = config.hostPath
  if (typeof hostPath !== 'string' || hostPath.trim().length === 0) {
    throw new Error('bob-deepseek-host-tracer requires config.hostPath')
  }

  ctx.tools.register(defineTool({
    name: 'bob_plan_remaining_work',
    description: 'Ask B.O.B. to deterministically choose the next action and bounded focus set from supplied work state.',
    parameters: {
      activeId: {
        type: 'string',
        description: 'Current active work item id when one exists.',
      },
      items: {
        type: 'array',
        required: true,
        description: 'Planning-relevant work items. B.O.B. receives no title, notes, credentials, filesystem state, or harness session transcript.',
        items: {
          type: 'object',
          additionalProperties: false,
          properties: {
            id: { type: 'string', required: true },
            kind: { type: 'string', required: true },
            priority: { type: 'string', required: true },
            due: { type: 'string' },
            status: { type: 'string', required: true },
          },
        },
      },
    },
    output: {
      schema: {
        type: 'object',
        additionalProperties: false,
        properties: {
          nextId: { type: 'string' },
          focusIds: { type: 'array', items: { type: 'string' }, required: true },
        },
      },
      render: (_args, value) => [{
        type: 'text',
        text: `B.O.B. next action: ${value.nextId ?? 'none'}; focus: ${value.focusIds.join(', ') || 'none'}`,
      }],
    },
    async execute(args, exec) {
      const result = await runPlanningRequest(hostPath, {
        activeId: args.activeId ?? null,
        items: args.items.map((item) => ({
          id: item.id,
          kind: item.kind,
          priority: item.priority,
          due: item.due ?? null,
          status: item.status,
        })),
      }, exec.signal)

      return {
        ...(result.nextId === null ? {} : { nextId: result.nextId }),
        focusIds: result.focusIds,
      }
    },
  }))
}
