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
estão na [página de releases](https://github.com/fabianogoes/relay/releases),
um arquivo por alvo:

| Alvo | Arquivo | Situação |
| --- | --- | --- |
| macOS, Apple Silicon | `relay-tui-<versão>-aarch64-apple-darwin.tar.gz` | requisito |
| macOS, Intel | `relay-tui-<versão>-x86_64-apple-darwin.tar.gz` | requisito |
| Linux x64 e arm64 | `…-x86_64-unknown-linux-musl.tar.gz`, `…-aarch64-unknown-linux-musl.tar.gz` | melhor esforço |
| Windows x64 | `…-x86_64-pc-windows-msvc.zip` | melhor esforço |

**Melhor esforço** quer dizer que o build é tentado a cada release, mas Linux e
Windows não são testados à mão: se um alvo falhar, o release sai sem ele. O
macOS é o que é verificado.

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
  curl -fLO https://github.com/fabianogoes/relay/releases/download/relay-tui-v<versão>/relay-tui-<versão>-<alvo>.tar.gz
  ```

- ou remover o atributo depois de baixar:

  ```sh
  xattr -d com.apple.quarantine relay-tui
  ```

No Windows o SmartScreen avisa que o aplicativo é desconhecido: "Mais
informações" e "Executar assim mesmo".

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

Teclas: `q`, `Esc` ou `Ctrl-C` saem. Não há mais nada para apertar: é um painel
para olhar.

- A cor segue o status: verde em andamento ou concluído, azul pronto ou
  disponível, amarelo bloqueado, vermelho inconsistente. Todo estado também
  aparece por extenso, nunca só pela cor.
- `● atualizando` aparece enquanto os arquivos mudam; 150 ms depois da última
  escrita o workspace é relido por inteiro e volta a `● atualizado`.
- Um diretório sem `.orchestration/` mostra "Não é um workspace Relay" e
  continua vigiando: o painel muda sozinho quando o diretório aparecer.
- Sem rolagem: se faltar altura, o TODO corta em `+N itens`; abaixo de 40
  colunas só o cabeçalho e o status aparecem.

O fundo não é pintado: vem do terminal. As cores presumem um terminal escuro.

## Limites conhecidos

- Linhas de registro terminadas em CRLF (arquivos com `\r\n`, comuns no Windows
  com `autocrlf`) são ignoradas. É o comportamento do core em TypeScript, que o
  core em Rust reproduz de propósito até os dois serem corrigidos juntos.
- Um `SIGTERM` ou `SIGINT` vindo de fora do teclado não restaura o terminal;
  `Ctrl-C` pelo teclado, `q`, `Esc` e um erro interno restauram.
- Só tema escuro.

## Compilar do código

Precisa de Rust (`rustup`, versão estável).

```sh
cd app/relay-tui
cargo run -- --workspace /caminho/do/repo   # roda
cargo test                                  # suíte inteira
scripts/package.sh aarch64-apple-darwin     # gera dist/relay-tui-<versão>-<alvo>.tar.gz
```

A pasta `../conformance/` tem os casos que o core em Rust e o `relay-core` em
TypeScript precisam passar; ver o `README.md` dela e a
[ADR-0009](../../docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md).

## Publicar uma versão

1. Subir `version` em `Cargo.toml` e commitar junto com o `Cargo.lock`.
2. Criar e enviar a tag `relay-tui-v<versão>` (a mesma versão do `Cargo.toml`;
   o workflow recusa se forem diferentes).
3. O workflow `relay-tui-release` compila os dois alvos de macOS (requisito) e
   os de Linux e Windows (melhor esforço), e só publica se os de macOS passaram.
