# Relay

Relay da memoria operacional a agentes de codigo. O estado do trabalho fica em
arquivos Markdown no repositorio, e nao no historico do chat, para que qualquer
agente (Claude Code, Codex, OpenCode) retome de onde outro parou.

## Por que existe

Voce comeca uma tarefa no Codex, para no meio (porque quis ou porque bateu no
limite de uso) e quer continuar no Claude Code. O agente novo nao sabe o que
foi decidido nem onde voce parou, e quem reconstroi esse contexto e voce.

Relay guarda esse contexto no repositorio:

- o que fazer e por que (a spec);
- o que falta (backlog e subtarefas);
- o que esta em andamento agora, quem deixou e quando (o handoff);
- o que foi concluido, com evidencia (o changelog).

## Como usar

| Skill | Quando usar |
| --- | --- |
| `relay-setup` | Uma vez, para instalar o protocolo no repositorio. |
| `relay-spec` | Quando voce tem uma ideia: uma entrevista a transforma em spec e tarefas. |
| `relay-session` | Ao abrir uma sessao: retoma ou comeca o trabalho. |
| `relay-continue` | Quando voce quer saber qual e o proximo passo. |
| `relay-status` | Para ver o estado sem alterar nada. |

## Documentos

- Para saber mais sobre **Instalacao** veja [docs/INSTALL.md](docs/INSTALL.md).
- Para saber mais sobre **Arquivos, estados e regras** veja [docs/PROTOCOL.md](docs/PROTOCOL.md).

## Perguntas frequentes

**Spec, backlog, TODO, handoff e changelog nao e burocracia demais?**
Seria, se voce escrevesse esses arquivos. Quem escreve sao as skills. Voce
responde a uma entrevista quando tem uma ideia nova e escolhe entre opcoes
quando abre uma sessao. O resto e registro que o agente faz enquanto trabalha.

**Isso nao aumenta minha carga cognitiva?**
A carga ja existe: sem Relay, e voce quem guarda o contexto entre sessoes e
agentes. Relay tira esse estado da sua cabeca e o coloca em arquivos que
qualquer agente le.

**Preciso usar Relay para toda tarefa?**
Nao. Para corrigir um typo ou um bug de cinco minutos, peca direto ao agente,
sem invocar nenhuma skill. Relay compensa em trabalho que atravessa sessoes,
agentes ou interrupcoes.

**Qual o custo de usar?**
A entrevista da spec toma tempo antes do primeiro codigo, e toda sessao comeca
lendo os registros, o que consome tokens. Em troca, a retomada nao depende da
memoria de ninguem.

**E se eu parar no meio de uma tarefa?**
Na proxima sessao, em qualquer harness, `relay-session` le o handoff, encontra
a subtarefa em andamento e pergunta se retoma dali.

**E se os registros estiverem errados?**
O agente para, explica o problema e pergunta antes de agir. Ele nunca segue
trabalhando em cima de um estado incoerente.

**Preciso de um harness especifico, uma UI ou um CLI?**
Nao. Relay e Markdown no repositorio mais skills para Claude Code, Codex e
OpenCode. Interfaces podem exibir o estado, mas nunca escrevem nos registros.
Para acompanhar o estado num split de terminal existe o `relay-tui`, um binario
so de leitura baixado em https://github.com/fabianogoes/relay/releases (macOS;
Linux em melhor esforco; Windows fora por enquanto).

**Relay substitui meu gerenciador de projetos?**
Nao. Ele guarda o minimo para agentes trabalharem com continuidade, e nao
roadmap, prioridade ou estimativa.

## Licenca

[MIT](LICENSE).
