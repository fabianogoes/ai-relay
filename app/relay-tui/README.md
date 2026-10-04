# relay-tui

Painel de terminal, só de leitura, do estado Relay de um workspace. **Instalar,
abrir num split, as teclas, as telas e os limites estão em
[`docs/TUI.md`](../../docs/TUI.md)** (português: [`docs/TUI.pt-BR.md`](../../docs/TUI.pt-BR.md)),
que é a única fonte de uso. Este README é do desenvolvimento do crate: estrutura,
compilar, testar e publicar.

Ele **só lê**: nunca escreve em `.specs/` nem em `.orchestration/` e nunca lança
processo. Quem escreve nos registros são as skills, dentro de um harness.

## Estrutura

```
app/relay-tui/
├── Cargo.toml / Cargo.lock
├── README.md
├── DESIGN.md              ← como a tela deve parecer (autoridade sobre a view)
├── scripts/package.sh     ← empacota o binário para o release
├── dist/                  ← tarballs gerados (.tar.gz + .sha256), fora do git
├── src/
│   ├── main.rs, cli.rs    ← entrada e flags (--workspace, --version…)
│   ├── app.rs             ← loop principal, Configurações e idioma da sessão
│   ├── nav.rs             ← troca entre Agora e Histórico, Esc, mouse
│   ├── suggest.rs         ← sugestão do próximo passo
│   ├── theme.rs           ← cores e estilos
│   ├── core/              ← o leitor do protocolo, puro (sem disco)
│   │   ├── parse.rs       ← lê HANDOFF/TODO/BACKLOG/changelog/specs
│   │   ├── derive.rs      ← calcula o estado
│   │   ├── integrity.rs   ← checagens de integridade
│   │   ├── history.rs     ← changelog e títulos das specs
│   │   └── types.rs
│   ├── view/              ← o que é desenhado na tela
│   │   ├── mod.rs, cards.rs, specs.rs   ← Agora (cards, agrupado por spec)
│   │   ├── history.rs     ← tela Histórico
│   │   ├── settings.rs    ← tela Configurações
│   │   └── hint.rs, text.rs
│   └── workspace/         ← leitura do disco e watcher (watch.rs, debounce.rs)
└── tests/                 ← testes de integração
    ├── derive_state.rs    ← cada fixture deriva o seu expected.json
    ├── fixtures/          ← workspaces de exemplo + estado esperado
    └── snapshots/*.txt    ← o layout esperado em cada largura e altura
```

Para mudar o que aparece na tela, os arquivos são os de `src/view/` e o
`src/theme.rs`. A regra de qual estado mostrar fica em `src/core/`, conferida pela suíte
de `tests/fixtures/`. O desenho da tela está em
[`DESIGN.md`](DESIGN.md).

## Compilar do código

Precisa de Rust (`rustup`, versão estável).

```sh
cd app/relay-tui
cargo run -- --workspace /caminho/do/repo   # roda
cargo test                                  # suíte inteira
scripts/package.sh aarch64-apple-darwin     # gera dist/relay-tui-<versão>-<alvo>.tar.gz
```

A pasta `tests/fixtures/` tem os casos que o core precisa derivar
(`tests/derive_state.rs`); ver o `README.md` dela e a
[ADR-0003](../../docs/adr/0003-relay-tui-observador-de-terminal-em-rust.md).

## Publicar uma versão

1. Subir `version` em `Cargo.toml` e commitar junto com o `Cargo.lock`.
2. Criar e enviar a tag `relay-tui-v<versão>` (a mesma versão do `Cargo.toml`;
   o workflow recusa se forem diferentes).
3. O workflow `relay-tui-release` compila os dois alvos de macOS (requisito) e
   os de Linux (melhor esforço), e só publica se os de macOS passaram.
