import { describe, expect, it } from 'vitest'
import { DISCOVERY_AFTER_RETRIES, parseDiscoveryPort, shouldAttemptDiscovery } from '../src/lib/relay-client.ts'

describe('shouldAttemptDiscovery: só recorre ao farol depois de falhas persistentes na mesma origem (A-010)', () => {
  it('tentativas abaixo do limite não acionam descoberta', () => {
    for (let attempt = 0; attempt < DISCOVERY_AFTER_RETRIES; attempt++) {
      expect(shouldAttemptDiscovery(attempt)).toBe(false)
    }
  })

  it('a partir do limite, aciona descoberta', () => {
    expect(shouldAttemptDiscovery(DISCOVERY_AFTER_RETRIES)).toBe(true)
    expect(shouldAttemptDiscovery(DISCOVERY_AFTER_RETRIES + 5)).toBe(true)
  })
})

describe('parseDiscoveryPort: só aceita a forma exata { port: <inteiro positivo> }', () => {
  it('aceita uma porta inteira positiva', () => {
    expect(parseDiscoveryPort({ port: 41234 })).toBe(41234)
  })

  it('rejeita ausência de porta, tipos errados e valores inválidos', () => {
    expect(parseDiscoveryPort({})).toBeNull()
    expect(parseDiscoveryPort({ port: '41234' })).toBeNull()
    expect(parseDiscoveryPort({ port: 0 })).toBeNull()
    expect(parseDiscoveryPort({ port: -1 })).toBeNull()
    expect(parseDiscoveryPort({ port: 1.5 })).toBeNull()
    expect(parseDiscoveryPort(null)).toBeNull()
    expect(parseDiscoveryPort('41234')).toBeNull()
  })
})
