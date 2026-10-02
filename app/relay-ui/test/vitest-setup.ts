// jsdom não implementa layout: `offsetParent` é sempre `null`, o que quebra
// o filtro de `focusables()` em focus-trap.ts (ele usa offsetParent!==null
// para distinguir elemento visível de escondido). Este shim aproxima o
// suficiente para os testes de foco: null quando `hidden`/`inert` está no
// próprio elemento ou em um ancestral, o parentElement caso contrário.
Object.defineProperty(HTMLElement.prototype, 'offsetParent', {
  configurable: true,
  get(this: HTMLElement) {
    let node: HTMLElement | null = this
    while (node) {
      if (node.hidden || node.inert) return null
      node = node.parentElement
    }
    return this.parentElement
  },
})
