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

## Acompanhar pelo terminal: relay-tui

O `relay-tui` e um painel so de leitura para deixar num split do terminal: o
harness de um lado, o painel do outro. Handoff, TODO e backlog mudam na tela
conforme os arquivos mudam, sem recarregar nada. Ele nunca escreve nos
registros: quem escreve sao as skills.

**Instalar (macOS).** Um download, sem Node e sem clonar o repositorio. Em Mac
Apple Silicon use `aarch64-apple-darwin`; em Mac Intel, `x86_64-apple-darwin`
(`uname -m` diz qual e o seu). Baixando com `curl`, o macOS nao bloqueia o
arquivo:

```sh
curl -fLO https://github.com/fabianogoes/ai-relay/releases/download/relay-tui-v<versao>/relay-tui-<versao>-aarch64-apple-darwin.tar.gz
tar -xzf relay-tui-<versao>-aarch64-apple-darwin.tar.gz
mkdir -p ~/.local/bin
mv relay-tui-<versao>-aarch64-apple-darwin/relay-tui ~/.local/bin/
```

Garanta que `~/.local/bin` esta no seu `PATH`. Se baixou pelo navegador, o macOS
diz que o `relay-tui` nao pode ser aberto (o binario nao e assinado); libere com
`xattr -d com.apple.quarantine ~/.local/bin/relay-tui`. As versoes estao em
https://github.com/fabianogoes/ai-relay/releases. Linux e melhor esforco, e o
Windows ficou de fora por enquanto.

**Usar.** Abra um split do terminal (no iTerm, `Cmd+D`; no tmux,
`tmux split-window -h`), entre no repositorio Relay e rode:

```sh
relay-tui                          # observa o diretorio atual
relay-tui --workspace /caminho/do/repo
```

Deixe o harness no outro painel e trabalhe como sempre; o painel acompanha.
`q`, `Esc` ou `Ctrl-C` saem. O que a tela mostra:

- **Handoff**: o status (`Em andamento`, `Bloqueado`), quem deixou e quando, o
  objetivo e o proximo passo. Se estiver bloqueado, mostra o bloqueio e a
  condicao de retomada.
- **TODO**: uma barra com um segmento por subtarefa e a lista, com `✓` feita,
  `●` em andamento, `○` disponivel, `◌` esperando outra subtarefa e `!`
  bloqueada.
- **Backlog**: quantos itens estao feitos, em curso, disponiveis ou aguardando.
- **`● atualizando` / `● atualizado`**, no canto: o painel espera os arquivos
  pararem de mudar (150 ms) e le tudo de novo.
- **`Inconsistente`**, em vermelho: os registros se contradizem, e o painel
  lista cada violacao. E o mesmo estado que `relay-status` reporta.

Se faltar altura ou largura, o painel mostra menos (o TODO corta em `+N itens`);
numa janela estreita, so o status. Um diretorio sem `.orchestration/` mostra
"Nao e um workspace Relay" e passa a mostrar o estado assim que o Relay for
instalado ali (`relay-setup`).

## Documentos

- Para saber mais sobre **Instalacao** veja [docs/INSTALL.pt-BR.md](docs/INSTALL.pt-BR.md)
  (em ingles: [docs/INSTALL.md](docs/INSTALL.md)).
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
so de leitura baixado em https://github.com/fabianogoes/ai-relay/releases (macOS;
Linux em melhor esforco; Windows fora por enquanto).

**Relay substitui meu gerenciador de projetos?**
Nao. Ele guarda o minimo para agentes trabalharem com continuidade, e nao
roadmap, prioridade ou estimativa.

## Licenca

[MIT](LICENSE).
