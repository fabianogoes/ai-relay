import { test } from 'node:test'
import assert from 'node:assert/strict'

import { createRelayServer, type ServerDeps } from '../src/server.ts'
import { readWorkspace } from '../src/reader.ts'
import { buildPayload } from '../src/state.ts'
import { listSpecs } from '../src/specs.ts'
import { preview } from '../src/launcher.ts'
import { makeWorkspace } from '../support/workspace.ts'
import { WebSocket } from 'ws'
import { parseChangelog, type UiPayload } from 'relay-core'

const TOKEN = 'test-token'

function makeDeps(workspace: string): ServerDeps {
  const read = () => readWorkspace(workspace)
  const backlogSpecs = () => {
    const state = buildPayload(read(), { workspace, execEnabled: true }).state
    return state.kind === 'ok' ? state.backlog.map((e) => e.spec) : []
  }
  return {
    payload: () => buildPayload(read(), { workspace, execEnabled: true }),
    specs: () => listSpecs(read(), backlogSpecs()),
    changelog: () => read().changelog,
    changelogEntries: () => parseChangelog(read().changelog),
    spec: (id) => read().specs[`.specs/${id}`] ?? null,
    harnesses: () => [{ id: 'test', name: 'Test', version: '1.0.0', state: 'installed' }],
    launchPreview: (req) => {
      try {
        return preview(req, workspace)
      } catch {
        return null
      }
    },
    launch: (req) => ({
      runId: 'run-1',
      scratchDir: workspace,
      plan: preview(req, workspace),
    }),
  }
}

async function request(
  base: string,
  path: string,
  opts: { method?: string; token?: string; fetchSite?: string; body?: unknown } = {},
): Promise<Response> {
  const headers: Record<string, string> = {}
  if (opts.token !== undefined) headers['x-relay-token'] = opts.token
  if (opts.fetchSite !== undefined) headers['sec-fetch-site'] = opts.fetchSite
  if (opts.body !== undefined) headers['content-type'] = 'application/json'
  return fetch(`${base}${path}`, {
    method: opts.method ?? 'GET',
    headers,
    body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
  })
}

test('server: bind em 127.0.0.1, porta efêmera', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({
    workspace: ws.dir,
    execEnabled: true,
    token: TOKEN,
    discoveryPort: 41234,
    deps: makeDeps(ws.dir),
  })
  try {
    assert.equal(server.address, '127.0.0.1')
    assert.ok(server.port > 0)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('GET / é o bootstrap: sem token, entrega HTML com token e workspace', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  try {
    const res = await fetch(`http://127.0.0.1:${server.port}/`)
    assert.equal(res.status, 200)
    const html = await res.text()
    assert.ok(html.includes(`content="${TOKEN}"`))
    assert.ok(html.includes(ws.dir))
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('servir a UI construída: metas injetadas e assets estáticos resolvem', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const res = await fetch(`${base}/`)
    assert.equal(res.status, 200)
    const html = await res.text()
    assert.ok(html.includes('relay-token'))
    assert.ok(html.includes('relay-workspace'))
    assert.ok(html.includes('relay-exec-enabled'))
    const assetMatches = [...html.matchAll(/src="(\/assets\/[^"]+)"/g)].map((m) => m[1])
    for (const asset of assetMatches) {
      const assetRes = await fetch(`${base}${asset}`)
      assert.equal(assetRes.status, 200)
    }
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('API sem token ou sem same-origin é 403', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    assert.equal((await request(base, '/api/state')).status, 403)
    assert.equal((await request(base, '/api/state', { token: 'errado', fetchSite: 'same-origin' })).status, 403)
    assert.equal((await request(base, '/api/state', { token: TOKEN })).status, 403)
    assert.equal((await request(base, '/api/state', { token: TOKEN, fetchSite: 'cross-site' })).status, 403)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('API com token e same-origin devolve dado', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const state = await request(base, '/api/state', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(state.status, 200)
    const payload = (await state.json()) as UiPayload
    assert.equal(payload.state.kind, 'ok')
    if (payload.state.kind === 'ok') assert.equal(payload.state.status, 'backlog')

    const specs = await request(base, '/api/specs', { token: TOKEN, fetchSite: 'same-origin' })
    const list = (await specs.json()) as Array<{ id: string; title: string; taskCount: number }>
    assert.deepEqual(list, [{ id: '20260907-001-teste.md', title: '20260907-001 - Teste', taskCount: 1 }])

    const changelog = await request(base, '/api/changelog', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(await changelog.text(), '# Change log\n')

    const specRaw = await request(base, '/api/specs/20260907-001-teste.md', { token: TOKEN, fetchSite: 'same-origin' })
    assert.ok((await specRaw.text()).includes('## Acceptance criteria'))

    assert.equal((await request(base, '/api/specs/nao-existe.md', { token: TOKEN, fetchSite: 'same-origin' })).status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('GET /api/changelog/entries devolve registros estruturados por backlog', async () => {
  const ws = makeWorkspace([
    [
      '.orchestration/CHANGELOG.md',
      '# Change log\n\n## 2026-09-10 - T-001 - Titulo do registro\n- Backlog: B-001\n- Spec: .specs/20260907-001-teste.md\n- Result: breve.\n- Evidence: vitest 4/4\n- Criteria: A-001\n',
    ],
  ])
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const res = await request(base, '/api/changelog/entries', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(res.status, 200)
    const entries = await res.json()
    assert.deepEqual(entries, [
      {
        date: '2026-09-10',
        todoId: 'T-001',
        title: 'Titulo do registro',
        backlogId: 'B-001',
        spec: '.specs/20260907-001-teste.md',
        evidence: 'vitest 4/4',
        criteria: ['A-001'],
      },
    ])
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('rota de lançamento: POST /api/launch/preview compõe o plano, sem shell', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const res = await request(base, '/api/launch/preview', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { harness: 'claude-code', skill: 'relay-session', intent: 'Retomar sessão' },
    })
    assert.equal(res.status, 200)
    const plan = (await res.json()) as { bin: string; args: string[]; prompt: string; cwd: string }
    assert.equal(plan.bin, 'claude')
    assert.deepEqual(plan.args, ['-p'])
    assert.equal(plan.prompt, '/relay-session Retomar sessão')
    assert.equal(plan.cwd, ws.dir)

    const launch = await request(base, '/api/launch', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { harness: 'codex', skill: 'relay-spec', intent: 'Especificar uma ideia' },
    })
    assert.equal(launch.status, 200)
    const result = (await launch.json()) as { runId: string }
    assert.equal(result.runId, 'run-1')

    const bad = await request(base, '/api/launch/preview', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { harness: 'nao-existe', skill: 'relay-session', intent: 'x' },
    })
    assert.equal(bad.status, 400)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('sob --no-exec a rota de lançamento também é 404, não 403 (A-011)', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: false, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const res = await request(base, '/api/launch', { method: 'POST', token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(res.status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('sob --no-exec, GET /api/runs, GET /api/run/<id> e /disk não existem: 404 com auth válida (A-011)', async () => {
  const ws = makeWorkspace()
  const deps: ServerDeps = {
    ...makeDeps(ws.dir),
    runs: () => [],
    runInfo: () => ({
      runId: 'run-1',
      status: 'running',
      exitCode: null,
      startedAt: '',
      harnessId: 'claude-code',
      harnessName: 'Claude Code',
      processName: 'p',
    }),
    runDiskEntries: () => [],
  }
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: false, token: TOKEN, discoveryPort: 41234, deps })
  const base = `http://127.0.0.1:${server.port}`
  try {
    assert.equal((await request(base, '/api/runs', { token: TOKEN, fetchSite: 'same-origin' })).status, 404)
    assert.equal((await request(base, '/api/run/run-1', { token: TOKEN, fetchSite: 'same-origin' })).status, 404)
    assert.equal((await request(base, '/api/run/run-1/disk', { token: TOKEN, fetchSite: 'same-origin' })).status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('sob --no-exec, o WebSocket /ws/term não existe: handshake com origem e token válidos recebe 404 (A-011)', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: false, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  try {
    const attempt = new Promise<number>((resolve, reject) => {
      const client = new WebSocket(`ws://127.0.0.1:${server.port}/ws/term`, `relay.${TOKEN}`, {
        origin: `http://127.0.0.1:${server.port}`,
      })
      client.once('unexpected-response', (_req, res) => {
        resolve(res.statusCode ?? 0)
        client.terminate()
      })
      client.once('open', () => {
        client.terminate()
        reject(new Error('handshake deveria falhar sob --no-exec, não abrir'))
      })
      client.once('error', () => {
        // 'error' pode disparar junto de 'unexpected-response' dependendo da lib; ignora aqui
      })
    })
    const timeout = new Promise<number>((_, reject) =>
      setTimeout(() => reject(new Error('handshake de /ws/term não respondeu em 2s')), 2000),
    )
    const status = await Promise.race([attempt, timeout])
    assert.equal(status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('rota embutida exige exec ligado e launchEmbedded; sem ela, 404', async () => {
  const ws = makeWorkspace()
  // com exec ligado mas sem launchEmbedded no deps, devolve 404 (superfície reservada)
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const res = await request(base, '/api/launch/embedded', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { harness: 'claude-code', skill: 'relay-session', intent: 'x' },
    })
    assert.equal(res.status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('GET /api/run/<id> e /disk devolvem dado da run existente e 404 para run desconhecida (A-005)', async () => {
  const ws = makeWorkspace()
  const deps: ServerDeps = {
    ...makeDeps(ws.dir),
    runInfo: (runId) =>
      runId === 'run-1'
        ? {
            runId,
            status: 'running',
            exitCode: null,
            startedAt: '2026-01-01T00:00:00Z',
            harnessId: 'claude-code',
            harnessName: 'Claude Code',
            processName: 'claude · relay-session',
          }
        : null,
    runDiskEntries: (runId) => (runId === 'run-1' ? [{ path: 'a', type: 'updated' }] : null),
  }
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const info = await request(base, '/api/run/run-1', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(info.status, 200)
    assert.deepEqual(await info.json(), {
      runId: 'run-1',
      status: 'running',
      exitCode: null,
      startedAt: '2026-01-01T00:00:00Z',
      harnessId: 'claude-code',
      harnessName: 'Claude Code',
      processName: 'claude · relay-session',
    })

    const disk = await request(base, '/api/run/run-1/disk', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(disk.status, 200)
    assert.deepEqual(await disk.json(), [{ path: 'a', type: 'updated' }])

    const missingInfo = await request(base, '/api/run/run-nao-existe', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(missingInfo.status, 404)

    const missingDisk = await request(base, '/api/run/run-nao-existe/disk', { token: TOKEN, fetchSite: 'same-origin' })
    assert.equal(missingDisk.status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('POST /api/run/<id> terminate: run existente termina, run desconhecida devolve 404 (A-008)', async () => {
  const ws = makeWorkspace()
  let terminated: string | null = null
  const deps: ServerDeps = {
    ...makeDeps(ws.dir),
    runInfo: (runId) =>
      runId === 'run-1'
        ? {
            runId,
            status: 'running',
            exitCode: null,
            startedAt: '2026-01-01T00:00:00Z',
            harnessId: 'claude-code',
            harnessName: 'Claude Code',
            processName: 'claude · relay-session',
          }
        : null,
    runTerminate: (runId) => {
      terminated = runId
    },
  }
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const ok = await request(base, '/api/run/run-1', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { action: 'terminate' },
    })
    assert.equal(ok.status, 200)
    assert.equal(terminated, 'run-1')

    const missing = await request(base, '/api/run/run-nao-existe', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { action: 'terminate' },
    })
    assert.equal(missing.status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('POST /api/run/<id> close: fecha run existente e devolve o registro; run desconhecida/já fechada é 404 (A-006/A-007)', async () => {
  const ws = makeWorkspace()
  let closed: string | null = null
  const deps: ServerDeps = {
    ...makeDeps(ws.dir),
    runClose: (runId) => {
      if (runId !== 'run-1' || closed !== null) return false
      closed = runId
      return true
    },
  }
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps })
  const base = `http://127.0.0.1:${server.port}`
  try {
    const ok = await request(base, '/api/run/run-1', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { action: 'close' },
    })
    assert.equal(ok.status, 200)
    assert.equal(closed, 'run-1')

    const again = await request(base, '/api/run/run-1', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { action: 'close' },
    })
    assert.equal(again.status, 404)

    const missing = await request(base, '/api/run/run-nao-existe', {
      method: 'POST',
      token: TOKEN,
      fetchSite: 'same-origin',
      body: { action: 'close' },
    })
    assert.equal(missing.status, 404)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

type RelayMessage =
  | { kind: 'snapshot'; payload: UiPayload }
  | { kind: 'refreshing' }

function connectWs(url: string, token: string): Promise<{ ws: WebSocket; messages: RelayMessage[] }> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url, `relay.${token}`, { origin: `http://127.0.0.1:${new URL(url).port}` })
    const messages: RelayMessage[] = []
    ws.on('message', (data) => messages.push(JSON.parse(data.toString()) as RelayMessage))
    ws.once('open', () => resolve({ ws, messages }))
    ws.once('error', reject)
    ws.once('unexpected-response', (_req, res) => {
      res.resume()
      ws.terminate()
      reject(new Error(`unexpected-response: ${res.statusCode}`))
    })
  })
}

async function firstMessage(messages: RelayMessage[]): Promise<RelayMessage> {
  if (messages.length > 0) return messages[0]
  return new Promise((resolve) => {
    const iv = setInterval(() => {
      if (messages.length > 0) {
        clearInterval(iv)
        resolve(messages[0])
      }
    }, 5)
  })
}

async function waitForMessage(
  messages: RelayMessage[],
  predicate: (message: RelayMessage) => boolean,
): Promise<RelayMessage> {
  const existing = messages.find(predicate)
  if (existing) return existing
  return new Promise((resolve) => {
    const iv = setInterval(() => {
      const match = messages.find(predicate)
      if (match) {
        clearInterval(iv)
        resolve(match)
      }
    }, 5)
  })
}

test('WebSocket entrega snapshot envelopado na conexao e sinaliza refreshing', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const url = `ws://127.0.0.1:${server.port}/ws`
  try {
    const client = await connectWs(url, TOKEN)
    const first = await firstMessage(client.messages)
    assert.equal(first.kind, 'snapshot')
    if (first.kind === 'snapshot') assert.equal(first.payload.state.kind, 'ok')

    server.broadcastRefreshing()
    const refreshing = await waitForMessage(client.messages, (message) => message.kind === 'refreshing')
    assert.deepEqual(refreshing, { kind: 'refreshing' })
    client.ws.close()

    await assert.rejects(connectWs(url, 'token-errado'), /unexpected-response: 403/)
  } finally {
    await server.close()
    ws.cleanup()
  }
})

test('cliente que conecta durante transicao recebe refreshing antes do proximo snapshot', async () => {
  const ws = makeWorkspace()
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: false, token: TOKEN, discoveryPort: 41234, deps: makeDeps(ws.dir) })
  const url = `ws://127.0.0.1:${server.port}/ws`
  try {
    server.broadcastRefreshing()
    const client = await connectWs(url, TOKEN)
    const first = await firstMessage(client.messages)
    assert.deepEqual(first, { kind: 'refreshing' })

    server.broadcast()
    const snapshot = await waitForMessage(client.messages, (message) => message.kind === 'snapshot')
    assert.equal(snapshot.kind, 'snapshot')
    client.ws.close()
  } finally {
    await server.close()
    ws.cleanup()
  }
})

type TermMessage = { kind: string; runId?: string; data?: string; entries?: unknown; exitCode?: number | null }

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
  predicate: (message: TermMessage) => boolean,
): Promise<TermMessage> {
  const existing = messages.find(predicate)
  if (existing) return existing
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      clearInterval(iv)
      reject(new Error('waitForTermMessage: nenhuma mensagem casou o predicado em 2s'))
    }, 2000)
    const iv = setInterval(() => {
      const match = messages.find(predicate)
      if (match) {
        clearInterval(iv)
        clearTimeout(timeout)
        resolve(match)
      }
    }, 5)
  })
}

test('anexar ao terminal reenvia scrollback e os diffs de disco acumulados, não só scrollback (A-006)', async () => {
  const ws = makeWorkspace()
  const deps: ServerDeps = {
    ...makeDeps(ws.dir),
    runScrollback: (runId) => (runId === 'run-1' ? 'scrollback previo' : null),
    runDiskEntries: (runId) =>
      runId === 'run-1' ? [{ id: 1, type: 'updated', path: 'a', at: '', meaning: '', before: '', after: '' }] : null,
  }
  const server = await createRelayServer({ workspace: ws.dir, execEnabled: true, token: TOKEN, discoveryPort: 41234, deps })
  const url = `ws://127.0.0.1:${server.port}/ws/term`
  try {
    const client = await connectTermWs(url, TOKEN)
    client.ws.send(JSON.stringify({ kind: 'attach', runId: 'run-1' }))

    const dataMsg = await waitForTermMessage(client.messages, (m) => m.kind === 'data')
    assert.equal(dataMsg.data, 'scrollback previo')

    const diskMsg = await waitForTermMessage(client.messages, (m) => m.kind === 'disk')
    assert.deepEqual(diskMsg.entries, [{ id: 1, type: 'updated', path: 'a', at: '', meaning: '', before: '', after: '' }])

    client.ws.close()
  } finally {
    await server.close()
    ws.cleanup()
  }
})
