import type { RawEntry, RawHandoff, ChangelogRecord } from './parse.ts'
import { parseSpecCriteria } from './parse.ts'
import type { Violation } from './types.ts'

export interface IntegrityInput {
  handoff: RawHandoff | null
  handoffCount: number
  activeBacklogId: string | null
  todo: RawEntry[]
  backlog: RawEntry[]
  changelog: ChangelogRecord[]
  specs: Record<string, string>
}

const HARNESS_RE = /^[a-z0-9][a-z0-9._-]*$/
const RFC3339_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})$/
const KNOWN_MARKERS = new Set([' ', '•', '!', 'x'])

function violation(check: string, detail: string, records: string[]): Violation {
  return { check, detail, records }
}

function checkHandoffNamesNoPendingTodo({ handoff, todo }: IntegrityInput): Violation | null {
  if (!handoff) return null
  const named = todo.find((e) => e.id === handoff.todoId)
  if (named && named.marker !== 'x') return null
  if (named) {
    const inProgress = todo.find((e) => e.marker === '•')
    return violation(
      'handoff-names-no-pending-todo',
      `O handoff aponta ${handoff.todoId}, que está [x]; ${inProgress ? inProgress.id : '?'} está [•].`,
      ['handoff', 'todo'],
    )
  }
  return violation(
    'handoff-names-no-pending-todo',
    `O handoff aponta ${handoff.todoId}, que não existe no TODO.`,
    ['handoff', 'todo'],
  )
}

function checkBacklogIdMismatch({ handoff, activeBacklogId, backlog }: IntegrityInput): Violation | null {
  if (!handoff && !activeBacklogId) return null
  if (activeBacklogId && !backlog.some((e) => e.id === activeBacklogId)) {
    return violation(
      'backlog-id-mismatch',
      `O backlog ativo ${activeBacklogId} não existe no backlog.`,
      ['handoff', 'todo', 'backlog'],
    )
  }
  if (!handoff) return null
  if (handoff.backlogId !== activeBacklogId) {
    return violation(
      'backlog-id-mismatch',
      `O handoff diz ${handoff.backlogId}, o TODO diz ${activeBacklogId ?? 'nenhum'}.`,
      ['handoff', 'todo', 'backlog'],
    )
  }
  return null
}

function checkSpecPathMismatch({ handoff, activeBacklogId, backlog }: IntegrityInput): Violation | null {
  if (!handoff) return null
  const entry = backlog.find((e) => e.id === activeBacklogId)
  if (!entry) {
    return violation(
      'spec-path-mismatch',
      `A tarefa ativa ${activeBacklogId ?? '?'} não tem entrada de backlog confrontável.`,
      ['handoff', 'backlog'],
    )
  }
  if (!handoff.spec) {
    return violation('spec-path-mismatch', 'O handoff não tem Spec.', ['handoff'])
  }
  if (!entry.spec) {
    return violation('spec-path-mismatch', `A entrada ${entry.id} não tem spec.`, ['backlog'])
  }
  if (entry.spec !== handoff.spec) {
    return violation(
      'spec-path-mismatch',
      `O handoff diz ${handoff.spec}, a tarefa diz ${entry.spec}.`,
      ['handoff', 'backlog'],
    )
  }
  return null
}

function checkHandoffHarness({ handoff }: IntegrityInput): Violation | null {
  if (!handoff) return null
  if (!handoff.harness || !HARNESS_RE.test(handoff.harness)) {
    return violation('handoff-harness-invalid', `Harness inválido: "${handoff.harness}".`, ['handoff'])
  }
  return null
}

function checkHandoffUpdated({ handoff }: IntegrityInput): Violation | null {
  if (!handoff) return null
  if (!handoff.updated || !RFC3339_RE.test(handoff.updated) || Number.isNaN(Date.parse(handoff.updated))) {
    return violation('handoff-updated-invalid', `Updated inválido: "${handoff.updated}".`, ['handoff'])
  }
  return null
}

function checkMultipleHandoffs({ handoffCount }: IntegrityInput): Violation | null {
  if (handoffCount > 1) {
    return violation('multiple-handoffs', `Existem ${handoffCount} registros de handoff.`, ['handoff'])
  }
  return null
}

function checkTodoClearedBeforeChangelog({ activeBacklogId, todo, changelog }: IntegrityInput): Violation | null {
  for (const entry of todo) {
    if (entry.marker !== 'x') continue
    const has = changelog.some((r) => r.backlogId === activeBacklogId && r.todoId === entry.id)
    if (!has) {
      return violation(
        'todo-cleared-before-changelog',
        `A subtarefa ${entry.id} está [x] sem registro no changelog.`,
        ['handoff', 'changelog'],
      )
    }
  }
  return null
}

function checkBacklogDoneWithPendingTodo({ activeBacklogId, backlog, todo }: IntegrityInput): Violation | null {
  if (!activeBacklogId) return null
  const entry = backlog.find((e) => e.id === activeBacklogId)
  if (entry && entry.marker === 'x' && todo.some((e) => e.marker !== 'x')) {
    return violation(
      'backlog-done-with-pending-todo',
      `Backlog ${activeBacklogId} está done com TODO pendente.`,
      ['backlog', 'todo'],
    )
  }
  return null
}

function checkUnknownMarker({ todo, backlog }: IntegrityInput): Violation | null {
  for (const entry of [...todo, ...backlog]) {
    if (!KNOWN_MARKERS.has(entry.marker)) {
      return violation('unknown-marker', `Marcador desconhecido "[${entry.marker}]" em ${entry.id}.`, ['todo', 'backlog'])
    }
  }
  return null
}

function checkNeedsUnknownId({ todo, backlog }: IntegrityInput): Violation | null {
  const records: Array<[string, RawEntry[]]> = [
    ['todo', todo],
    ['backlog', backlog],
  ]
  for (const [name, entries] of records) {
    const ids = new Set(entries.map((e) => e.id))
    for (const entry of entries) {
      for (const need of entry.needs) {
        if (!ids.has(need)) {
          return violation('needs-unknown-id', `${entry.id} referencia ${need}, ausente em ${name}.`, [name])
        }
      }
    }
  }
  return null
}

function checkNeedsCycle({ todo, backlog }: IntegrityInput): Violation | null {
  const records: Array<[string, RawEntry[]]> = [
    ['todo', todo],
    ['backlog', backlog],
  ]
  for (const [name, entries] of records) {
    const byId = new Map(entries.map((e) => [e.id, e]))
    const color = new Map<string, number>()
    const visit = (id: string, trail: string[]): string[] | null => {
      color.set(id, 1)
      const entry = byId.get(id)
      if (entry) {
        for (const need of entry.needs) {
          const c = color.get(need) ?? 0
          if (c === 1) return [...trail, id, need]
          if (c === 0) {
            const found = visit(need, [...trail, id])
            if (found) return found
          }
        }
      }
      color.set(id, 2)
      return null
    }
    for (const id of byId.keys()) {
      if ((color.get(id) ?? 0) === 0) {
        const cycle = visit(id, [])
        if (cycle) {
          return violation('needs-cycle', `Ciclo em ${name}: ${cycle.join(' -> ')}.`, [name])
        }
      }
    }
  }
  return null
}

function checkNeedsIncompleteOnDone({ todo, backlog }: IntegrityInput): Violation | null {
  const records: Array<[string, RawEntry[]]> = [
    ['todo', todo],
    ['backlog', backlog],
  ]
  for (const [name, entries] of records) {
    const byId = new Map(entries.map((e) => [e.id, e]))
    for (const entry of entries) {
      if (entry.marker !== 'x') continue
      for (const need of entry.needs) {
        const dep = byId.get(need)
        if (!dep || dep.marker !== 'x') {
          return violation(
            'needs-incomplete-on-done',
            `${entry.id} está [x] mas precisa de ${need}, não concluído.`,
            [name],
          )
        }
      }
    }
  }
  return null
}

function specPrefix(path: string): string {
  const m = path.match(/(\d{8}-\d{3})/)
  return m ? m[1] : ''
}

function checkCriteriaWithoutEvidence({ backlog, changelog, specs }: IntegrityInput): Violation | null {
  const bySpec = new Map<string, RawEntry[]>()
  for (const entry of backlog) {
    if (!entry.spec) continue
    const list = bySpec.get(entry.spec) ?? []
    list.push(entry)
    bySpec.set(entry.spec, list)
  }
  for (const [specPath, entries] of bySpec) {
    if (!entries.every((e) => e.marker === 'x')) continue
    const content = specs[specPath]
    if (!content) continue
    const criteria = parseSpecCriteria(content)
    if (criteria.length === 0) continue
    const prefix = specPrefix(specPath)
    const missing = criteria.filter(
      (c) =>
        !changelog.some(
          (r) => (r.spec === specPath && r.criteria.includes(c)) || (prefix !== '' && r.criteria.includes(`${prefix}/${c}`)),
        ),
    )
    if (missing.length > 0) {
      return violation(
        'criteria-without-evidence',
        `Spec ${specPath}: critérios sem evidência: ${missing.join(', ')}.`,
        ['backlog', 'changelog', 'spec'],
      )
    }
  }
  return null
}

type Check = (input: IntegrityInput) => Violation | null

const CHECKS: Check[] = [
  checkHandoffNamesNoPendingTodo,
  checkBacklogIdMismatch,
  checkSpecPathMismatch,
  checkHandoffHarness,
  checkHandoffUpdated,
  checkMultipleHandoffs,
  checkTodoClearedBeforeChangelog,
  checkBacklogDoneWithPendingTodo,
  checkUnknownMarker,
  checkNeedsUnknownId,
  checkNeedsCycle,
  checkNeedsIncompleteOnDone,
  checkCriteriaWithoutEvidence,
]

export function runIntegrityChecks(input: IntegrityInput): Violation[] {
  const violations: Violation[] = []
  for (const check of CHECKS) {
    const result = check(input)
    if (result) violations.push(result)
  }
  return violations
}
