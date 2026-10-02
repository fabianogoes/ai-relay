# Suite de conformidade

Casos neutros de linguagem que todo leitor do protocolo precisa passar. O
`relay-core` (TypeScript) e o `core` do `relay-tui` (Rust) executam esta mesma
suite; uma divergência quebra o CI de quem divergiu. Decisão e motivo na
[ADR-0009](../../docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md);
o contrato do estado em
[ADR-0003](../../docs/adr/0003-contrato-do-estado-derivado.md).

O core em TypeScript é a referência enquanto a ADR-0009 não for substituída. Um
caso, porém, só vale se concorda com o `docs/PROTOCOL.md`: se o core e o
protocolo discordarem, o defeito é do core.

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
- `expected.json` é o `RelayState` da ADR-0003 e **só ele**: sem `environment`,
  que pertence ao host. Os detalhes de violação entram por inteiro, texto
  incluído, e a comparação é por igualdade estrutural. Uma chave opcional
  ausente (`spec` numa entrada, por exemplo) está ausente no JSON, não `null`.

## Casos

- `status-<status>`: um por estado derivado — `idle`, `backlog`, `ready`,
  `in_progress`, `blocked`, `done`, `inconsistent`. O `expected.json` de cada um
  é o `state` do fixture de mesmo nome em `../fixtures/`.
- `check-<id>`: um por verificação de integridade do protocolo, cada um
  produzindo exatamente a violação que nomeia e mais nenhuma.

O runner do TypeScript é `../relay-core/test/conformance.test.ts` e exige o
conjunto completo: acrescentar ou remover um caso exige editar a lista dele.
