import { test } from 'node:test'
import assert from 'node:assert/strict'

import { readTerminalTheme } from '../src/lib/terminal-theme.ts'

test('tema do terminal consome e normaliza as custom properties do design system', () => {
  const values = new Map([
    ['--bg-deep', ' #070a0d '],
    ['--ink', ' #e9edf3 '],
    ['--green', ' #5fe3b3 '],
  ])

  const theme = readTerminalTheme({
    getPropertyValue: (name: string) => values.get(name) ?? '',
  })

  assert.deepEqual(theme, {
    background: '#070a0d',
    foreground: '#e9edf3',
    cursor: '#5fe3b3',
  })
})
