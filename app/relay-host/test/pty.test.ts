import { test } from 'node:test'
import assert from 'node:assert/strict'
import { tmpdir } from 'node:os'

import { ptyAvailable, startPtyRun } from '../src/pty.ts'

test('ptyAvailable declara macOS como plataforma suportada', () => {
  assert.equal(ptyAvailable(), process.platform === 'darwin')
})

test('PTY real: envia entrada, recebe saída e obtém o exit code correto (A-001)', { skip: !ptyAvailable() }, async () => {
  const handle = startPtyRun({ bin: '/bin/sh', args: [], cwd: tmpdir() })

  let output = ''
  handle.onData((chunk) => {
    output += chunk
  })

  const exitCode = await new Promise<number | null>((resolve) => {
    handle.onExit((code) => resolve(code))
    handle.write('echo echo-de-volta; exit 7\r')
  })

  assert.equal(exitCode, 7)
  assert.match(output, /echo-de-volta/)
  assert.equal(handle.status(), 'exited')
  assert.equal(handle.exitCode(), 7)
})
