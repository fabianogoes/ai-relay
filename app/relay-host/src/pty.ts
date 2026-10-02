import * as pty from 'node-pty'
import { randomUUID } from 'node:crypto'

export type RunStatus = 'running' | 'exited'

export interface RunInfo {
  runId: string
  status: RunStatus
  exitCode: number | null
  startedAt: string
  harnessId: string
  harnessName: string
  processName: string
}

export interface PtyHandle {
  runId: string
  status(): RunStatus
  exitCode(): number | null
  startedAt(): string
  write(data: string): void
  terminate(): void
  scrollback(): string
  onData(cb: (chunk: string) => void): () => void
  onExit(cb: (code: number | null) => void): () => void
}

export function ptyAvailable(): boolean {
  return process.platform === 'darwin'
}

export interface StartRunOptions {
  bin: string
  args: string[]
  cwd: string
}

export function startPtyRun(options: StartRunOptions): PtyHandle {
  const runId = randomUUID()
  const startedAt = new Date().toISOString()

  let scrollback = ''
  let exited = false
  let exitCode: number | null = null
  const dataCbs = new Set<(chunk: string) => void>()
  const exitCbs = new Set<(code: number | null) => void>()

  const term = pty.spawn(options.bin, options.args, {
    name: 'xterm-256color',
    cols: 80,
    rows: 24,
    cwd: options.cwd,
    env: process.env as Record<string, string>,
  })

  term.onData((chunk) => {
    scrollback += chunk
    for (const cb of dataCbs) cb(chunk)
  })

  term.onExit(({ exitCode: code }) => {
    exited = true
    exitCode = code
    for (const cb of exitCbs) cb(code)
  })

  return {
    runId,
    status: () => (exited ? 'exited' : 'running'),
    exitCode: () => exitCode,
    startedAt: () => startedAt,
    write(data: string): void {
      if (!exited) term.write(data)
    },
    terminate(): void {
      if (!exited) term.kill('SIGHUP')
    },
    scrollback: () => scrollback,
    onData(cb) {
      dataCbs.add(cb)
      return () => dataCbs.delete(cb)
    },
    onExit(cb) {
      exitCbs.add(cb)
      return () => exitCbs.delete(cb)
    },
  }
}
