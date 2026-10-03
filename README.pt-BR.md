# Relay

[![relay-tui CI](https://github.com/fabianogoes/ai-relay/actions/workflows/relay-tui-ci.yml/badge.svg?branch=main)](https://github.com/fabianogoes/ai-relay/actions/workflows/relay-tui-ci.yml)

**[English](README.md)** · Português

Relay dá memória operacional a agentes de código. O estado do trabalho fica em
arquivos Markdown no repositório, e não no histórico do chat, para que qualquer
agente (Claude Code, Codex, OpenCode) retome de onde outro parou.

## Por que existe

Você começa uma tarefa no Codex, para no meio (porque quis ou porque bateu no
limite de uso) e quer continuar no Claude Code. O agente novo não sabe o que
foi decidido nem onde você parou, e quem reconstrói esse contexto é você.

Relay guarda esse contexto no repositório:

- o que fazer e por quê (a spec);
- o que falta (backlog e subtarefas);
- o que está em andamento agora, quem deixou e quando (o handoff);
- o que foi concluído, com evidência (o changelog).

## Por que o nome

Numa corrida de revezamento, quem vence é o bastão, não um corredor: cada um
corre o seu trecho e o entrega sem deixá-lo cair. Relay é esse bastão para
agentes de código. A passagem de um agente a outro é o momento que importa, e o
repositório carrega o bastão, para que o próximo agente, qualquer que seja, o
pegue já em velocidade.

## Como usar

| Skill | Quando usar |
| --- | --- |
| `relay-setup` | Uma vez, para instalar o protocolo no repositório. |
| `relay-spec` | Quando você tem uma ideia: uma entrevista a transforma em spec e tarefas. |
| `relay-session` | Ao abrir uma sessão: retoma ou começa o trabalho. |
| `relay-continue` | Quando você quer saber qual é o próximo passo. |
| `relay-status` | Para ver o estado sem alterar nada. |

## Painel de terminal

Relay tem um painel só de leitura para o terminal, o `relay-tui`: deixe-o num
split ao lado do harness e veja handoff, TODO e backlog mudarem enquanto o
agente trabalha. É um único binário, sem Node.

## Documentos

| Assunto | Português | English |
| --- | --- | --- |
| **Instalação** | [docs/INSTALL.pt-BR.md](docs/INSTALL.pt-BR.md) | [docs/INSTALL.md](docs/INSTALL.md) |
| **Arquivos, estados e regras** | [docs/PROTOCOL.pt-BR.md](docs/PROTOCOL.pt-BR.md) | [docs/PROTOCOL.md](docs/PROTOCOL.md) |
| **Painel de terminal** | [docs/TUI.pt-BR.md](docs/TUI.pt-BR.md) | [docs/TUI.md](docs/TUI.md) |

## Perguntas frequentes

**Spec, backlog, TODO, handoff e changelog não é burocracia demais?**
Seria, se você escrevesse esses arquivos. Quem escreve são as skills. Você
responde a uma entrevista quando tem uma ideia nova e escolhe entre opções
quando abre uma sessão. O resto é registro que o agente faz enquanto trabalha.

**Isso não aumenta minha carga cognitiva?**
A carga já existe: sem Relay, é você quem guarda o contexto entre sessões e
agentes. Relay tira esse estado da sua cabeça e o coloca em arquivos que
qualquer agente lê.

**Preciso usar Relay para toda tarefa?**
Não. Para corrigir um typo ou um bug de cinco minutos, peça direto ao agente,
sem invocar nenhuma skill. Relay compensa em trabalho que atravessa sessões,
agentes ou interrupções.

**Qual o custo de usar?**
A entrevista da spec toma tempo antes do primeiro código, e toda sessão começa
lendo os registros, o que consome tokens. Em troca, a retomada não depende da
memória de ninguém.

**E se eu parar no meio de uma tarefa?**
Na próxima sessão, em qualquer harness, `relay-session` lê o handoff, encontra
a subtarefa em andamento e pergunta se retoma dali.

**E se os registros estiverem errados?**
O agente para, explica o problema e pergunta antes de agir. Ele nunca segue
trabalhando em cima de um estado incoerente.

**Preciso de um harness específico, uma UI ou um CLI?**
Não. Relay é Markdown no repositório mais skills para Claude Code, Codex e
OpenCode. Interfaces podem exibir o estado, mas nunca escrevem nos registros; o
`relay-tui` é uma delas (veja [docs/TUI.pt-BR.md](docs/TUI.pt-BR.md)).

**Relay substitui meu gerenciador de projetos?**
Não. Ele guarda o mínimo para agentes trabalharem com continuidade, e não
roadmap, prioridade ou estimativa.

## Licença

[MIT](LICENSE).
