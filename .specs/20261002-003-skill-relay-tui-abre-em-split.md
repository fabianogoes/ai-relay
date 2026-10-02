# 20261002-003 - Skill que abre o relay-tui em um split

## Problem

Hoje a pessoa abre o split a mao e roda `relay-tui`, e o jeito muda a cada
terminal (Warp, iTerm2, tmux...). Uma skill do Relay poderia fazer isso sozinha:
detectar em qual terminal esta e abrir o painel ao lado do harness.

## Scope

- Nova skill `relay-tui` em `skills/`, neutra de harness e que nao escreve nenhum
  registro. No Claude Code ela vira o comando `/relay-tui`; nos outros harnesses,
  um pedido como "use relay-tui".
- Um script `skills/relay-tui/scripts/open-split.sh` detecta o terminal pelas
  variaveis de ambiente e abre um painel a direita rodando `relay-tui` no
  diretorio atual:

  | Terminal | Detecao | Acao |
  | --- | --- | --- |
  | tmux | `TMUX` | `tmux split-window -h` |
  | zellij | `ZELLIJ` | `zellij action new-pane --direction right` |
  | WezTerm | `WEZTERM_PANE` | `wezterm cli split-pane --right` |
  | kitty | `KITTY_WINDOW_ID` | `kitten @ launch --location=vsplit` (exige controle remoto) |
  | iTerm2 | `TERM_PROGRAM=iTerm.app` | AppleScript: `split vertically` |
  | Warp | `TERM_PROGRAM=WarpTerminal` | AppleScript: `Cmd+D` e digitar o comando (exige permissao de Acessibilidade) |

- Terminal nao reconhecido (Terminal.app, o terminal embutido de um editor...):
  imprime a instrucao manual daquele terminal em vez de tentar adivinhar.
- Sem `relay-tui` no `PATH`: nao instala nada; aponta o guia `docs/TUI.md` e o
  comando de download.
- `--dry-run` imprime o comando que executaria, sem executar.
- Docs: a lista de skills nos comandos de instalacao (`docs/INSTALL.md` e
  `docs/INSTALL.pt-BR.md`), a tabela de skills do README e a nota de que a skill
  esta fora do protocolo (ela nao le nem escreve registros).

## Non-goals

- Instalar ou atualizar o binario; fechar ou gerenciar o ciclo de vida do painel.
- Terminais fora da tabela e Windows.
- Qualquer escrita nos cinco registros.

## Decisions

**A skill e do pacote, nao do protocolo.** Ela nao interpreta nem muda registros
(`docs/PROTOCOL.md`); so abre uma janela. Por isso vive em `skills/` (vai nos
manifestos e nas instalacoes), mas e documentada como fora do protocolo.

**Deteccao por variavel de ambiente.** A skill roda no shell do harness, que e
filho do terminal e herda `TMUX`, `TERM_PROGRAM` e afins. Que o Claude Code, o
Codex e o OpenCode realmente repassam isso e uma hipotese a verificar, nao um
fato: e o que o A-004 comprova.

**O Warp e o caso mais fragil.** Ele nao tem comando de split nem AppleScript
proprio; so resta simular `Cmd+D` pelo System Events, que exige permissao de
Acessibilidade e quebra se o atalho mudar. Fica na tabela porque e o terminal do
dono do projeto, com a instrucao manual como saida quando a simulacao falha.

**O script e testavel sem terminal.** Recebe o ambiente por variaveis e tem
`--dry-run`; os testes montam o ambiente de cada terminal e comparam o comando.

## Acceptance criteria

- A-001 - Para cada terminal da tabela, o script, com as variaveis de ambiente
  daquele terminal, imprime em `--dry-run` o comando de split correto, verificado
  por testes.
- A-002 - Sem terminal reconhecido o script imprime a instrucao manual; sem
  `relay-tui` no `PATH` aponta o download; nunca instala nada.
- A-003 - A skill `relay-tui` existe em `skills/` com menos de 40 linhas e consta
  nas listas de instalacao (EN e PT-BR) e na tabela de skills do README.
- A-004 - No tmux, no iTerm2 e no Warp (este na maquina do dono), a skill abre um
  painel a direita rodando o `relay-tui` no repositorio atual, com a evidencia
  registrada, incluindo que o ambiente do terminal chega ao shell do harness.
- A-005 - A skill nao escreve nenhum dos cinco registros, verificado por teste ou
  inspecao do script.

## Backlog candidates

- B-001: Script `open-split.sh` com deteccao do terminal e `--dry-run`, testado por terminal.
- B-002: Skill `relay-tui`, manifestos e documentacao de instalacao.
- B-003: Verificacao nos terminais reais (tmux, iTerm2, Warp) com evidencia.
