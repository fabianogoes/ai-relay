import { test } from 'node:test'
import assert from 'node:assert/strict'

import {
  formatAbsolute,
  formatCalendarDate,
  formatRelative,
  statusLabel,
} from '../src/lib/presentation.ts'

test('todos os estados do protocolo recebem rotulo humano em portugues', () => {
  assert.deepEqual(
    ['backlog', 'ready', 'in_progress', 'blocked', 'done', 'idle', 'inconsistent'].map(
      statusLabel,
    ),
    ['A escolher', 'Pronto', 'Em andamento', 'Bloqueado', 'Concluído', 'Sem trabalho', 'Inconsistente'],
  )
})

test('timestamp absoluto usa data e hora no formato pt-BR', () => {
  const localTimestamp = new Date(2026, 8, 11, 13, 56).toISOString()

  assert.equal(formatAbsolute(localTimestamp), '11/09/2026, 13:56')
  assert.equal(formatAbsolute('data inválida'), 'data inválida')
})

test('tempo relativo usa palavras em portugues', () => {
  const now = Date.parse('2026-09-11T14:00:00Z')

  assert.equal(formatRelative('2026-09-11T13:55:00Z', now), 'há 5 minutos')
  assert.equal(formatRelative('2026-09-11T14:00:00Z', now), 'agora')
  assert.equal(formatRelative('data inválida', now), 'data inválida')
})

test('data civil do changelog e localizada sem deslocar o dia', () => {
  assert.equal(formatCalendarDate('2026-09-11'), '11/09/2026')
  assert.equal(formatCalendarDate('data inválida'), 'data inválida')
})
