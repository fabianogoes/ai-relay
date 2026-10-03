# Suite de conformidade

Casos neutros de linguagem que todo leitor do protocolo precisa passar. Hoje o
único leitor é o `core` do `relay-tui` (Rust), que executa esta suite em
`../relay-tui/tests/conformance.rs`; uma divergência quebra o CI.

Os casos são a referência do leitor, não o contrário. Um caso, porém, só vale se
concorda com o `docs/PROTOCOL.md`: se o caso e o protocolo discordarem, o
defeito é do caso.

## Formato de um caso

```text
<caso>/
  workspace/
    .orchestration/{BACKLOG,TODO,HANDOFF,CHANGELOG}.md
    .specs/*.md
  expected.json
```

- `workspace/` é o que um leitor encontraria num repositório Relay. Um registro
  ausente lê como texto vazio; cada spec entra com a chave `.specs/<nome>`.
- `expected.json` é o `RelayState` (os tipos estão em
  `../relay-tui/src/core/types.rs`) e **só ele**. Os detalhes de violação
  entram por inteiro, texto incluído, e a comparação é por igualdade estrutural. Uma chave opcional
  ausente (`spec` numa entrada, por exemplo) está ausente no JSON, não `null`.

## Casos

- `status-<status>`: um por estado derivado — `idle`, `backlog`, `ready`,
  `in_progress`, `blocked`, `done`, `inconsistent`.
- `check-<id>`: um por verificação de integridade do protocolo, cada um
  produzindo exatamente a violação que nomeia e mais nenhuma.

O runner é `../relay-tui/tests/conformance.rs` e exige o conjunto completo:
acrescentar ou remover um caso exige editar a lista dele.
