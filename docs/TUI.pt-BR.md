# relay-tui: o painel de terminal

**[English](TUI.md)** · Português

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

Peça ao agente. No Claude Code a skill `relay-tui-split` é o comando
`/relay-tui-split`; no Codex e no OpenCode, peça `Use relay-tui-split`. Ela
descobre em qual terminal o harness está rodando e abre um painel à direita com
`relay-tui --workspace '<seu repositório>'`. É uma skill do pacote, fora do
protocolo do Relay: não toca registro nenhum, não instala nada e nunca tenta um
terminal diferente do que detectou.

| Terminal | Detectado por | O que ela roda |
| --- | --- | --- |
| **tmux** | `TMUX` | `tmux split-window -h` |
| **zellij** | `ZELLIJ` | `zellij action new-pane --direction right --` |
| **WezTerm** | `WEZTERM_PANE` | `wezterm cli split-pane --right` |
| **kitty** | `KITTY_WINDOW_ID` | `kitten @ launch --location=vsplit` (exige `allow_remote_control yes` no `kitty.conf`) |
| **iTerm2** | `TERM_PROGRAM=iTerm.app` | AppleScript: divide na vertical a sessão que a rodou e digita o comando |
| **Warp** | `TERM_PROGRAM=WarpTerminal` | AppleScript pelo System Events: `Cmd+D` e depois digita o comando |

Vale a primeira que casar, e os multiplexadores vêm antes dos emuladores porque
rodam dentro deles: tmux dentro do iTerm2 abre um painel do tmux. Qualquer outro
(Terminal.app, o terminal embutido de um editor) recebe a instrução manual
abaixo.

A sandbox do harness pode atrapalhar, porque o script fala com o terminal (o
socket do tmux, Apple Events). Verificado de dentro do tmux: o Claude Code e o
OpenCode abrem o painel. **A sandbox padrão do Codex bloqueia o socket do tmux**
(`error connecting to ... (Operation not permitted)`): o script então imprime a
instrução manual e sai com `2`, e o painel só abre se o script rodar fora da
sandbox (por exemplo `codex exec -s danger-full-access`, ou aprovando o comando
quando o Codex pedir). As variáveis de ambiente do terminal chegam ao shell do
harness em todos os casos verificados.

No macOS a primeira execução pode pedir permissão, e negá-la faz a skill cair na
instrução manual:

- **iTerm2**: permita que o seu terminal controle o iTerm2 em *Ajustes do Sistema
  > Privacidade e Segurança > Automação*.
- **Warp**: não tem comando de split, então a skill manda `Cmd+D` pelo System
  Events, o que exige *Privacidade e Segurança > Acessibilidade*. As teclas vão
  para o app em primeiro plano, então a skill confere antes que ele é o Warp e,
  se não for, para em vez de digitar em outro aplicativo.

Fora do harness, `sh skills/relay-tui-split/scripts/open-split.sh [dir]` faz o
mesmo, e `--dry-run` imprime o que rodaria sem rodar. Código de saída: `0`
abriu (ou `--dry-run`), `1` o `relay-tui` não está no `PATH` (imprime o link
deste guia e não instala nada), `2` terminal não reconhecido ou o split falhou
(imprime o que fazer à mão).

À mão: abra um painel ao lado do harness, entre no repositório Relay e rode
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

### O que a tela mostra

- **Handoff**: o status (`Em andamento`, `Bloqueado`), quem deixou e quando, o
  objetivo e o próximo passo. Bloqueado, mostra também o bloqueio e a condição
  de retomada.
- **TODO**: uma barra com um segmento por subtarefa e a lista, com `✓` feita,
  `●` em andamento, `○` disponível, `◌` esperando outra subtarefa e `!`
  bloqueada.
- **Backlog**: quantos itens estão feitos, em curso, disponíveis ou aguardando.
- **Próximo passo**, uma linha acima do rodapé: o que fazer em seguida e qual
  skill chamar (por exemplo `Retome T-002 de B-001 com relay-session.`), tirado
  dos mesmos registros. O painel só sugere: nunca executa nada.
- **`● atualizando` / `● atualizado`**, no canto: o painel espera os arquivos
  pararem de mudar (150 ms) e lê tudo de novo.
- **`Inconsistente`**, em vermelho: os registros se contradizem, e o painel
  lista cada violação. É o mesmo estado que a skill `relay-status` reporta.

A cor segue o status (verde em andamento ou concluído, azul pronto ou
disponível, amarelo bloqueado, vermelho inconsistente), mas todo estado também
aparece por extenso, nunca só pela cor.

Se faltar altura, a linha de próximo passo some primeiro, depois o Handoff se
compacta e o TODO corta em `+N itens`. Abaixo de 40 colunas só o
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
