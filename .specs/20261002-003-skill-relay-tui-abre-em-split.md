# 20261002-003 - Skill que abre o relay-tui em um split

## Problem

Hoje a pessoa abre o split a mao e roda `relay-tui`, e o jeito muda a cada
terminal (Warp, iTerm2, tmux...). Uma skill do Relay poderia fazer isso sozinha:
detectar em qual terminal esta e abrir o painel ao lado do harness.

## Scope

- Nova skill `relay-tui-split` em `skills/` (em ingles, como as demais, e com menos de
  40 linhas), neutra de harness e que nao le nem escreve nenhum registro. No
  Claude Code ela vira o comando `/relay-tui-split`; nos outros harnesses, um pedido
  como "use relay-tui-split". A skill so manda rodar o script e relatar o resultado.
- Um script POSIX `sh` (sem bashismos; roda no `/bin/sh` do macOS e no `dash`)
  em `skills/relay-tui-split/scripts/open-split.sh` detecta o terminal pelas variaveis
  de ambiente e abre um painel a direita rodando
  `relay-tui --workspace '<dir>'`, com `<dir>` absoluto: o argumento opcional do
  script ou, sem ele, o diretorio corrente. O caminho vai sempre citado (pode ter
  espacos) e o `--workspace` dispensa depender do diretorio inicial do painel.
- Deteccao na ordem da tabela; vale a primeira que casar. Multiplexadores vem
  antes dos emuladores porque rodam dentro deles (tmux dentro do iTerm2 abre um
  painel do tmux):

  | Ordem | Terminal | Detecao | Acao |
  | --- | --- | --- | --- |
  | 1 | tmux | `TMUX` | `tmux split-window -h <cmd>` |
  | 2 | zellij | `ZELLIJ` | `zellij action new-pane --direction right -- <cmd>` |
  | 3 | WezTerm | `WEZTERM_PANE` | `wezterm cli split-pane --right --pane-id "$WEZTERM_PANE" -- <cmd>` |
  | 4 | kitty | `KITTY_WINDOW_ID` | `kitten @ launch --location=vsplit <cmd>` (exige controle remoto ligado) |
  | 5 | iTerm2 | `TERM_PROGRAM=iTerm.app` | AppleScript: na sessao de `ITERM_SESSION_ID`, `split vertically with default profile` e `write text "<cmd>"` na sessao nova |
  | 6 | Warp | `TERM_PROGRAM=WarpTerminal` | AppleScript via System Events: confere que o Warp e o app em primeiro plano, `Cmd+D`, espera o painel e digita `<cmd>` com Enter (exige permissao de Acessibilidade) |

  No iTerm2, `write text` roda o comando no shell de login do painel novo, com o
  `PATH` do usuario; `command` do AppleScript nao passaria pelo shell. Mirar a
  sessao por `ITERM_SESSION_ID` evita dividir outra janela se o foco mudou.
- Terminal nao reconhecido (Terminal.app, o terminal embutido de um editor...),
  ou comando de split que falha (kitty sem controle remoto, permissao negada no
  macOS, Warp fora do primeiro plano): imprime a instrucao manual daquele
  terminal, ou a generica (abrir um painel e rodar `<cmd>`). Nunca tenta outro
  terminal da tabela.
- Sem `relay-tui` no `PATH` (`command -v`): nao instala nada; imprime o link do
  guia no GitHub (`docs/TUI.md` do repositorio publico, pois numa instalacao do
  pacote o arquivo nao esta no repositorio do usuario).
- `--dry-run` imprime o comando que executaria e sai com zero, sem executar; a
  verificacao do `PATH` vale tambem nele.
- Codigos de saida: `0` abriu o painel (ou `--dry-run`); `1` sem `relay-tui` no
  `PATH`; `2` terminal nao reconhecido ou split falhou (instrucao manual
  impressa). A skill relata a pessoa o que o script imprimiu.
- Testes do script em `.agents/tests/open-split.test.sh` (ferramenta do
  repositorio, fora de `skills/`, nunca superficie de pacote): montam o ambiente
  de cada terminal, um `PATH` com um `relay-tui` falso e comparam a saida de
  `--dry-run`.
- Este repositorio passa a usar a skill pelos links por item de `.agents/skills/`
  e `.claude/skills/`, como as demais (ADR-0002).
- Docs: o laco de skills dos comandos de instalacao (`docs/INSTALL.md` e
  `docs/INSTALL.pt-BR.md`), a tabela de skills de `README.md` e `README.pt-BR.md`
  com a nota de que a skill esta fora do protocolo, a secao "Open it in a split"
  de `docs/TUI.md` e `docs/TUI.pt-BR.md` (incluindo as permissoes de Automacao e
  Acessibilidade do macOS) e, no `AGENTS.md`, a linha das skills deixa de dizer
  "all five".

## Non-goals

- Instalar ou atualizar o binario; fechar, focar ou gerenciar o ciclo de vida do
  painel; evitar abrir um segundo painel se ja houver um.
- Terminais fora da tabela e Windows. Warp e iTerm2 fora do macOS caem na
  instrucao manual.
- Qualquer leitura ou escrita nos cinco registros; mudar `docs/PROTOCOL.md`.

## Decisions

**A skill e do pacote, nao do protocolo.** (ADR-0011.) Ela nao interpreta nem muda registros
(`docs/PROTOCOL.md`); so abre uma janela. Por isso vive em `skills/` (vai nos
manifestos, que publicam `./skills/` inteiro, e nas instalacoes), mas e
documentada como fora do protocolo, e o `docs/PROTOCOL.md` nao muda: sua lista de
responsabilidades cobre as skills que tocam registros.

**Deteccao por variavel de ambiente.** A skill roda no shell do harness, que e
filho do terminal e herda `TMUX`, `TERM_PROGRAM` e afins. Que o Claude Code, o
Codex e o OpenCode realmente repassam isso, e que a sandbox de cada um deixa o
script falar com o tmux ou mandar Apple Events, e uma hipotese a verificar, nao
um fato: e o que o A-004 comprova.

**O Warp e o caso mais fragil.** Ele nao tem comando de split nem AppleScript
proprio; so resta simular `Cmd+D` (atalho confirmado pelo dono do projeto) pelo
System Events, que exige permissao de Acessibilidade e quebra se o atalho mudar.
Teclas simuladas vao para o app em primeiro plano, entao o script confere antes
que ele e o Warp; se nao for, para e imprime a instrucao manual em vez de digitar
um comando em outro aplicativo. Fica na tabela porque e o terminal do dono do
projeto.

**O comando leva `--workspace` absoluto.** Cada terminal abre o painel novo num
diretorio diferente (o do processo, o padrao do perfil, o home); passar o caminho
torna a acao igual em todos e testavel no `--dry-run`.

**O script e testavel sem terminal.** Recebe o ambiente por variaveis e tem
`--dry-run`; os testes montam o ambiente de cada terminal e comparam o comando.
Ficam fora de `skills/` para nao serem publicados com o pacote.

## Acceptance criteria

- A-001 - Para cada terminal da tabela, o script, com as variaveis de ambiente
  daquele terminal, imprime em `--dry-run` a acao da tabela com
  `--workspace` absoluto e citado (inclusive para um caminho com espaco), e com
  `TMUX` e `TERM_PROGRAM=iTerm.app` juntos escolhe o tmux, verificado por
  `.agents/tests/open-split.test.sh`.
- A-002 - Sem terminal reconhecido o script imprime a instrucao manual e sai com
  `2`; sem `relay-tui` no `PATH` imprime o link do guia e sai com `1`; nunca
  instala nada, verificado pelos mesmos testes.
- A-003 - A skill `relay-tui-split` existe em `skills/` com menos de 40 linhas, tem os
  links por item em `.agents/skills/` e `.claude/skills/` e consta nos lacos de
  instalacao (EN e PT-BR), na tabela de skills de `README.md` e `README.pt-BR.md`
  e na secao de split de `docs/TUI.md` e `docs/TUI.pt-BR.md`.
- A-004 - Com o Claude Code, no tmux, no iTerm2 e no Warp (na maquina do dono), a
  skill abre um painel a direita rodando o `relay-tui` no repositorio atual; com
  o Codex e com o OpenCode, ao menos no tmux, a skill abre o painel ou a
  limitacao encontrada (por exemplo, a sandbox) fica registrada em `docs/TUI.md`
  e `docs/TUI.pt-BR.md`. A evidencia registra, por harness e terminal, o comando
  executado e o resultado, incluindo que o ambiente do terminal chega ao shell do
  harness.
- A-005 - A skill e o script nao leem nem escrevem nenhum dos cinco registros,
  verificado por inspecao do script e da skill.

## Backlog candidates

- B-001: Script `open-split.sh` com deteccao do terminal e `--dry-run`, testado por terminal.
- B-002: Skill `relay-tui-split`, links do repositorio e documentacao de instalacao.
- B-003: Verificacao nos terminais reais (tmux, iTerm2, Warp) com evidencia.
