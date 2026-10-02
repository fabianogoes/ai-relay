import { spawn } from 'node:child_process'
import { chmodSync, existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { randomUUID } from 'node:crypto'
import { buildLaunchArgv, type LaunchPlan, type RelaySkill } from './harness.ts'

export interface LaunchRequest {
  harness: string
  skill: RelaySkill
  intent: string
}

export interface LaunchResult {
  runId: string
  scratchDir: string
  plan: LaunchPlan
}

export interface LaunchOptions {
  openExternal?: (scriptPath: string) => boolean
}

const TERMINAL_CANDIDATES = ['Terminal', 'iTerm', 'Ghostty', 'Warp']

export function shellQuote(arg: string): string {
  return `'${arg.replace(/'/g, `'\\''`)}'`
}

export function detectTerminal(): string | null {
  if (process.platform !== 'darwin') return null
  return TERMINAL_CANDIDATES.find((name) => existsSync(join('/Applications', `${name}.app`))) ?? null
}

export function buildWrapperScript(argv: string[], cwd: string, pidPath: string, exitPath: string): string {
  return [
    '#!/bin/sh',
    'umask 077',
    `cd ${shellQuote(cwd)} || exit 1`,
    `echo $$ > ${shellQuote(pidPath)}`,
    argv.map(shellQuote).join(' '),
    `echo $? > ${shellQuote(exitPath)}`,
    '',
  ].join('\n')
}

export function scratchDir(runId: string): string {
  return join(tmpdir(), 'relay-run', runId)
}

export function preview(request: LaunchRequest, cwd: string): LaunchPlan {
  return buildLaunchArgv(request.harness, request.skill, request.intent, cwd)
}

function defaultOpenExternal(scriptPath: string): boolean {
  const terminal = detectTerminal()
  if (!terminal) return false
  spawn('open', ['-a', terminal, scriptPath], { detached: true, stdio: 'ignore' }).unref()
  return true
}

export function launch(request: LaunchRequest, cwd: string, options: LaunchOptions = {}): LaunchResult {
  const plan = buildLaunchArgv(request.harness, request.skill, request.intent, cwd)
  const runId = randomUUID()
  const dir = scratchDir(runId)
  mkdirSync(dir, { recursive: true, mode: 0o700 })

  const argv = [plan.bin, ...plan.args, plan.prompt]
  const pidPath = join(dir, 'pid')
  const exitPath = join(dir, 'exit')
  const scriptPath = join(dir, 'launch.sh')
  writeFileSync(scriptPath, buildWrapperScript(argv, plan.cwd, pidPath, exitPath))
  chmodSync(scriptPath, 0o700)

  const openExternal = options.openExternal ?? defaultOpenExternal
  if (!openExternal(scriptPath)) {
    throw new Error('Nenhum terminal disponível para abrir o modo externo.')
  }

  return { runId, scratchDir: dir, plan }
}
