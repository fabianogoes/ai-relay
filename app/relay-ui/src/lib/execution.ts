import { reactive } from 'vue'
import { apiGetJson, apiPostJson } from './relay-client'
import { createRunSocket, type RunSocket } from './term-client'

export type RunStatus = 'running' | 'exited'

export interface DiskEntry {
  id: number
  type: 'updated' | 'cleared'
  path: string
  at: string
  meaning: string
  before: string
  after: string
}

export interface ExecutionState {
  activeRunId: string | null
  status: RunStatus
  detached: boolean
  harnessId: string
  harnessName: string
  processName: string
  writtenFiles: number
  reattached: boolean
  firstRunThisSession: boolean
}

const store = reactive<ExecutionState>({
  activeRunId: null,
  status: 'running',
  detached: false,
  harnessId: '',
  harnessName: '',
  processName: '',
  writtenFiles: 0,
  reattached: false,
  firstRunThisSession: true,
})

const diskEntries = reactive<DiskEntry[]>([])

export function useDisk(): Readonly<DiskEntry[]> {
  return diskEntries
}

let socket: RunSocket | null = null
let clientScrollback = ''
const dataListeners = new Set<(data: string) => void>()

export function useExecution(): Readonly<ExecutionState> {
  return store
}

export function drainScrollback(): string {
  const s = clientScrollback
  clientScrollback = ''
  return s
}

export function subscribeData(cb: (data: string) => void): () => void {
  dataListeners.add(cb)
  const initial = drainScrollback()
  if (initial) cb(initial)
  return () => dataListeners.delete(cb)
}

function pushData(chunk: string): void {
  // repassa aos ouvintes ativos; se nenhum ouvir, acumula para replay
  if (dataListeners.size === 0) clientScrollback += chunk
  for (const cb of dataListeners) cb(chunk)
}

export interface EmbeddedLaunch {
  runId: string
  scrollback: string
}

export function launchEmbedded(params: {
  harness: string
  harnessName: string
  skill: string
  intent: string
  processName: string
}): Promise<EmbeddedLaunch> {
  return apiPostJson<EmbeddedLaunch>('/api/launch/embedded', {
    harness: params.harness,
    skill: params.skill,
    intent: params.intent,
  }).then((result) => {
    store.activeRunId = result.runId
    store.status = 'running'
    store.detached = false
    store.harnessId = params.harness
    store.harnessName = params.harnessName
    store.processName = params.processName
    store.reattached = false
    clientScrollback = result.scrollback
    diskEntries.length = 0
    store.writtenFiles = 0
    wire(result.runId)
    return result
  })
}

function wire(runId: string): void {
  socket?.close()
  socket = createRunSocket()
  socket.onData(pushData)
  socket.onExit((code) => {
    if (store.activeRunId !== runId) return
    store.status = 'exited'
    store.detached = false
  })
  socket.onDisk((entries) => {
    // reanexar reenvia todos os diffs acumulados, não só os novos — dedupe por
    // id pra nao duplicar entrada ja conhecida (A-006).
    for (const e of entries as DiskEntry[]) {
      if (diskEntries.some((existing) => existing.id === e.id)) continue
      diskEntries.push(e)
    }
    store.writtenFiles = diskEntries.length
  })
  socket.connect(runId)
}

export function detachRun(): void {
  store.detached = true
  socket?.close()
  socket = null
}

export function reattachRun(): void {
  if (!store.activeRunId) return
  store.detached = false
  store.reattached = true
  wire(store.activeRunId)
}

export function sendInput(text: string): void {
  socket?.send(text)
}

export function terminateRun(): void {
  if (!store.activeRunId) return
  const runId = store.activeRunId
  // não marca `exited` aqui: o status real só muda quando socket.onExit()
  // (wire()) recebe a saída confirmada pelo host, nunca de forma otimista.
  void apiPostJson(`/api/run/${runId}`, { action: 'terminate' }).catch(() => {})
}

export function closeRun(): void {
  const runId = store.activeRunId
  if (runId) void apiPostJson(`/api/run/${runId}`, { action: 'close' }).catch(() => {})
  socket?.close()
  socket = null
  store.activeRunId = null
  store.detached = false
  store.reattached = false
  clientScrollback = ''
  diskEntries.length = 0
  store.writtenFiles = 0
}

export function markRunConsumed(): void {
  store.firstRunThisSession = false
}

export interface RunInfo {
  runId: string
  status: RunStatus
  exitCode: number | null
  startedAt: string
  harnessId: string
  harnessName: string
  processName: string
}

/**
 * Recupera uma run viva do host que a aba ainda não conhece — fechar a aba e
 * reabrir durante uma run em andamento reanexa a ela sem reload manual
 * (A-009). Não faz nada se já houver uma run ativa localmente, ou se o host
 * não reportar nenhuma run em execução.
 */
export function discoverActiveRun(): Promise<void> {
  if (store.activeRunId) return Promise.resolve()
  return apiGetJson<RunInfo[]>('/api/runs')
    .then((runs) => {
      if (store.activeRunId) return
      const run = runs.find((r) => r.status === 'running')
      if (!run) return
      store.activeRunId = run.runId
      store.status = 'running'
      store.detached = false
      store.harnessId = run.harnessId
      store.harnessName = run.harnessName
      store.processName = run.processName
      store.reattached = true
      clientScrollback = ''
      diskEntries.length = 0
      store.writtenFiles = 0
      wire(run.runId)
    })
    .catch(() => {})
}
