import { ref, type Ref } from 'vue'
import type { RelayMessage, UiPayload } from '../types'
import { reduceRelayViewState, type Freshness, type RelayViewState } from './observer'

export interface RelayClient {
  hostMode: boolean
  payload: Ref<UiPayload | null>
  connected: Ref<boolean>
  freshness: Ref<Freshness>
  reconnectBackoff: number
  connect(): void
  disconnect(): void
}

export const RECONNECT_BASE = 500
export const RECONNECT_MAX = 10_000

/**
 * Depois de algumas tentativas seguidas na mesma origem, para de assumir que
 * é uma falha transitória e passa a consultar o farol de descoberta — o host
 * pode ter reiniciado numa porta nova (ADR-0006 decisão 8, A-010).
 */
export const DISCOVERY_AFTER_RETRIES = 3

export function shouldAttemptDiscovery(attempt: number): boolean {
  return attempt >= DISCOVERY_AFTER_RETRIES
}

export function parseDiscoveryPort(data: unknown): number | null {
  if (typeof data !== 'object' || data === null) return null
  const port = (data as { port?: unknown }).port
  return typeof port === 'number' && Number.isInteger(port) && port > 0 ? port : null
}

function meta(name: string): string | null {
  return document.querySelector(`meta[name="${name}"]`)?.getAttribute('content') ?? null
}

function wsUrl(token: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss' : 'ws'
  return `${proto}://${window.location.host}/ws`
}

const token = ref(meta('relay-token') ?? '')
const hostMode = token.value !== ''
const discoveryPort = meta('relay-discovery-port')

const payload = ref<UiPayload | null>(null)
const connected = ref(false)
const freshness = ref<Freshness>('connecting')
let viewState: RelayViewState = { payload: null, freshness: 'connecting' }

function applyViewState(next: RelayViewState): void {
  viewState = next
  payload.value = next.payload
  freshness.value = next.freshness
}

let socket: WebSocket | null = null
let retries = 0
let retryTimer: number | null = null
let closed = false

async function refreshToken(): Promise<string> {
  try {
    const res = await fetch('/', { headers: { accept: 'text/html' } })
    const html = await res.text()
    const doc = new DOMParser().parseFromString(html, 'text/html')
    const fresh = doc.querySelector('meta[name="relay-token"]')?.getAttribute('content') ?? ''
    if (fresh) token.value = fresh
  } catch {
    // host fora do ar; mantém o token atual até a próxima tentativa
  }
  return token.value
}

async function tryDiscoverNewOrigin(): Promise<boolean> {
  if (!discoveryPort) return false
  try {
    const res = await fetch(`http://127.0.0.1:${discoveryPort}/`)
    if (!res.ok) return false
    const port = parseDiscoveryPort(await res.json())
    if (port === null) return false
    window.location.href = `http://127.0.0.1:${port}/`
    return true
  } catch {
    return false
  }
}

function scheduleReconnect(): void {
  if (closed) return
  const delay = Math.min(RECONNECT_BASE * 2 ** retries, RECONNECT_MAX)
  const attempt = retries
  retries += 1
  retryTimer = window.setTimeout(async () => {
    if (shouldAttemptDiscovery(attempt) && (await tryDiscoverNewOrigin())) return
    await refreshToken()
    connect()
  }, delay)
}

function connect(): void {
  if (closed) return
  if (!hostMode) return
  const current = token.value
  socket = new WebSocket(wsUrl(current), `relay.${current}`)
  socket.addEventListener('open', () => {
    connected.value = true
    retries = 0
  })
  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data)) as RelayMessage
    applyViewState(reduceRelayViewState(viewState, message))
  })
  socket.addEventListener('close', () => {
    connected.value = false
    applyViewState(reduceRelayViewState(viewState, { kind: 'disconnected' }))
    scheduleReconnect()
  })
  socket.addEventListener('error', () => {
    socket?.close()
  })
}

function disconnect(): void {
  closed = true
  if (retryTimer !== null) window.clearTimeout(retryTimer)
  socket?.close()
  socket = null
  connected.value = false
  applyViewState(reduceRelayViewState(viewState, { kind: 'disconnected' }))
}

async function apiGet<T>(path: string): Promise<T> {
  const res = await fetch(path, { headers: { 'X-Relay-Token': token.value } })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json() as Promise<T>
}

async function apiGetText(path: string): Promise<string> {
  const res = await fetch(path, { headers: { 'X-Relay-Token': token.value } })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.text()
}

export const relayClient: RelayClient = {
  hostMode,
  payload,
  connected,
  freshness,
  reconnectBackoff: RECONNECT_BASE,
  connect,
  disconnect,
}

export function useRelayClient(): RelayClient {
  return relayClient
}

export function apiGetJson<T>(path: string): Promise<T> {
  return apiGet<T>(path)
}

export function apiGetRaw(path: string): Promise<string> {
  return apiGetText(path)
}

export async function apiPostJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'X-Relay-Token': token.value, 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json() as Promise<T>
}
