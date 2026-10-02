import { createHash } from 'node:crypto'
import { createServer, type Server } from 'node:http'

const DISCOVERY_PORT_BASE = 40000
const DISCOVERY_PORT_RANGE = 5000

/**
 * Porta do farol de descoberta: derivada do caminho absoluto do workspace,
 * nunca aleatória — a mesma workspace sempre resolve à mesma porta, em toda
 * execução, pra uma aba antiga conseguir reencontrar um host reiniciado sem
 * nenhum estado compartilhado além dessa conta (ADR-0006 decisão 8).
 */
export function discoveryPort(workspace: string): number {
  const digest = createHash('sha256').update(workspace).digest()
  return DISCOVERY_PORT_BASE + (digest.readUInt32BE(0) % DISCOVERY_PORT_RANGE)
}

function originAllowed(origin: string | undefined): origin is string {
  return origin !== undefined && /^http:\/\/127\.0\.0\.1:\d+$/.test(origin)
}

export interface DiscoveryBeacon {
  port: number
  close(): Promise<void>
}

/**
 * Farol somente-leitura, sem token: devolve só a porta efêmera atual do
 * servidor principal para quem já sabia como chegar até aqui (o workspace
 * decide a porta do farol). Aceita origem cross-origin de 127.0.0.1 porque é
 * exatamente isso que uma aba presa numa origem morta precisa para ler a
 * resposta e navegar até a origem nova.
 */
export function startDiscoveryBeacon(workspace: string, currentPort: number): Promise<DiscoveryBeacon> {
  const port = discoveryPort(workspace)
  const server: Server = createServer((req, res) => {
    const origin = req.headers.origin
    const headers: Record<string, string> = { 'content-type': 'application/json; charset=utf-8' }
    if (originAllowed(origin)) headers['access-control-allow-origin'] = origin
    if (req.method !== 'GET' || req.url !== '/') {
      res.writeHead(404, headers)
      res.end()
      return
    }
    res.writeHead(200, headers)
    res.end(JSON.stringify({ port: currentPort }))
  })
  return new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(port, '127.0.0.1', () => {
      resolve({
        port,
        close: () => new Promise((done) => server.close(() => done())),
      })
    })
  })
}
