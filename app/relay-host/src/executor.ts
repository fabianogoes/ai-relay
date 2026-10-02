import { startPtyRun, ptyAvailable, type PtyHandle } from './pty.ts'
import { startDiskTracker, type DiskTracker, type DiskEntry } from './disk.ts'
import type { LaunchPlan } from './harness.ts'

export interface EmbeddedRun {
  runId: string
  plan: LaunchPlan
  handle: PtyHandle
  harnessId: string
  harnessName: string
  processName: string
  disk: DiskTracker
}

interface ExecutorEvents {
  runStarted(run: EmbeddedRun): void
  runExited(run: EmbeddedRun, code: number | null): void
}

// Guarda runs vivas e runs já saídas ainda não fechadas explicitamente — uma
// run concluída continua endereçável (info, disk, scrollback) até `closeRun`,
// nunca some sozinha ao processo sair (ver Decisions da spec 012).
const registry = new Set<EmbeddedRun>()

export const executorEvents: ExecutorEvents = {
  runStarted(_run: EmbeddedRun): void {},
  runExited(_run: EmbeddedRun, _code: number | null): void {},
}

function processName(plan: LaunchPlan): string {
  const skill = plan.prompt.match(/\/(\w[\w-]*)/)?.[1] ?? plan.prompt.split(/\s+/)[0] ?? plan.bin
  return `${plan.bin} · ${skill}`
}

export function canEmbed(): boolean {
  return ptyAvailable()
}

export function startExec(plan: LaunchPlan, harnessId: string, harnessName: string): EmbeddedRun | null {
  if (!ptyAvailable()) return null
  const handle = startPtyRun({ bin: plan.bin, args: [...plan.args, plan.prompt], cwd: plan.cwd })
  const disk = startDiskTracker(plan.cwd)
  const run: EmbeddedRun = {
    runId: handle.runId,
    plan,
    handle,
    harnessId,
    harnessName,
    processName: processName(plan),
    disk,
  }
  registry.add(run)
  handle.onExit((code) => {
    executorEvents.runExited(run, code)
  })
  executorEvents.runStarted(run)
  return run
}

export function getRun(runId: string): EmbeddedRun | null {
  for (const run of registry) if (run.runId === runId) return run
  return null
}

export function listRuns(): EmbeddedRun[] {
  return [...registry]
}

export function listActiveRuns(): EmbeddedRun[] {
  return [...registry].filter((run) => run.handle.status() === 'running')
}

export function terminateRun(runId: string): void {
  getRun(runId)?.handle.terminate()
}

/** Remove a run do registro (fechamento explícito). Devolve false se já não existia. */
export function closeRun(runId: string): boolean {
  for (const run of registry) {
    if (run.runId === runId) return registry.delete(run)
  }
  return false
}

export function runningCount(): number {
  return listActiveRuns().length
}
