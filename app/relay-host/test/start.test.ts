import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createServer } from 'node:http'

import { start } from '../src/index.ts'
import { discoveryPort } from '../src/discovery.ts'
import { makeWorkspace } from '../support/workspace.ts'

test('start: abre o farol de descoberta junto do servidor principal, e close() derruba os dois (A-010)', async () => {
  const ws = makeWorkspace()
  const server = await start({ workspace: ws.dir, execEnabled: false })
  try {
    const beaconPort = discoveryPort(ws.dir)
    const res = await fetch(`http://127.0.0.1:${beaconPort}/`)
    assert.equal(res.status, 200)
    assert.deepEqual(await res.json(), { port: server.port })
  } finally {
    await server.close()
    ws.cleanup()
  }

  const beaconPort = discoveryPort(ws.dir)
  await assert.rejects(() => fetch(`http://127.0.0.1:${beaconPort}/`))
})

test('matar e reiniciar o host muda a porta principal; o farol (mesma workspace, mesma porta) sempre aponta pra atual (A-010, A-012)', async () => {
  const ws = makeWorkspace()
  const beaconPort = discoveryPort(ws.dir)
  let server1: Awaited<ReturnType<typeof start>> | null = null
  let server2: Awaited<ReturnType<typeof start>> | null = null
  try {
    server1 = await start({ workspace: ws.dir, execEnabled: false })
    const port1 = server1.port

    const beforeRestart = await fetch(`http://127.0.0.1:${beaconPort}/`)
    assert.deepEqual(await beforeRestart.json(), { port: port1 })

    // "matar o host": fecha o processo/servidor da execução anterior
    await server1.close()
    server1 = null
    await assert.rejects(() => fetch(`http://127.0.0.1:${port1}/`))

    // "reiniciar": mesma workspace, nova execução, porta efêmera nova do SO
    server2 = await start({ workspace: ws.dir, execEnabled: false })
    const port2 = server2.port

    // a aba antiga não recalcula nada: consulta o MESMO farol de sempre
    const afterRestart = await fetch(`http://127.0.0.1:${beaconPort}/`)
    assert.deepEqual(await afterRestart.json(), { port: port2 })

    const bootstrap = await fetch(`http://127.0.0.1:${port2}/`)
    assert.equal(bootstrap.status, 200)
  } finally {
    await server1?.close()
    await server2?.close()
    ws.cleanup()
  }
})

test('start: falha e fecha o servidor principal quando a porta do farol já está ocupada (A-010)', async () => {
  const ws = makeWorkspace()
  const beaconPort = discoveryPort(ws.dir)
  const occupier = createServer((_req, res) => res.end())
  await new Promise<void>((resolve) => occupier.listen(beaconPort, '127.0.0.1', resolve))
  try {
    const timeout = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('start() deveria rejeitar, não travar com o servidor aberto')), 2000),
    )
    await assert.rejects(() => Promise.race([start({ workspace: ws.dir, execEnabled: false }), timeout]))
  } finally {
    await new Promise<void>((resolve) => occupier.close(() => resolve()))
    ws.cleanup()
  }
})
