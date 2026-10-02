import type { WorkStatus } from '../types'

export type DisplayStatus = WorkStatus | 'inconsistent'

const STATUS_LABELS: Record<DisplayStatus, string> = {
  backlog: 'A escolher',
  ready: 'Pronto',
  in_progress: 'Em andamento',
  blocked: 'Bloqueado',
  done: 'Concluído',
  idle: 'Sem trabalho',
  inconsistent: 'Inconsistente',
}

const relativeFormatter = new Intl.RelativeTimeFormat('pt-BR', { numeric: 'auto' })
const absoluteFormatter = new Intl.DateTimeFormat('pt-BR', {
  dateStyle: 'short',
  timeStyle: 'short',
})
const calendarFormatter = new Intl.DateTimeFormat('pt-BR', {
  dateStyle: 'short',
  timeZone: 'UTC',
})

export function statusLabel(status: DisplayStatus): string {
  return STATUS_LABELS[status]
}

export function formatAbsolute(updated: string): string {
  const date = new Date(updated)
  return Number.isNaN(date.getTime()) ? updated : absoluteFormatter.format(date)
}

export function formatRelative(updated: string, now = Date.now()): string {
  const then = Date.parse(updated)
  if (Number.isNaN(then)) return updated

  const minutes = Math.floor((now - then) / 60_000)
  if (minutes < 1) return relativeFormatter.format(0, 'second')
  if (minutes < 60) return relativeFormatter.format(-minutes, 'minute')

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return relativeFormatter.format(-hours, 'hour')
  return relativeFormatter.format(-Math.floor(hours / 24), 'day')
}

export function formatCalendarDate(value: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) return value

  const date = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])))
  return calendarFormatter.format(date)
}
