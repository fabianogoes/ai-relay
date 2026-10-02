import { test } from 'node:test'
import assert from 'node:assert/strict'
import { WebSocket } from 'ws'

import { start } from '../src/index.ts'
import { ptyAvailable } from '../src/pty.ts'
import { startExec } from '../src/executor.ts'
import { makeWorkspace } from '../support/workspace.ts'

function extractToken(html: string): string {
  const match = html.match(/<meta name="relay-token" content="([^"]+)">/)
  if (!match) throw new Error('token não encontrado no HTML')
  return match[1]!
}

function apiFetch(base: string, path: string, token: string, init: RequestInit = {}): Promise<Response> {
  return fetch(`${base}${path}`, {
    ...init,
    headers: { ...(init.headers ?? {}), 'x-relay-token': token, 'sec-fetch-site': 'same-origin' },
  })
}

type TermMessage = { kind: string; runId?: string; data?: string; exitCode?: number | null; entries?: unknown[] }

function connectTermWs(url: string, token: string): Promise<{ ws: WebSocket; messages: TermMessage[] }> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url, `relay.${token}`, { origin: `http://127.0.0.1:${new URL(url).port}` })
    const messages: TermMessage[] = []
    ws.on('message', (data) => messages.push(JSON.parse(data.toString()) as TermMessage))
    ws.once('open', () => resolve({ ws, messages }))
    ws.once('error', reject)
  })
}

async function waitForTermMessage(
  messages: TermMessage[],
  predicate: (m: TermMessage) => boolean,
  timeoutMs = 3000,
): Promise<TermMessage> {
  const existing = messages.find(predicate)
  if (existing) return existing
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      clearInterval(iv)
      reject(new Error('waitForTermMessage: nenhuma mensagem casou o predicado a tempo'))
    }, timeoutMs)
    const iv = setInterval(() => {
      const match = messages.find(predicate)
      if (match) {
        clearInterval(iv)
        clearTimeout(timeout)
        resolve(match)
      }
    }, 10)
  })
}

test(
  'fluxo real ponta a ponta: iniciar, desanexar, escrever, reanexar, terminar e verificar saída (A-012)',
  { skip: !ptyAvailable() },
  async () => {
    const ws = makeWorkspace()
    // startExec direto (não via /api/launch/embedded) para o processo ser
    // determinístico (/bin/sh) em vez de depender de um harness real
    // instalado — a composição de argv do harness já está coberta em outro
    // teste (A-001/A-002); aqui o alvo é o ciclo de vida ponta a ponta.
    const server = await start({ workspace: ws.dir, execEnabled: true })
    const base = `http://127.0.0.1:${server.port}`
    let firstClient: WebSocket | null = null
    let secondClient: WebSocket | null = null
    let run: ReturnType<typeof startExec> = null
    try {
      const html = await (await fetch(`${base}/`)).text()
      const token = extractToken(html)

      // script determinístico: lê uma linha, escreve em TODO.md, lê outra linha
      // (mantém o processo vivo pra testar desanexar/reanexar) e sai.
      const script = 'read line; printf "%s\\n" "$line" > .orchestration/TODO.md; read line2; exit 0'
      run = startExec({ bin: '/bin/sh', args: ['-c', script], prompt: '', cwd: ws.dir }, 'claude-code', 'Claude Code')
      assert.ok(run, 'startExec deveria retornar uma run com PTY disponível')
      const runId = run!.runId

      // HTTP real: a run recém-iniciada já é endereçável (A-005)
      const infoRes = await apiFetch(base, `/api/run/${runId}`, token)
      assert.equal(infoRes.status, 200)
      assert.equal(((await infoRes.json()) as { status: string }).status, 'running')

      // WS real: anexar recebe o scrollback (vazio ainda) e passa a receber dados
      const first = await connectTermWs(`ws://127.0.0.1:${server.port}/ws/term`, token)
      firstClient = first.ws
      first.ws.send(JSON.stringify({ kind: 'attach', runId }))

      // escrever via input real do terminal: muda um registro que o disk tracker observa
      first.ws.send(JSON.stringify({ kind: 'input', runId, data: 'novo conteudo\n' }))
      await waitForTermMessage(first.messages, (m) => m.kind === 'data' && (m.data ?? '').includes('novo conteudo'))
      await new Promise((resolve) => setTimeout(resolve, 200))

      // desanexar: fechar o WS do cliente não termina o processo (A-006)
      first.ws.close()
      assert.equal(run!.handle.status(), 'running', 'o processo continua vivo depois de desanexar')

      const infoWhileDetached = await apiFetch(base, `/api/run/${runId}`, token)
      assert.equal(((await infoWhileDetached.json()) as { status: string }).status, 'running')

      // reanexar: recupera o diff acumulado durante a ausência, sem esperar o watcher (A-006)
      const second = await connectTermWs(`ws://127.0.0.1:${server.port}/ws/term`, token)
      secondClient = second.ws
      second.ws.send(JSON.stringify({ kind: 'attach', runId }))
      const diskMsg = await waitForTermMessage(second.messages, (m) => m.kind === 'disk')
      const entries = diskMsg.entries as Array<{ path: string }>
      assert.ok(
        entries.some((e) => e.path === '.orchestration/TODO.md'),
        `esperava um diff de .orchestration/TODO.md, recebi ${JSON.stringify(entries)}`,
      )

      // terminar via HTTP real e verificar a saída real via WS, não otimista (A-008)
      const terminateRes = await apiFetch(base, `/api/run/${runId}`, token, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ action: 'terminate' }),
      })
      assert.equal(terminateRes.status, 200)

      const exitMsg = await waitForTermMessage(second.messages, (m) => m.kind === 'exit')
      assert.notEqual(exitMsg.exitCode, undefined)

      const infoAfterExit = await apiFetch(base, `/api/run/${runId}`, token)
      assert.equal(((await infoAfterExit.json()) as { status: string }).status, 'exited')

      // run continua endereçável até fechamento explícito (A-006/A-007)
      const closeRes = await apiFetch(base, `/api/run/${runId}`, token, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ action: 'close' }),
      })
      assert.equal(closeRes.status, 200)
      assert.equal((await apiFetch(base, `/api/run/${runId}`, token)).status, 404)
    } finally {
      firstClient?.close()
      secondClient?.close()
      run?.handle.terminate()
      await server.close()
      ws.cleanup()
    }
  },
)
