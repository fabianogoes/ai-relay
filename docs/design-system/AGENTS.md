# Instruções desta pasta

Esta pasta é o design do Relay. A única interface é o painel de terminal
`relay-tui`, descrito em `terminal.md`.

## Ordem de mudança

Cor, rótulo, layout ou tecla mudam **primeiro** em `terminal.md`, depois no
código (`app/relay-tui/src/theme.rs` e `app/relay-tui/src/view/`). O documento
vence sobre o código: uma divergência entre os dois é defeito de um deles e se
resolve aqui primeiro.
