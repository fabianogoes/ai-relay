# relay-tui: o painel de terminal

O `relay-tui` é um painel só de leitura para deixar num split do terminal: o
harness de um lado, o painel do outro. Handoff, TODO e backlog mudam na tela
conforme os arquivos mudam, sem recarregar nada. Ele nunca escreve nos
registros: quem escreve são as skills, dentro de um harness.

É um único binário, sem Node e sem clonar o repositório.

## Instalar

As versões ficam em
[Releases](https://github.com/fabianogoes/ai-relay/releases), um arquivo por
alvo:

| Alvo | Arquivo | Situação |
| --- | --- | --- |
| macOS, Apple Silicon | `relay-tui-<versão>-aarch64-apple-darwin.tar.gz` | requisito |
| macOS, Intel | `relay-tui-<versão>-x86_64-apple-darwin.tar.gz` | requisito |
| Linux x64 e arm64 | `…-x86_64-unknown-linux-musl.tar.gz`, `…-aarch64-unknown-linux-musl.tar.gz` | melhor esforço |
| Windows | — | fora por enquanto |

No macOS, `uname -m` diz qual é o seu (`arm64` ou `x86_64`). Baixando com
`curl`, o macOS não bloqueia o arquivo:

```sh
curl -fLO https://github.com/fabianogoes/ai-relay/releases/download/relay-tui-v<versão>/relay-tui-<versão>-aarch64-apple-darwin.tar.gz
tar -xzf relay-tui-<versão>-aarch64-apple-darwin.tar.gz
mkdir -p ~/.local/bin
mv relay-tui-<versão>-aarch64-apple-darwin/relay-tui ~/.local/bin/
```

Garanta que `~/.local/bin` está no seu `PATH`, e confira com `relay-tui --version`.

**Baixou pelo navegador?** O binário não é assinado, e o macOS recusa abrir um
arquivo baixado assim ("não pode ser aberto"). Libere com:

```sh
xattr -d com.apple.quarantine ~/.local/bin/relay-tui
```

Cada arquivo tem um `.sha256` ao lado: `shasum -a 256 -c <arquivo>.sha256`.

## Abrir num split

Abra um painel ao lado do harness, entre no repositório Relay e rode
`relay-tui`:

| Terminal | Como abrir o painel ao lado |
| --- | --- |
| **Warp** | `Cmd+D` divide para a direita (`Cmd+Shift+D`, para baixo). |
| **iTerm2** | `Cmd+D` divide para a direita (`Cmd+Shift+D`, para baixo). |
| **tmux** | `tmux split-window -h relay-tui`, ou `Ctrl-b %` e depois `relay-tui`. |
| **Terminal.app** | Não tem split. Use o Warp, o iTerm2 ou o tmux (`brew install tmux`). |

```sh
cd ~/Developer/meu-repositorio
relay-tui
```

Deixe o harness no outro painel e trabalhe como sempre: o painel acompanha.

## Usar

```sh
relay-tui                              # observa o diretório atual
relay-tui --workspace ~/Developer/repo # observa outro repositório
relay-tui --version
relay-tui --help
```

Teclas: `q`, `Esc` ou `Ctrl-C` saem. Não há mais nada para apertar: é um painel
para olhar.

### O que a tela mostra

- **Handoff**: o status (`Em andamento`, `Bloqueado`), quem deixou e quando, o
  objetivo e o próximo passo. Bloqueado, mostra também o bloqueio e a condição
  de retomada.
- **TODO**: uma barra com um segmento por subtarefa e a lista, com `✓` feita,
  `●` em andamento, `○` disponível, `◌` esperando outra subtarefa e `!`
  bloqueada.
- **Backlog**: quantos itens estão feitos, em curso, disponíveis ou aguardando.
- **`● atualizando` / `● atualizado`**, no canto: o painel espera os arquivos
  pararem de mudar (150 ms) e lê tudo de novo.
- **`Inconsistente`**, em vermelho: os registros se contradizem, e o painel
  lista cada violação. É o mesmo estado que a skill `relay-status` reporta.

A cor segue o status (verde em andamento ou concluído, azul pronto ou
disponível, amarelo bloqueado, vermelho inconsistente), mas todo estado também
aparece por extenso, nunca só pela cor.

Se faltar altura, o TODO corta em `+N itens`. Abaixo de 40 colunas só o
cabeçalho e o status aparecem. Um diretório sem `.orchestration/` mostra "Não é
um workspace Relay" e passa a mostrar o estado assim que o Relay for instalado
ali (`relay-setup`).

O fundo não é pintado: vem do terminal. As cores presumem um terminal escuro.

## Limites conhecidos

- Linhas de registro terminadas em CRLF (comuns no Windows com `autocrlf`) são
  ignoradas.
- Um `SIGTERM` vindo de fora do teclado não restaura o terminal; `q`, `Esc`,
  `Ctrl-C` e um erro interno restauram.
- Só tema escuro.

## A partir do código

Sem baixar nada, com o Rust instalado (`rustup`):

```sh
cd app/relay-tui
cargo build --release
./target/release/relay-tui --workspace /caminho/do/repo
```
