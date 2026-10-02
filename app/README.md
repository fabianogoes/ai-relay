# Rodando o Relay

Este diretório é a interface do Relay. Leia `AGENTS.md` antes de mudar
qualquer coisa aqui — ele traz as regras de fronteira (ADR-0004) e o contrato
de dados (ADR-0003).

## Pré-requisito

Node ≥ 24 (`node --version`). Nada além disso.

## Instalar

```sh
cd app
npm install
```

Isso instala as dependências do workspace inteiro (`relay-core`, `relay-host`
e `relay-ui`). Os binários (`vite`, `vue-tsc`) ficam em
`app/node_modules/.bin/` — não existem soltos no PATH nem dentro de
`app/relay-ui/`. Sempre rode pelos scripts do `npm`, nunca pelo binário cru.

## Abrir qualquer repositório Relay em read-only

O produto padrão é o observador: o host lê os cinco registros do workspace e a
UI acompanha o estado sem nunca escrever neles.

```sh
cd app
npm run build --workspace relay-ui
npm start --workspace relay-host -- --workspace=/caminho/do/repo
```

O `relay-host` liga só em `127.0.0.1`, numa porta efêmera, e imprime a URL
(`http://127.0.0.1:<porta>`) com o workspace observado. Sem `--workspace`, ele
observa o diretório corrente. Opções:

- `--workspace=<path>` — repositório Relay a observar (relativo é resolvido
  contra o cwd).
- `--exec` — reabre a superfície experimental de execução integrada
  (lançamento de harness, preflight e terminal). **Ausente por padrão.**
- `--no-exec` — alias de compatibilidade; nunca habilita execução.
- `--port=<n>` — porta fixa em vez da efêmera.

Em read-only nenhuma superfície de seleção, preflight, terminal ou consulta de
harness é montada; a proveniência do handoff continua visível. O WebSocket
entrega snapshots estáveis após 150 ms de quiescência e marca transição e
desconexão com texto acessível.

## As duas visões

- **Agora** — projeção operacional: handoff e TODO do trabalho em curso.
- **Trabalho** — cascata de cartões spec → backlog → changelog; selecionar uma
  spec mostra seus cartões de backlog, e selecionar um cartão de backlog filtra
  o changelog daquele `Backlog:`. O texto integral da spec não é reproduzido.

## Modo de execução

Com `--exec`, escolher "Retomar" ou "Começar" abre o `PreflightModal`: ele
mostra o `argv` elemento por elemento (nunca uma string montada), com o
seletor de harness e consentimento embutido. Confirmar fecha o modal e entra
no **modo de execução** — a viewport inteira vira duas colunas:

- **Terminal** (esquerda) — `xterm.js` sobre PTY via WebSocket, com
  alt-screen, mouse tracking e bracketed paste. Fechar o painel **desanexa**
  o processo (não mata); reabrir **reanexa** com replay do scrollback. Na
  primeira execução, um aviso explica o conflito de atalhos do teclado
  (`Cmd+W`, `Cmd+T`) e oferece o modo externo (lançado por script wrapper).
- **Gravado em disco** (direita) — uma entrada por escrita nos cinco
  registros do protocolo, mais recente primeiro, com antes/depois do trecho
  cru. É a prova visível de que quem escreve é a skill no harness, nunca a
  UI (ADR-0001, ponto 5).

Uma execução desanexada continua visível como uma faixa abaixo do Header
("Rodando em segundo plano"), com um botão para reconectar ao terminal.
"Encerrar processo" é a única ação de perigo do app e exige confirmação.

## relay-tui, o painel de terminal

Existe também um painel de terminal só de leitura, em Rust, que se instala por
um único download: ver [`relay-tui/README.md`](relay-tui/README.md). Ele
compartilha com o `relay-core` a suíte de conformidade de [`conformance/`](conformance/README.md).

## Desenvolvimento da UI

```sh
cd app/relay-ui
npm run dev
```

Abre em `http://localhost:5173`. A barra no topo troca entre os sete estados
do protocolo (fixtures de `app/fixtures/`) — ferramenta de desenvolvimento para
ver cada estado sem um `relay-host` rodando.

## Verificar tipos, testes e build

```sh
npm run typecheck --workspace relay-core
npm test --workspace relay-core

npm run typecheck --workspace relay-host
npm test --workspace relay-host

npm run typecheck --workspace relay-ui
npm test --workspace relay-ui
npm run build --workspace relay-ui
```

## Estrutura

```text
app/
  package.json       raiz do workspace (nunca publicada, "private": true)
  AGENTS.md           regras desta pasta
  fixtures/           os sete estados normativos do protocolo (ADR-0003)
  relay-core/         deriva o estado a partir dos registros (puro, sem I/O)
  relay-host/         lê o workspace, serve a UI e empurra snapshots por WS
  relay-ui/           a interface Vue 3 + Vite + TS
    src/
      types.ts        o contrato (ADR-0003): RelayState, UiPayload
      components/     StatusPill, HandoffCard, ChecklistList, RepairScreen,
                      PreflightModal, Terminal, DiskLog, ExecutionMode...
      styles/
        tokens.css      derivado de docs/design-system/README.md — nunca editar
                        sem mudar o README primeiro
        app.css         classes por componente, consumindo só var(--token)
```
