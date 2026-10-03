# relay-tui

Painel de terminal, só de leitura, do estado Relay de um workspace. Pensado para
um split: o harness de um lado e o `relay-tui` do outro, mostrando handoff, TODO
e backlog mudarem sem recarregar nada.

```
 relay  ~/Developer/relay                     ● atualizado

╭ Handoff ─────────────────────────────── ● Em andamento ╮
│ B-001 · T-002 · claude-code · há 4 min                 │
│                                                        │
│ Objetivo  Validar os fixtures contra o protocolo.      │
│ Próximo   Rodar as verificações de integridade.        │
╰────────────────────────────────────────────────────────╯
╭ TODO ───────────────────────────────────────────── 1/3 ╮
│ ████████ ████████ ████████                             │
│                                                        │
│ ✓ T-001 Escrever a ADR-0003                            │
│ ● T-002 Validar os fixtures                            │
│ ○ T-003 Registrar no índice                            │
╰────────────────────────────────────────────────────────╯
```

Ele **só lê**: nunca escreve em `.specs/` nem em `.orchestration/` e nunca lança
processo. Quem escreve nos registros são as skills, dentro de um harness.

## Instalar

Um download, sem Node, sem repositório e sem gerenciador de pacotes. Os binários
estão na [página de releases](https://github.com/fabianogoes/ai-relay/releases),
um arquivo por alvo:

| Alvo | Arquivo | Situação |
| --- | --- | --- |
| macOS, Apple Silicon | `relay-tui-<versão>-aarch64-apple-darwin.tar.gz` | requisito |
| macOS, Intel | `relay-tui-<versão>-x86_64-apple-darwin.tar.gz` | requisito |
| Linux x64 e arm64 | `…-x86_64-unknown-linux-musl.tar.gz`, `…-aarch64-unknown-linux-musl.tar.gz` | melhor esforço |
| Windows | — | fora por enquanto |

**Melhor esforço** quer dizer que o build é tentado a cada release, mas o Linux
não é testado à mão: se falhar, o release sai sem ele. O macOS é o que é
verificado. O Windows saiu do build por enquanto; como retomá-lo está na
[ADR-0009](../../docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md).

No macOS:

```sh
tar -xzf relay-tui-<versão>-<alvo>.tar.gz
cd relay-tui-<versão>-<alvo>
./relay-tui --version
```

### Quarentena do macOS (Gatekeeper)

O binário **não é assinado nem notarizado**. Um arquivo baixado pelo navegador
recebe o atributo `com.apple.quarantine`, e o macOS não o executa. Duas saídas:

- baixar com `curl`, que não aplica a quarentena:

  ```sh
  curl -fLO https://github.com/fabianogoes/ai-relay/releases/download/relay-tui-v<versão>/relay-tui-<versão>-<alvo>.tar.gz
  ```

- ou remover o atributo depois de baixar:

  ```sh
  xattr -d com.apple.quarantine relay-tui
  ```

Cada arquivo tem um `.sha256` ao lado: `shasum -a 256 -c <arquivo>.sha256`.

Depois, ponha o `relay-tui` em qualquer pasta do `PATH`, por exemplo
`/usr/local/bin`.

## Usar

```sh
relay-tui                              # observa o diretório atual
relay-tui --workspace ~/Developer/repo # observa outro repositório
relay-tui --version
relay-tui --help
```

O painel abre em **Agora**, o que está em curso. `Tab` (ou `t`) abre a segunda
visão, **Histórico**, e volta; `r` relê o workspace por inteiro (só é preciso
quando o watcher não entrega eventos, num volume de rede por exemplo). `q` e
`Ctrl-C` saem de qualquer visão, na hora. `Esc` em Agora não sai sozinho: pergunta
`Sair?` no rodapé, e `Esc`, `Enter` ou `y` confirmam enquanto qualquer outra tecla
cancela. Em Histórico `Esc` volta. Trocar de visão, mover a seleção e recarregar
não escrevem nada: o painel continua só de leitura.

### Histórico

Quatro níveis, cada um aprofundando o anterior: as **specs** (da mais recente à
mais antiga, com `feitos/total` dos itens), os **itens de backlog** da spec, as
**tarefas** do item (os registros do changelog) e o **detalhe** da tarefa, com
Result, Evidence, Criteria e Decisions. Cada linha de lista ocupa uma linha e
termina em `…` quando não cabe; o detalhe quebra o texto e nunca corta. A lista
rola para manter a seleção visível e diz quantas linhas há acima e abaixo.

| Tecla | Efeito |
| --- | --- |
| `↑` `↓`, `j` `k`, roda do mouse | movem a seleção (no detalhe, rolam) |
| `PgUp` `PgDn` | movem uma página |
| `Enter` ou clique numa linha | abre o nível seguinte |
| `Esc` ou `Backspace` | voltam um nível; no de specs, voltam a Agora. Em Histórico `Esc` volta em vez de sair |
| `Tab`, `t` | alternam Agora e Histórico, no mesmo nível e na mesma seleção |
| `r` | relê o workspace |
| `q`, `Ctrl-C` | saem |

O mouse só é capturado enquanto Histórico está aberto, para Agora continuar
permitindo selecionar e copiar texto; a captura é desligada ao voltar a Agora,
ao sair e num erro interno. Com ela ligada, muitos terminais pedem uma tecla
(`Shift`, ou `Option` no iTerm2) para selecionar texto. Todo gesto do mouse tem
uma tecla equivalente. Abaixo de 40 colunas Histórico mostra só um aviso, e as
teclas continuam valendo.

- A cor segue o status: verde em andamento ou concluído, azul pronto ou
  disponível, amarelo bloqueado, vermelho inconsistente. Todo estado também
  aparece por extenso, nunca só pela cor.
- `● atualizando` aparece enquanto os arquivos mudam; 150 ms depois da última
  escrita o workspace é relido por inteiro e volta a `● atualizado`.
- Um diretório sem `.orchestration/` mostra "Não é um workspace Relay" e
  continua vigiando: o painel muda sozinho quando o diretório aparecer.
- Agora não rola: se faltar altura, o TODO corta em `+N itens`; abaixo de 40
  colunas só o cabeçalho e o status aparecem.

O fundo não é pintado: vem do terminal. As cores presumem um terminal escuro.

## Limites conhecidos

- Linhas de registro terminadas em CRLF (arquivos com `\r\n`, comuns no Windows
  com `autocrlf`) são ignoradas. É o comportamento do core em TypeScript, que o
  core em Rust reproduz de propósito até os dois serem corrigidos juntos.
- Um `SIGTERM` ou `SIGINT` vindo de fora do teclado não restaura o terminal;
  `Ctrl-C` pelo teclado, `q`, `Esc` e um erro interno restauram.
- Só tema escuro.

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
│   ├── app.rs             ← loop principal e estado da aplicação
│   ├── nav.rs             ← troca entre Agora e Histórico, Esc, mouse
│   ├── suggest.rs         ← sugestão do próximo passo
│   ├── theme.rs           ← cores e estilos
│   ├── core/              ← o leitor do protocolo, puro (sem disco)
│   │   ├── parse.rs       ← lê HANDOFF/TODO/BACKLOG/CHANGELOG/specs
│   │   ├── derive.rs      ← calcula o estado
│   │   ├── integrity.rs   ← checagens de integridade
│   │   ├── history.rs     ← changelog e títulos das specs
│   │   └── types.rs
│   ├── view/              ← o que é desenhado na tela
│   │   ├── mod.rs, cards.rs, specs.rs   ← Agora (cards, agrupado por spec)
│   │   ├── history.rs     ← tela Histórico
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
[ADR-0009](../../docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md).

## Publicar uma versão

1. Subir `version` em `Cargo.toml` e commitar junto com o `Cargo.lock`.
2. Criar e enviar a tag `relay-tui-v<versão>` (a mesma versão do `Cargo.toml`;
   o workflow recusa se forem diferentes).
3. O workflow `relay-tui-release` compila os dois alvos de macOS (requisito) e
   os de Linux (melhor esforço), e só publica se os de macOS passaram.
