import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createServer } from 'node:http'

import { discoveryPort, startDiscoveryBeacon } from '../src/discovery.ts'

test('discoveryPort: determinística e estável para o mesmo workspace (ADR-0006 decisão 8)', () => {
  const a = discoveryPort('/Users/dev/projeto-a')
  const b = discoveryPort('/Users/dev/projeto-a')
  assert.equal(a, b)
  assert.ok(a >= 40000 && a < 45000, `porta ${a} fora da faixa esperada`)
})

test('discoveryPort: workspaces diferentes tendem a portas diferentes', () => {
  const a = discoveryPort('/Users/dev/projeto-a')
  const b = discoveryPort('/Users/dev/projeto-b')
  assert.notEqual(a, b)
})

test('farol de descoberta: GET / devolve a porta atual, com CORS restrito a origens 127.0.0.1', async () => {
  const workspace = `/tmp/relay-discovery-test-${Date.now()}-1`
  const beacon = await startDiscoveryBeacon(workspace, 55123)
  try {
    const base = `http://127.0.0.1:${beacon.port}`

    const allowed = await fetch(base, { headers: { origin: 'http://127.0.0.1:9999' } })
    assert.equal(allowed.status, 200)
    assert.deepEqual(await allowed.json(), { port: 55123 })
    assert.equal(allowed.headers.get('access-control-allow-origin'), 'http://127.0.0.1:9999')

    const disallowed = await fetch(base, { headers: { origin: 'https://evil.example' } })
    assert.equal(disallowed.status, 200)
    assert.equal(disallowed.headers.get('access-control-allow-origin'), null)

    const notFound = await fetch(`${base}/outra-rota`)
    assert.equal(notFound.status, 404)
  } finally {
    await beacon.close()
  }
})

test('farol de descoberta: falha ao iniciar se a porta derivada já estiver em uso', async () => {
  const workspace = `/tmp/relay-discovery-test-${Date.now()}-2`
  const port = discoveryPort(workspace)
  const occupier = createServer((_req, res) => res.end())
  await new Promise<void>((resolve) => occupier.listen(port, '127.0.0.1', resolve))
  try {
    const timeout = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('startDiscoveryBeacon deveria rejeitar, não travar')), 2000),
    )
    await assert.rejects(() => Promise.race([startDiscoveryBeacon(workspace, 12345), timeout]))
  } finally {
    await new Promise<void>((resolve) => occupier.close(() => resolve()))
  }
})
