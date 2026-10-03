# Interface do Relay

Este diretório é a interface do Relay: o `relay-tui`, um painel de terminal só
de leitura, em Rust, que se instala por um único download. Leia `AGENTS.md`
antes de mudar qualquer coisa aqui.

- Instalar e usar: [`../docs/TUI.md`](../docs/TUI.md).
- Compilar, testar e publicar: [`relay-tui/README.md`](relay-tui/README.md).
- O desenho da tela: [`relay-tui/DESIGN.md`](relay-tui/DESIGN.md).

## Estrutura

```text
app/
  AGENTS.md      regras desta pasta
  relay-tui/     o painel (crate Rust); a estrutura interna está no README dele
```
