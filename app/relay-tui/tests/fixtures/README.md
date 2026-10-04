# Fixtures

Workspaces Relay em miniatura, cada um com o estado que o `core` tem de derivar
dele (golden file). `../derive_state.rs` deriva cada caso e compara com o
`expected.json`; uma divergência quebra o CI. Os workspaces `status-*` também
servem de entrada para os testes da view (`view_snapshots`, `view_smoke`,
`view_semantics`, `suggest`, `e2e_pty`).

Os casos são a referência do leitor, não o contrário. Um caso, porém, só vale se
concorda com o `docs/PROTOCOL.md`: se o caso e o protocolo discordarem, o
defeito é do caso.

## Formato de um caso

```text
<caso>/
  workspace/
    .orchestration/{BACKLOG,TODO,HANDOFF,CHANGELOG}.md
    .orchestration/changelog/<AAAAMMDD-NNN>.md
    .specs/*.md
  expected.json
```

- `workspace/` é o que um leitor encontraria num repositório Relay. Um registro
  ausente lê como texto vazio; cada spec entra com a chave `.specs/<nome>` e cada changelog por spec com
  `changelog/<nome>`. `CHANGELOG.md` é o legado.
- `expected.json` é o `RelayState` (os tipos estão em
  `../../src/core/types.rs`) e **só ele**. Os detalhes de violação
  entram por inteiro, texto incluído, e a comparação é por igualdade estrutural. Uma chave opcional
  ausente (`spec` numa entrada, por exemplo) está ausente no JSON, não `null`.

## Casos

- `status-<status>`: um por estado derivado — `idle`, `backlog`, `ready`,
  `in_progress`, `blocked`, `done`, `inconsistent`.
- `check-<id>`: um por verificação de integridade do protocolo, cada um
  produzindo exatamente a violação que nomeia e mais nenhuma.
- `structure-<caso>`: a estrutura do changelog e a leitura estrita — por spec,
  legado junto com por spec, spec fechada, entrada arquivada, descarte, dispensa,
  CRLF e `Status` dentro de `Context`.
- `../data/repository-history/workspace` é a cópia congelada lida por
  `history.rs`; ela fica fora de `fixtures/` porque não é um caso golden com
  `expected.json`, e os testes de histórico não dependem dos registros vivos.

O runner é `../derive_state.rs` e exige o conjunto completo:
acrescentar ou remover um caso exige editar a lista dele.
