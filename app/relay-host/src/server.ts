import { createServer, type IncomingMessage, type Server, type ServerResponse } from 'node:http'
import type { AddressInfo } from 'node:net'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { WebSocket, WebSocketServer } from 'ws'
import type { ChangelogRecord, UiPayload } from 'relay-core'
import type { Harness, LaunchPlan } from './harness.ts'
import type { LaunchRequest, LaunchResult } from './launcher.ts'
import type { SpecSummary } from './specs.ts'
import type { RunInfo } from './pty.ts'

const UI_DIST = fileURLToPath(new URL('../../relay-ui/dist', import.meta.url))

const MIME: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
  '.woff': 'font/woff',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.json': 'application/json',
  '.map': 'application/json',
}

export interface ServerDeps {
  payload(): UiPayload
  specs(): SpecSummary[]
  changelog(): string
  changelogEntries(): ChangelogRecord[]
  spec(id: string): string | null
  harnesses(): Harness[]
  launchPreview(request: LaunchRequest): LaunchPlan | null
  launch(request: LaunchRequest): LaunchResult
  launchEmbedded?(request: LaunchRequest): EmbeddedStart | null
  runInfo?(runId: string): RunInfo | null
  runWrite?(runId: string, data: string): void
  runTerminate?(runId: string): void
  runClose?(runId: string): boolean
  runScrollback?(runId: string): string | null
  runDiskEntries?(runId: string): unknown[] | null
  runs?(): RunInfo[]
}

export interface EmbeddedStart {
  runId: string
  scrollback: string
}

export interface RelayServerOptions {
  workspace: string
  execEnabled: boolean
  token: string
  discoveryPort: number
  port?: number
  deps: ServerDeps
}

export interface RelayServer {
  port: number
  address: string
  close(): Promise<void>
  broadcast(): void
  broadcastRefreshing(): void
  termBroadcast(runId: string, data: string): void
  termExit(runId: string, code: number | null): void
  termDisk(runId: string, entries: unknown): void
}

export type RelayMessage =
  | { kind: 'snapshot'; payload: UiPayload }
  | { kind: 'refreshing' }

function json(res: ServerResponse, status: number, body: unknown): void {
  res.writeHead(status, { 'content-type': 'application/json; charset=utf-8' })
  res.end(JSON.stringify(body))
}

function readJsonBody(req: IncomingMessage): Promise<Record<string, unknown>> {
  return new Promise((resolve, reject) => {
    let raw = ''
    req.setEncoding('utf8')
    req.on('data', (chunk) => {
      raw += chunk
      if (raw.length > 1024 * 1024) {
        reject(new Error('corpo grande demais'))
        req.destroy()
      }
    })
    req.on('end', () => {
      if (raw.length === 0) {
        resolve({})
        return
      }
      try {
        resolve(JSON.parse(raw) as Record<string, unknown>)
      } catch {
        reject(new Error('corpo inválido'))
      }
    })
    req.on('error', reject)
  })
}

function text(res: ServerResponse, status: number, body: string): void {
  res.writeHead(status, { 'content-type': 'text/plain; charset=utf-8' })
  res.end(body)
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

type MetaOptions = Pick<RelayServerOptions, 'workspace' | 'execEnabled' | 'token' | 'discoveryPort'>

function bootstrapHtml(options: MetaOptions): string {
  return [
    '<!doctype html>',
    '<html lang="pt-BR">',
    '<head>',
    '<meta charset="utf-8">',
    `<meta name="relay-token" content="${escapeHtml(options.token)}">`,
    `<meta name="relay-workspace" content="${escapeHtml(options.workspace)}">`,
    `<meta name="relay-exec-enabled" content="${options.execEnabled ? 'true' : 'false'}">`,
    `<meta name="relay-discovery-port" content="${options.discoveryPort}">`,
    '<title>Relay</title>',
    '</head>',
    '<body>',
    `<p>Relay host — workspace: ${escapeHtml(options.workspace)}</p>`,
    '</body>',
    '</html>',
    '',
  ].join('\n')
}

function relayMetas(options: MetaOptions): string {
  return [
    `<meta name="relay-token" content="${escapeHtml(options.token)}">`,
    `<meta name="relay-workspace" content="${escapeHtml(options.workspace)}">`,
    `<meta name="relay-exec-enabled" content="${options.execEnabled ? 'true' : 'false'}">`,
    `<meta name="relay-discovery-port" content="${options.discoveryPort}">`,
  ].join('\n    ')
}

function uiIndex(options: MetaOptions): string | null {
  const file = join(UI_DIST, 'index.html')
  if (!existsSync(file)) return null
  const html = readFileSync(file, 'utf8')
  if (html.includes('<head>')) {
    return html.replace('<head>', `<head>\n    ${relayMetas(options)}`)
  }
  return html
}

function serveStatic(res: ServerResponse, pathname: string): boolean {
  const file = join(UI_DIST, pathname)
  if (!file.startsWith(UI_DIST)) return false
  try {
    if (!existsSync(file) || statSync(file).isDirectory()) return false
    const type = MIME[extname(file)] ?? 'application/octet-stream'
    res.writeHead(200, { 'content-type': type })
    res.end(readFileSync(file))
    return true
  } catch {
    return false
  }
}

export function createRelayServer(options: RelayServerOptions): Promise<RelayServer> {
  const { token, workspace, execEnabled, deps, port = 0, discoveryPort } = options
  const clients = new Set<WebSocket>()
  let serverOrigin = ''
  let refreshing = false

  function broadcast(): void {
    refreshing = false
    const data = JSON.stringify({ kind: 'snapshot', payload: deps.payload() } satisfies RelayMessage)
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) client.send(data)
    }
  }

  function broadcastRefreshing(): void {
    refreshing = true
    const data = JSON.stringify({ kind: 'refreshing' } satisfies RelayMessage)
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) client.send(data)
    }
  }

  function authed(req: IncomingMessage): boolean {
    if (req.headers['sec-fetch-site'] !== 'same-origin') return false
    return req.headers['x-relay-token'] === token
  }

  async function handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const url = new URL(req.url ?? '/', `http://${req.headers.host ?? 'localhost'}`)
    if (url.pathname === '/' && req.method === 'GET') {
      const ui = uiIndex({ workspace, execEnabled, token, discoveryPort })
      if (ui !== null) {
        res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' })
        res.end(ui)
      } else {
        res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' })
        res.end(bootstrapHtml({ workspace, execEnabled, token, discoveryPort }))
      }
      return
    }
    if (url.pathname.startsWith('/assets/') && req.method === 'GET') {
      if (serveStatic(res, url.pathname)) return
      res.writeHead(404)
      res.end()
      return
    }
    if (!url.pathname.startsWith('/api/')) {
      res.writeHead(404)
      res.end()
      return
    }
    if (!authed(req)) {
      res.writeHead(403)
      res.end()
      return
    }
    if (url.pathname === '/api/launch' || url.pathname === '/api/launch/preview') {
      if (!execEnabled) {
        res.writeHead(404)
        res.end()
        return
      }
      if (req.method !== 'POST') {
        res.writeHead(404)
        res.end()
        return
      }
      try {
        const body = await readJsonBody(req)
        const request: LaunchRequest = {
          harness: String(body.harness ?? ''),
          skill: body.skill === 'relay-spec' ? 'relay-spec' : 'relay-session',
          intent: String(body.intent ?? ''),
        }
        if (url.pathname === '/api/launch/preview') {
          const plan = deps.launchPreview(request)
          if (plan === null) {
            res.writeHead(400)
            res.end()
            return
          }
          json(res, 200, plan)
          return
        }
        const result = deps.launch(request)
        json(res, 200, result)
      } catch {
        res.writeHead(400)
        res.end()
      }
      return
    }
    if (url.pathname === '/api/launch/embedded') {
      if (!execEnabled || !deps.launchEmbedded) {
        res.writeHead(404)
        res.end()
        return
      }
      if (req.method !== 'POST') {
        res.writeHead(404)
        res.end()
        return
      }
      try {
        const body = await readJsonBody(req)
        const request: LaunchRequest = {
          harness: String(body.harness ?? ''),
          skill: body.skill === 'relay-spec' ? 'relay-spec' : 'relay-session',
          intent: String(body.intent ?? ''),
        }
        const started = deps.launchEmbedded(request)
        if (started === null) {
          res.writeHead(400)
          res.end()
          return
        }
        json(res, 200, started)
      } catch {
        res.writeHead(400)
        res.end()
      }
      return
    }
    if (url.pathname.startsWith('/api/run/') && req.method === 'POST') {
      if (!execEnabled) {
        res.writeHead(404)
        res.end()
        return
      }
      const runId = decodeURIComponent(url.pathname.slice('/api/run/'.length))
      try {
        const body = await readJsonBody(req)
        if (body.action === 'terminate') {
          if (!deps.runInfo || deps.runInfo(runId) === null) {
            res.writeHead(404)
            res.end()
            return
          }
          deps.runTerminate?.(runId)
          json(res, 200, { terminated: true })
        } else if (body.action === 'close') {
          const closed = deps.runClose ? deps.runClose(runId) : false
          if (!closed) {
            res.writeHead(404)
            res.end()
            return
          }
          json(res, 200, { closed: true })
        } else {
          res.writeHead(400)
          res.end()
        }
      } catch {
        res.writeHead(400)
        res.end()
      }
      return
    }
    if (req.method !== 'GET') {
      res.writeHead(404)
      res.end()
      return
    }
    const parts = url.pathname.split('/').filter(Boolean) // ['api', route, ...]
    const route = parts[1] ?? ''
    if (route === 'state') {
      json(res, 200, deps.payload())
      return
    }
    if (route === 'specs') {
      if (parts.length === 2) {
        json(res, 200, deps.specs())
        return
      }
      const id = decodeURIComponent(parts[2] ?? '')
      const content = deps.spec(id)
      if (content === null) {
        res.writeHead(404)
        res.end()
        return
      }
      text(res, 200, content)
      return
    }
    if (route === 'changelog') {
      if (parts[2] === 'entries') {
        json(res, 200, deps.changelogEntries())
        return
      }
      text(res, 200, deps.changelog())
      return
    }
    if (route === 'harnesses') {
      json(res, 200, deps.harnesses())
      return
    }
    if (route === 'runs' || route === 'run') {
      if (!execEnabled) {
        res.writeHead(404)
        res.end()
        return
      }
    }
    if (route === 'runs') {
      json(res, 200, deps.runs ? deps.runs() : [])
      return
    }
    if (route === 'run' && parts.length >= 3) {
      const runId = decodeURIComponent(parts[2] ?? '')
      if (parts.length === 4 && parts[3] === 'disk') {
        const entries = deps.runDiskEntries ? deps.runDiskEntries(runId) : null
        if (entries === null) {
          res.writeHead(404)
          res.end()
          return
        }
        json(res, 200, entries)
        return
      }
      if (parts.length === 3) {
        const info = deps.runInfo ? deps.runInfo(runId) : null
        if (info === null) {
          res.writeHead(404)
          res.end()
          return
        }
        json(res, 200, info)
        return
      }
    }
    res.writeHead(404)
    res.end()
  }

  const server: Server = createServer(handle)
  const wss = new WebSocketServer({
    noServer: true,
    handleProtocols(protocols) {
      return protocols.has(`relay.${token}`) ? `relay.${token}` : false
    },
  })

  const termWss = new WebSocketServer({
    noServer: true,
    handleProtocols(protocols) {
      return protocols.has(`relay.${token}`) ? `relay.${token}` : false
    },
  })

  const termClients = new Map<string, Set<WebSocket>>()

  function termBroadcast(runId: string, data: string): void {
    const clients = termClients.get(runId)
    if (!clients) return
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) client.send(data)
    }
  }

  function termExit(runId: string, code: number | null): void {
    const clients = termClients.get(runId)
    if (!clients) {
      termClients.delete(runId)
      return
    }
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) {
        client.send(JSON.stringify({ kind: 'exit', runId, exitCode: code }))
        client.close()
      }
    }
    termClients.delete(runId)
  }

  function termDisk(runId: string, entries: unknown): void {
    const clients = termClients.get(runId)
    if (!clients) return
    const frame = JSON.stringify({ kind: 'disk', runId, entries })
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) client.send(frame)
    }
  }

  server.on('upgrade', (req, socket, head) => {
    const url = new URL(req.url ?? '/', `http://${req.headers.host ?? 'localhost'}`)
    const requestOrigin = req.headers.origin
    const proto = req.headers['sec-websocket-protocol']
    const originOk = requestOrigin === serverOrigin
    const tokenOk = typeof proto === 'string' && proto === `relay.${token}`

    if (url.pathname === '/ws/term') {
      if (!originOk || !tokenOk) {
        socket.write('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n')
        socket.destroy()
        return
      }
      if (!execEnabled) {
        socket.write('HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n')
        socket.destroy()
        return
      }

      termWss.handleUpgrade(req, socket, head, (ws) => {
        let runId = ''
        ws.on('message', (raw) => {
          try {
            const msg = JSON.parse(String(raw)) as { kind?: string; runId?: string; data?: string }
            if (msg.kind === 'attach' && msg.runId) {
              runId = msg.runId
              let set = termClients.get(runId)
              if (!set) {
                set = new Set()
                termClients.set(runId, set)
              }
              set.add(ws)
              const sb = deps.runScrollback?.(runId) ?? ''
              if (sb) ws.send(JSON.stringify({ kind: 'data', runId, data: sb }))
              // reanexar precisa dos diffs acumulados durante a ausência, não só
              // do scrollback — sem isso o cliente perde escritas em disco (A-006).
              const diskSoFar = deps.runDiskEntries?.(runId) ?? null
              if (diskSoFar && diskSoFar.length > 0) {
                ws.send(JSON.stringify({ kind: 'disk', runId, entries: diskSoFar }))
              }
            } else if (msg.kind === 'input' && runId && msg.data !== undefined) {
              deps.runWrite?.(runId, msg.data)
            }
          } catch {
            // frame inválido: ignora
          }
        })
        ws.on('close', () => {
          if (runId) termClients.get(runId)?.delete(ws)
        })
        ws.on('error', () => {
          if (runId) termClients.get(runId)?.delete(ws)
        })
      })
      return
    }

    if (url.pathname !== '/ws' || !originOk || !tokenOk) {
      socket.write('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n')
      socket.destroy()
      return
    }
    wss.handleUpgrade(req, socket, head, (ws) => {
      clients.add(ws)
      ws.on('close', () => clients.delete(ws))
      ws.on('error', () => clients.delete(ws))
      const initial: RelayMessage = refreshing
        ? { kind: 'refreshing' }
        : { kind: 'snapshot', payload: deps.payload() }
      ws.send(JSON.stringify(initial))
    })
  })

  return new Promise((resolve) => {
    server.on('listening', () => {
      const address = server.address() as AddressInfo
      serverOrigin = `http://${address.address}:${address.port}`
      resolve({
        port: address.port,
        address: address.address,
        broadcast,
        broadcastRefreshing,
        termBroadcast,
        termExit,
        termDisk,
        close: () =>
          new Promise<void>((done) => {
            for (const client of clients) client.close()
            for (const set of termClients.values()) for (const c of set) c.close()
            termWss.close()
            wss.close()
            server.closeAllConnections()
            server.close(() => done())
          }),
      })
    })
    server.listen(port, '127.0.0.1')
  })
}
