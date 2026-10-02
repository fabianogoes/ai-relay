import { describe, expect, it } from 'vitest'
import { mountDialog } from '../src/lib/focus-trap.ts'

function buildDialog(): { outside: HTMLButtonElement; root: HTMLElement; first: HTMLButtonElement; last: HTMLButtonElement } {
  document.body.innerHTML = ''
  const outside = document.createElement('button')
  outside.textContent = 'fora do dialogo'
  document.body.appendChild(outside)

  const root = document.createElement('div')
  root.setAttribute('role', 'dialog')
  const first = document.createElement('button')
  first.textContent = 'primeiro'
  const middle = document.createElement('button')
  middle.textContent = 'meio'
  const last = document.createElement('button')
  last.textContent = 'ultimo'
  root.append(first, middle, last)
  document.body.appendChild(root)

  return { outside, root, first, last }
}

function tab(target: HTMLElement, shiftKey = false): void {
  target.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey, bubbles: true, cancelable: true }))
}

describe('foco não escapa da contenção do diálogo (A-013)', () => {
  it('Tab no último elemento volta pro primeiro, nunca sai do diálogo', () => {
    const { root, first, last } = buildDialog()
    const dialog = mountDialog(root)

    last.focus()
    expect(document.activeElement).toBe(last)
    tab(last)
    expect(document.activeElement).toBe(first)

    dialog.release()
  })

  it('Shift+Tab no primeiro elemento vai pro último, nunca escapa pro que está atrás', () => {
    const { root, first, last } = buildDialog()
    const dialog = mountDialog(root)

    first.focus()
    tab(first, true)
    expect(document.activeElement).toBe(last)

    dialog.release()
  })

  it('foco inicial e restauração ao originador', () => {
    const { outside, root, first } = buildDialog()
    outside.focus()
    expect(document.activeElement).toBe(outside)

    const dialog = mountDialog(root, first)
    expect(document.activeElement).toBe(first)

    dialog.release()
    expect(document.activeElement).toBe(outside)
  })
})
