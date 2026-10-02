import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, realpathSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { closeRun, getRun, listActiveRuns, startExec } from '../src/executor.ts'
import { ptyAvailable } from '../src/pty.ts'
import type { LaunchPlan } from '../src/harness.ts'

test(
  'startExec: prompt vai como último elemento do argv e o processo roda no cwd do plano (A-002)',
  { skip: !ptyAvailable() },
  async () => {
    const dir = mkdtempSync(join(tmpdir(), 'relay-executor-test-'))
    const cwd = realpathSync(dir)
    try {
      const plan: LaunchPlan = {
        bin: '/bin/sh',
        args: ['-c', 'printf "CWD=%s\\n" "$PWD"; printf "ARG0=%s\\n" "$0"'],
        prompt: '/relay-session minha intenção',
        cwd,
      }

      const run = startExec(plan, 'claude-code', 'Claude Code')
      assert.ok(run, 'startExec deveria retornar uma run com PTY disponível')
      assert.equal(run!.harnessId, 'claude-code')
      assert.equal(run!.harnessName, 'Claude Code')

      let output = ''
      const exitCode = await new Promise<number | null>((resolve) => {
        run!.handle.onData((chunk) => (output += chunk))
        run!.handle.onExit((code) => resolve(code))
      })

      assert.equal(exitCode, 0)
      assert.match(output, new RegExp(`CWD=${cwd.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`))
      assert.match(output, /ARG0=\/relay-session minha intenção/)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  },
)

test(
  'run concluída continua endereçável até fechamento explícito (A-006/A-007)',
  { skip: !ptyAvailable() },
  async () => {
    const dir = mkdtempSync(join(tmpdir(), 'relay-executor-test-'))
    const cwd = realpathSync(dir)
    try {
      const plan: LaunchPlan = { bin: '/bin/sh', args: ['-c', 'exit 0'], prompt: '', cwd }
      const run = startExec(plan, 'claude-code', 'Claude Code')
      assert.ok(run)

      await new Promise<void>((resolve) => run!.handle.onExit(() => resolve()))

      // saiu, mas continua no registro — não sumiu sozinha do getRun
      assert.ok(getRun(run!.runId), 'run deveria continuar endereçável após sair')
      // não aparece mais entre as runs ativas (já não está rodando)
      assert.ok(!listActiveRuns().some((r) => r.runId === run!.runId))

      assert.equal(closeRun(run!.runId), true)
      assert.equal(getRun(run!.runId), null, 'closeRun deveria remover a run do registro')
      assert.equal(closeRun(run!.runId), false, 'fechar de novo uma run já fechada devolve false')
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  },
)
