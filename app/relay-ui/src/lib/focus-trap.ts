const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

function focusables(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (el) => el.offsetParent !== null || el === document.activeElement,
  )
}

function inertSiblings(root: HTMLElement): Array<{ el: HTMLElement }> {
  const parent = root.parentElement
  if (!parent) return []
  const inerted: Array<{ el: HTMLElement }> = []
  for (const el of Array.from(parent.children) as HTMLElement[]) {
    if (el === root || root.contains(el) || el.contains(root)) continue
    if (el.inert) continue
    el.inert = true
    inerted.push({ el })
  }
  return inerted
}

export interface DialogHandle {
  release(): void
}

export function mountDialog(root: HTMLElement, initialFocus: HTMLElement | null = null): DialogHandle {
  const originator = document.activeElement as HTMLElement | null
  const inerted = inertSiblings(root)

  const target = initialFocus ?? focusables(root)[0] ?? root
  target.focus()

  const onKeydown = (event: KeyboardEvent): void => {
    if (event.key !== 'Tab') return
    const els = focusables(root)
    if (els.length === 0) return
    const first = els[0]
    const last = els[els.length - 1]
    const active = document.activeElement as HTMLElement | null
    if (event.shiftKey) {
      if (active === first || !root.contains(active)) {
        event.preventDefault()
        last.focus()
      }
    } else if (active === last || !root.contains(active)) {
      event.preventDefault()
      first.focus()
    }
  }

  root.addEventListener('keydown', onKeydown)

  return {
    release() {
      root.removeEventListener('keydown', onKeydown)
      for (const { el } of inerted) el.inert = false
      originator?.focus()
    },
  }
}