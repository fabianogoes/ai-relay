# Change log

## 2026-09-17 - T-003 - Teste ponta a ponta atravessa HTTP, WebSocket, executor, PTY e disco reais
- Backlog: B-024
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: novo `test/e2e.test.ts` (skip sem PTY disponível) inicia uma run
  real via `startExec` (script `/bin/sh -c` determinístico, não um harness
  de verdade — a composição de argv do harness já está coberta em outro
  teste), então usa exclusivamente HTTP e WebSocket reais contra o mesmo
  `start()`: confirma a run endereçável (`GET /api/run/<id>`, A-005), anexa
  o WS de terminal e escreve via input real (mudando `.orchestration/TODO.md`,
  detectável pelo disk tracker), desanexa (fecha o WS sem terminar o
  processo, A-006), confirma via HTTP que a run continua `running` enquanto
  desanexada, reanexa com um WS novo e recebe o diff acumulado durante a
  ausência, termina via `POST /api/run/<id>` `action: terminate` e recebe o
  evento `exit` real pelo WS (não otimista, A-008), confirma `status:
  exited` via HTTP, e por fim fecha explicitamente (`action: close`) e
  confirma `404` depois — a run parou de ser endereçável só após o
  fechamento explícito (A-006/A-007).
- Evidence: `npm test --workspace relay-host` 50/50; `tsc --noEmit` limpo.
  Construir o teste revelou e corrigiu dois problemas reais no próprio
  teste antes de estabilizar: um script de shell interativo genérico (sem
  `-c`) se mostrou não-determinístico entre plataformas (saiu sozinho com
  código 127 em vez de ficar esperando input), substituído por um script
  `-c` explícito com dois `read`; e a ausência de `run.handle.terminate()`
  no `finally` deixava um processo órfão esperando por `read line2`
  travando o processo Node indefinidamente sempre que uma asserção
  anterior falhava — corrigido garantindo terminação incondicional.
  Confirmada a sensibilidade do teste a regressões reais: comentar
  temporariamente `executorEvents.runStarted = (run) =>
  wireRunLifecycle(run, server)` em `index.ts` (simulando perder a fiação
  do ciclo de vida) fez o teste falhar corretamente esperando por uma
  mensagem `data` que nunca chegou — sem travar, graças ao `terminate()`
  garantido no `finally`.
- Criteria: A-012
- Decisions: o teste usa `startExec` diretamente em vez de
  `POST /api/launch/embedded` para não depender de um harness real (claude/
  codex/opencode) instalado na máquina que roda os testes — que além de
  frágil, invocaria de verdade o CLI de produção; a cobertura de argv do
  harness já pertence a outro teste. Testes de integração que envolvem um
  processo real (PTY) sempre garantem `terminate()` incondicional no
  `finally`, nunca dependem só do caminho feliz para encerrar o processo.

## 2026-09-17 - T-002 - Escolha embutido/externo ocorre antes do spawn; externo nunca deixa embutido vivo
- Backlog: B-024
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `PreflightModal.vue`'s `confirm()` deixou de lançar embutido
  imediatamente na primeira execução da sessão — em vez disso, mostra
  (dentro do próprio preflight, um novo `step === 'keyboard-choice'`) a
  escolha embutido/externo com o aviso de conflito de teclado, e só chama
  `launchEmbedded()` OU `POST /api/launch` depois da escolha, nunca as duas
  em sequência. Runs seguintes na mesma sessão continuam lançando embutido
  direto (com fallback automático a externo só se o embutido falhar sem
  chegar a spawnar nada, comportamento preexistente e seguro). Repaginado
  `KeyboardWarning.vue`: agora só aparece para uma run já viva que esta aba
  não lançou (reanexo automático via `discoverActiveRun`, A-009) — perdeu o
  botão "Lançar no modo externo" e seu `onExternal()` (que chamava
  `closeRun()` sem nunca terminar o processo, abandonando-o vivo e
  invisível — exatamente o bug descrito no Problem da spec), ficando só com
  um reconhecimento ("Entendi"). `docs/design-system/README.md` atualizado
  primeiro, documentando a nova ordem e o caso do reanexo sem opção de modo.
- Evidence: `npm test --workspace relay-ui` 14/14 (node --test) + 27/27
  (vitest, 3 testes novos: escolha aparece antes de qualquer lançamento,
  "externo" chama só `/api/launch`, "embutido" chama só
  `/api/launch/embedded`); `npm test --workspace relay-host` 49/49
  (inalterado); `vue-tsc --noEmit` limpo. Confirmado por reversão: sem o
  gate de `firstRunThisSession`, os três testes voltam a falhar porque
  `launchEmbedded` é chamado imediatamente ao confirmar, antes de qualquer
  escolha.
- Criteria: A-004
- Decisions: a escolha vive dentro do próprio `PreflightModal` (não num
  segundo modal sobreposto) para manter erro de lançamento visível no mesmo
  fluxo; `KeyboardWarning` deixa de oferecer "modo externo" porque não há
  como migrar uma run embutida já viva para externo — a ação honesta pra
  isso já existe (Encerrar processo, em `ExecutionMode`) e não precisa de
  duplicata.

## 2026-09-17 - T-001 - Sob --no-exec, rotas de run e o WS de terminal deixam de existir
- Backlog: B-024
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `server.ts` passa a checar `execEnabled` também em `GET /api/runs`,
  `GET /api/run/<id>` e `GET /api/run/<id>/disk` (404 antes de qualquer outra
  checagem, mesmo padrão de `/api/launch*` e `POST /api/run/<id>`), e no
  handshake de upgrade do `/ws/term`: origem/token são checados primeiro
  (403 se inválidos, como já era), e só depois, sob `--no-exec`, o socket
  recebe `404 Not Found` e é destruído sem nunca completar o handshake.
  Corrigido também o rótulo do teste pré-existente de `/api/launch` sob
  `--no-exec`, que citava A-004 mas descrevia A-011.
- Evidence: `npm test --workspace relay-host` 49/49 (2 testes novos: rotas
  GET de run sob `--no-exec` e handshake de `/ws/term` sob `--no-exec`);
  `tsc --noEmit` limpo. Cada fix confirmado por reversão: sem a checagem nas
  rotas GET, `/api/runs` respondia 200 sob `--no-exec`; sem a checagem no
  upgrade do WS, o handshake completava e abria a conexão — e essa reversão
  revelou um teste próprio malformado (sem fechar a conexão aberta no
  handler `open`), que travou o processo; corrigido fechando a conexão em
  qualquer desfecho antes de restaurar o fix.
- Criteria: A-011
- Decisions: a ordem de checagem no upgrade do WS segue a mesma leitura da
  ADR-0006 usada nas rotas HTTP — autenticação (403) antes de ausência de
  rota (404) — para que uma requisição autenticada sob `--no-exec` prove
  404 por ausência estrutural da rota, não por coincidência de dado vazio.

## 2026-09-16 - T-003 - Teste de integração prova a descoberta pós-restart ponta a ponta
- Backlog: B-023
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: novo teste em `test/start.test.ts` ("matar e reiniciar o host muda
  a porta principal; o farol (mesma workspace, mesma porta) sempre aponta
  pra atual") inicia `start()` para uma workspace, confirma que o farol
  devolve a porta principal correta, fecha esse processo ("mata o host"),
  confirma que a porta antiga fica inalcançável, inicia `start()` de novo
  para a MESMA workspace ("reinicia"), e confirma que o mesmo farol (mesma
  porta, nunca recalculada pelo cliente) passa a devolver a porta nova, e
  que o bootstrap (`GET /`) responde normalmente nela.
- Evidence: `npm test --workspace relay-host` 47/47. Confirmado por
  reversão: hardcodear a resposta do farol para ignorar `currentPort`
  quebrou a asserção — e revelou que o teste, sem `try/finally` cobrindo
  ambos os servidores, travava o processo indefinidamente ao falhar no meio
  (servidor e farol nunca fechados, handle TCP aberto mantém o event loop
  vivo); corrigido guardando as duas referências fora do `try` e fechando
  ambas incondicionalmente no `finally`, e o fix de `discovery.ts` foi
  restaurado.
- Criteria: A-010, A-012
- Decisions: nenhuma nova; consolida a prova ponta a ponta do mecanismo
  registrado na ADR-0006 decisão 8 e implementado no T-002.

## 2026-09-16 - T-002 - Farol de descoberta implementado no host e no cliente
- Backlog: B-023
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: novo módulo `relay-host/src/discovery.ts` com `discoveryPort(workspace)`
  (hash SHA-256 do caminho absoluto, mapeado numa faixa fixa acima de 1024) e
  `startDiscoveryBeacon(workspace, currentPort)` (listener somente-leitura em
  `127.0.0.1`, devolve `{ port }`, CORS restrito a origens `127.0.0.1:*`).
  `server.ts` embute a porta do farol no HTML inicial via novo
  `<meta name="relay-discovery-port">`, ao lado do token. `index.ts`'s
  `start()` computa a porta do farol a partir do workspace, abre o farol
  depois do servidor principal e falha (fechando o servidor principal) se o
  bind do farol colidir. Em `relay-ui`, `relay-client.ts` lê a nova meta;
  depois de `DISCOVERY_AFTER_RETRIES` tentativas seguidas de reconexão na
  mesma origem, consulta o farol e, encontrando uma porta, navega para lá
  via `window.location.href`, disparando o bootstrap normal.
- Evidence: `npm test --workspace relay-host` 47/47 (inclui `discovery.test.ts`
  novo: determinismo de `discoveryPort`, farol devolve porta+CORS correto,
  falha explícita em colisão) e `npm test --workspace relay-ui` 14/14
  (node --test) + 24/24 (vitest, inclui `relay-client-discovery.vitest.ts`
  novo para `shouldAttemptDiscovery`/`parseDiscoveryPort`); `tsc --noEmit` e
  `vue-tsc --noEmit` limpos. Cada peça confirmada por reversão: sem o hash
  correto, `discoveryPort` deixa de ser determinística; sem CORS restrito, a
  origem `https://evil.example` recebia o cabeçalho liberado; sem o
  `server.once('error', reject)`, uma colisão de porta trava o teste
  indefinidamente em vez de rejeitar (achado equivalente ao hang de WS do
  T-004); sem o limiar de tentativas, a descoberta dispararia a cada
  reconexão; sem a validação estrita de `parseDiscoveryPort`, um `port`
  string passava como válido.
- Criteria: none
- Decisions: o farol só é consultado depois de falhas persistentes na mesma
  origem (`DISCOVERY_AFTER_RETRIES`), nunca na primeira desconexão — a
  maioria das quedas de WebSocket é transitória, não um restart do host.

## 2026-09-16 - T-001 - ADR-0006 revisada com o farol de descoberta pós-restart
- Backlog: B-023
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: ADR-0006 ganha a decisão 8 — um segundo listener em `127.0.0.1`,
  somente leitura e sem token, numa porta derivada deterministicamente do
  caminho absoluto do workspace, que devolve só `{ port }` (a porta efêmera
  atual do host para aquele workspace). A porta do farol viaja ao cliente no
  HTML inicial (`<meta name="relay-discovery-port">`), do mesmo jeito que o
  token; o cliente nunca recalcula o hash. Consequências, Conformidade e a
  nota "o que esta ADR não decide" foram atualizadas; nenhuma decisão
  anterior foi revogada (status permanece Accepted).
- Evidence: revisão de design confirmada com o usuário via pergunta direta
  entre duas alternativas (farol por workspace vs farol único global);
  farol por workspace escolhido para não concentrar a descoberta de todos
  os workspaces ativos da máquina num só ponto. Documento revisado em
  `docs/adr/0006-contrato-http-ws-do-relay-host.md`.
- Criteria: none
- Decisions: farol de descoberta é por workspace (porta derivada por hash),
  não um farol único fixo; não exige token (não protege segredo, e exigi-lo
  seria inútil já que só é acionado quando o token anterior já morreu); erro
  de colisão de hash falha o start do host de forma explícita, nunca troca a
  porta de farol em silêncio.

## 2026-09-16 - T-005 - Reabrir a aba durante uma run viva reanexa automaticamente
- Backlog: B-022
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `EmbeddedRun` (executor.ts) passa a carregar `harnessId`/`harnessName`,
  resolvidos em `index.ts` via `resolveHarnessName` (nova funcao pura,
  testada isoladamente) a partir de `detectHarnesses()` — o `LaunchRequest`
  que atravessa a rede só leva o id, nunca o nome. `RunInfo` (pty.ts) e as
  respostas de `runInfo()`/`runs()` (index.ts) passam a incluir
  `harnessId`, `harnessName` e `processName`. Em `relay-ui`, nova
  `discoverActiveRun()` consulta `GET /api/runs` e, havendo uma run
  `running` e nenhuma run ativa localmente, adota seu estado
  (harness/processo/run id) e conecta o socket de termino (que já reenvia
  scrollback e diffs acumulados desde o T-004). `App.vue` chama isso uma
  única vez por sessão quando `execEnabled` fica `true` em modo host.
- Evidence: `npm test --workspace relay-host` 40/40 e `npm test --workspace
  relay-ui` 14/14 (node --test) + 20/20 (vitest); `tsc --noEmit` e
  `vue-tsc --noEmit` limpos nos dois workspaces. Cada correcao confirmada
  por reversao: sem `harnessId`/`harnessName` em `startExec`, o typecheck
  falha (assinatura exige os dois argumentos) e a asserção do executor
  falha; sem a resolucao correta em `resolveHarnessName`, o teste unitario
  falha comparando `'claude-code'` a `'Claude Code'`; sem o guard de
  `store.activeRunId` em `discoverActiveRun`, o teste "já existindo uma run
  ativa localmente, não consulta o host de novo" falha (segunda chamada
  tenta usar uma resposta mockada já consumida).
- Criteria: A-009
- Decisions: o nome do harness é resolvido no host (nunca repassado pelo
  cliente), porque só o id atravessa `POST /api/launch/embedded` hoje; a
  descoberta trata apenas a primeira run `running` retornada por
  `/api/runs`, coerente com o modelo de uma única run ativa por aba que já
  existe em `execution.ts` (`activeRunId` escalar, um único socket); a
  tentativa de descoberta ocorre uma única vez por sessão da aba, não a
  cada atualização de payload.

## 2026-09-16 - T-004 - Run continua enderecavel apos sair; diff final garantido; reanexar reenvia diffs acumulados
- Backlog: B-022
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `executor.ts` para de remover a run do registro no exit (so
  `closeRun(runId)` remove de verdade; `listActiveRuns` filtra por
  `status()==='running'`). `index.ts` extrai `wireRunLifecycle(run, server)`,
  exportada e testavel, que no exit calcula `run.disk.diff()` e manda por
  `termDisk` ANTES de `termExit` — a saida nunca "perde" a ultima escrita.
  `server.ts` ganha `deps.runClose` e a action `close` na rota
  `POST /api/run/<id>` (404 se a run nao existe ou ja foi fechada); o handler
  de attach do WS de terminal reenvia `deps.runDiskEntries(runId)` alem do
  scrollback. `relay-ui`'s `closeRun()` chama a rota de close antes de
  resetar o estado local; `wire()`/`onDisk` dedupe por `id` pra reanexar nao
  duplicar entrada ja conhecida.
- Evidence: `npm test --workspace relay-host` 39/39 e `npm test --workspace
  relay-ui` 14/14 (node --test) + 17/17 (vitest); `tsc --noEmit` e
  `vue-tsc --noEmit` limpos nos dois workspaces. Cada uma das cinco correcoes
  foi confirmada por reversao: sem a retencao no registro, o teste do
  executor falha na asserção "run deveria continuar endereçável após sair";
  sem a ordem disk-antes-de-exit, `wireRunLifecycle` produz `['exit','disk']`
  em vez de `['disk','exit']`; sem a action `close`, a rota devolve 200 em
  vez de 400; sem o reenvio de disk no attach, o teste de WS **trava
  indefinidamente** (motivou um timeout de 2s em `waitForTermMessage`); sem o
  dedupe no cliente, `useDisk().length` reporta 3 em vez de 2 apos reanexar.
- Criteria: A-006, A-007
- Decisions: run permanece no registro apos sair ate fechamento explicito
  (`closeRun`), nunca desaparece sozinha; o diff final e computado e
  publicado antes do evento de saida ser anunciado, nunca depois; reanexar
  reenvia o disk log acumulado inteiro (nao so o delta), com dedupe por id no
  cliente absorvendo a redundancia.

## 2026-09-11 - T-002 - Datas e status localizados na camada de apresentacao
- Backlog: B-027
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `presentation.ts` centraliza os sete rotulos humanos de status em
  portugues, timestamps absolutos e relativos com `Intl` e datas civis do
  changelog sem deslocamento de fuso. `StatusPill`, HandoffCard, DiskLog e
  WorkScreen consomem essa camada; nenhum campo foi acrescentado ao core.
- Evidence: ciclo RED/GREEN em `presentation.test.ts`; `npm test --workspace
  relay-ui` 14/14; `vue-tsc --noEmit` e `vite build` limpos; verificacao
  independente confirmou a integracao somente na camada de apresentacao e
  `git diff --check` ficou limpo.
- Criteria: A-009
- Decisions: locale fixo `pt-BR`; datas civis do changelog usam UTC apenas para
  preservar o dia do registro, enquanto timestamps mantem o fuso local do
  navegador.

## 2026-09-11 - T-001 - Cores da UI convergem para tokens do design system
- Backlog: B-027
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `--scrim` documentado e aplicado em `tokens.css`, no HTML derivado e
  no overlay; o tema do xterm passou a ler `--bg-deep`, `--ink` e `--green`
  via `getComputedStyle`, sem cores literais no componente. A conversao ficou
  isolada em `terminal-theme.ts` e coberta por teste.
- Evidence: `npm test --workspace relay-ui` 10/10; `vue-tsc --noEmit` e `vite
  build` limpos; varredura Python nao encontrou cores literais fora de
  `tokens.css`; `git diff --check` limpo; HTML derivado com duas adicoes, sem
  leitura como fonte conforme a governanca do design system.
- Criteria: A-010
- Decisions: `--scrim` permanece distinto de `--bg-deep`; o terminal resolve
  as custom properties no elemento hospedeiro antes de construir o xterm.

## 2026-09-11 - T-004 - Foco visivel e nomes/estados acessiveis; suite verde
- Backlog: B-026
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: foco visivel global confirmado (`:focus-visible` em `app.css`, de
  B-016); nomes/estados acessiveis: botoes de fechar com `aria-label`, abas com
  nome e estado, radios associados a `label`, `FreshnessStatus` com
  `role="status"` + `aria-live`. Suite da UI verde.
- Evidence: `npm test --workspace relay-ui` 9/9; `vue-tsc --noEmit` e `vite
  build` limpos.
- Criteria: A-007
- Decisions: nenhuma decisao nova.

## 2026-09-11 - T-003 - main, h1 e abas com estado acessivel
- Backlog: B-026
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `.app__body` virou `<main>`; o logo do header virou `<h1>` (com
  `margin: 0` em `.header__logo`); as abas Agora/Trabalho viraram
  `role="tablist"`/`role="tab"` com `aria-selected` e `aria-controls`, e as
  visoes `role="tabpanel"` com `aria-labelledby`. Hierarquia h1 > h2 (visoes) >
  h3 (secoes) preservada.
- Evidence: `vue-tsc --noEmit` e `vite build` limpos; leitura de `App.vue` e
  `Header.vue`.
- Criteria: A-007
- Decisions: o logo como h1 e a marca da pagina; `margin: 0` apenas para
  acomodar o elemento de heading, sem token novo.

## 2026-09-11 - T-002 - Escape e clique-fora por superficie
- Backlog: B-026
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: Escape fecha PreflightModal e HarnessSelector — superficies em que
  fechar nao dispara acao; KeyboardWarning nao fecha por Escape (as duas
  escolhas sao acao); clique-fora permanece so no seletor standalone, como o
  design autoriza.
- Evidence: leitura dos tres componentes; `vue-tsc --noEmit` e `vite build`
  limpos.
- Criteria: A-006
- Decisions: nenhuma superficie fecha disparando acao.

## 2026-09-11 - T-001 - mountDialog: foco inicial, fundo inerte e restauracao
- Backlog: B-026
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `lib/focus-trap.ts` ganhou `mountDialog(root, initialFocus?)` que
  salva o originador, move o foco para o alvo (ou o primeiro focavel), aplica
  `inert` aos irmaos do overlay, prende Tab/Shift+Tab e, no `release`, desfaz o
  `inert` e restaura o foco. Aplicado a PreflightModal (foco no prompt),
  HarnessSelector (foco no overlay), KeyboardWarning (foco no primeiro controle)
  e BacklogTasksModal.
- Evidence: `vue-tsc --noEmit` e `vite build` limpos; os tres dialogos do A-006
  e o modal de tarefas usam o mesmo mecanismo.
- Criteria: A-006
- Decisions: `inert` sobre os irmaos do overlay (`parent.children`), cobrindo o
  conteudo de tras do `.app`; restauracao ao originador no `release`.

## 2026-09-11 - T-004 - Seletor de harness e consentimento compartilhado
- Backlog: B-025
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: criado `HarnessConsentPicker.vue` com a lista de harnesses (avatar,
  nome, versao, estado, tons) e o radiogroup de consentimento;
  `HarnessSelector.vue` (standalone) e `PreflightModal.vue` passaram a usar o
  mesmo componente, sem duplicar marcacao nem regras. A marcacao
  `harness-option`/`consent-option` e `stateLabel` agora existem so no
  componente compartilhado.
- Evidence: grepe confirma ocorrencia unica da marcacao; `vue-tsc --noEmit` e
  `vite build` limpos; `npm test --workspace relay-ui` 9/9.
- Criteria: A-003
- Decisions: o componente e controlado (recebe a selecao e emite a mudanca); o
  store de harness segue como fonte unica de estado, sem logica duplicada.

## 2026-09-11 - T-003 - Preview atomico por revisao no PreflightModal
- Backlog: B-025
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `PreflightModal.vue` passou a versionar o plano: cada `loadPlan`
  incrementa a revisao e limpa o plano ate a resposta da mesma revisao chegar;
  `canConfirm` exige `activeHarness` + plano da revisao atual + `!launching`,
  entao digitar ou trocar harness e confirmar imediatamente nunca executa o
  argv anterior. Respostas fora de ordem sao descartadas por token. Loading e
  erro sao anunciados por `role="status"` e `role="alert"`.
- Evidence: `vue-tsc --noEmit` e `vite build` limpos; `npm test --workspace
  relay-ui` 9/9. O gate e por revisao (`planForRevision === revision`), nao por
  presenca de plano.
- Criteria: A-001, A-002
- Decisions: a tabela esvazia enquanto a nova revisao compoe (plano antigo nunca
  aparece como atual); o status reusa a classe `.preflight__error` para nao
  introduzir regra de componente nova fora do design system.

## 2026-09-11 - T-002 - Harnesses sem fallback e confirm desabilitado sem harness executavel
- Backlog: B-025
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `setHarnesses()` passou a atribuir a lista recebida sem cair em
  `HARNESS_FIXTURE`; `App.vue` chama `setHarnesses([])` quando a deteccao falha
  em modo host; `activeHarness` em `PreflightModal.vue` retorna null quando nao
  ha nenhum harness com `state !== 'absent'`, desabilitando o confirm. Teste
  cobre lista vazia sem fixture.
- Evidence: `npm test --workspace relay-ui` 9/9; `vue-tsc --noEmit` e `vite
  build` limpos.
- Criteria: A-005
- Decisions: a fixture continua sendo o valor inicial do store (modo fixture),
  nunca um fallback de deteccao.

## 2026-09-11 - T-001 - Consentimento reduzido remove persistencia
- Backlog: B-025
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `persist()` em `lib/harness.ts` agora sempre remove a chave de
  localStorage e a entrada de sessao do workspace antes de repersistir conforme
  o nivel; voltar a "So esta execucao" nao deixa nada gravado e reabrir a pagina
  nao ressuscita a escolha. Teste novo `test/harness.test.ts` cobre local->none
  e session->none.
- Evidence: `npm test --workspace relay-ui` 8/8.
- Criteria: A-004
- Decisions: remocao incondicional antes da gravacao condicional — um unico
  caminho de persistencia, sem estado residual.

## 2026-09-11 - T-004 - Read-only sem controles de execucao e suite verde; spec 014 fechada
- Backlog: B-032
- Spec: .specs/20260910-001-observador-read-only.md
- Result: o modo read-only confirmado sem superficie de execucao: `App.vue`
  monta `HarnessSelector`, `PreflightModal` e `KeyboardWarning` sob
  `execEnabled`, o default do host e read-only (cli.test.ts) e o snapshot em
  read-only carrega `environment.execEnabled: false` (B-032/T-001). Suite
  completa verde. B-032 e a ultima entrada pendente da spec 014; com este
  registro, A-001..A-012 estao todos nomeados e a spec fecha.
- Evidence: `npm test` relay-core 30/30, relay-host 24/24, relay-ui 6/6;
  typechecks limpos nos tres; `vite build` limpo. Leitura de `App.vue` (linhas
  106-113) e grepe de `Criteria` no changelog confirmando A-001..A-012.
- Criteria: A-010
- Decisions: ausencia de controles de execucao e coberta pela combinacao de
  default do host (cli), flag no snapshot (watcher) e gate de montagem na UI
  (App.vue), sem harness de teste de componente Vue — padrao ja registrado em
  B-031/T-003.

## 2026-09-11 - T-003 - Documentacao de uso real atualizada
- Backlog: B-032
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `app/README.md` reescrito para o produto atual: descreve
  `relay-core`, `relay-host`, o WebSocket e o comando para abrir qualquer
  workspace em read-only (`npm start --workspace relay-host --
  --workspace=<path>`), com `--exec` experimental e `--no-exec` de
  compatibilidade; as duas visoes e os comandos de typecheck/test/build.
  `app/TODO-BUILD.md` removido — documento temporario cujo grafo de marcos
  (B-010..B-016) ja estava todo concluido, e o proprio arquivo mandava apaga-lo
  quando nao fosse mais necessario.
- Evidence: leitura de `app/README.md`; `app/TODO-BUILD.md` inexistente; nenhum
  outro arquivo referenciava o removido alem das specs que o exigem.
- Criteria: A-011
- Decisions: remover em vez de reescrever o TODO-BUILD, por ser declaradamente
  temporario e obsoleto; a fonte de estado passam a ser as specs.

## 2026-09-11 - T-002 - Selecao de backlog reconciliada e filtro do changelog
- Backlog: B-032
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `app/relay-ui/test/work.test.ts` ganhou a regressao de selecao de
  cartao de backlog: primeiro card por padrao quando a selecao e nula,
  preservacao do existente, queda para o primeiro quando o selecionado
  desaparece e nulo sem cartoes. `WorkScreen.vue` filtra o changelog por
  `selectedBacklogId` (linhas 81-83) e reconcilia a cada mudanca de backlog
  (linhas 73-79).
- Evidence: `npm test --workspace relay-ui` 6/6; leitura de `WorkScreen.vue`.
- Criteria: A-012
- Decisions: a mesma funcao de reconciliacao (`reconcileSpecId`) serve spec e
  backlog; o teste cobre os dois usos.

## 2026-09-11 - T-001 - Regressoes do watcher: dois diretorios e snapshot inconsistente
- Backlog: B-032
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `app/relay-host/test/watcher.test.ts` ganhou duas regressoes: escrever
  em `.specs/` dispara `refreshing` (segundo diretorio observado, alem de
  `.orchestration/`), e um workspace inconsistente entrega pela WS um snapshot
  `state.kind: inconsistent` com `environment.execEnabled: false` em read-only —
  o dado que leva a UI a tela Reparar.
- Evidence: `npm test --workspace relay-host` 24/24. `MainScreen.vue:55`
  renderiza `RepairScreen` quando `state.kind` e `inconsistent`, sem parser do
  protocolo na UI.
- Criteria: A-009, A-010
- Decisions: o teste de diretorio escreve na spec (nao no TODO) para provar que
  `.specs/` e observado; o snapshot inconsistente e entregue na conexao inicial,
  sem depender de nova escrita.

## 2026-09-11 - T-003 - Cobertura por criterio nomeado confirmada e suite verde; spec 011 fechada
- Backlog: B-019
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: A-008 confirmado por construcao — `checkCriteriaWithoutEvidence`
  (`integrity.ts`) verifica apenas a presenca do ID no campo `Criteria`, nunca o
  significado da prosa de `Evidence`. Suite completa verde apos todas as
  mudancas de B-018/B-019. B-019 e a ultima entrada pendente da spec 011; com
  este registro, A-001..A-008 estao todos nomeados e a spec fecha.
- Evidence: `npm test` relay-core 30/30, relay-host 22/22 (duas execucoes
  estaveis), relay-ui 5/5; testes `criteria-without-evidence` e `criterio
  qualificado de outra spec conta como evidencia` cobrem o comportamento do
  A-008; grepe de `Criteria` no changelog confirma A-001..A-008 da spec 011
  nomeados.
- Criteria: A-008
- Decisions: a suite do relay-host teve uma falha transitoria na primeira
  execucao conjunta e passou 22/22 em execucoes subsequentes; registrado aqui
  como observacao, sem mudanca de codigo.

## 2026-09-11 - T-002 - Correcoes append-only da evidencia refutada pela revisao
- Backlog: B-019
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: quatro afirmacoes de evidencia registradas antes deixaram de ser
  verdadeiras e recebem correcao append-only; nenhum registro historico foi
  editado ou removido:
  1) PTY no macOS (B-015/T-001 dizia `script -q -e -F <fifo>` validado
     empiricamente) — refutada: o smoke nao roda no macOS atual; o binario BSD
     nao aceita a sintaxe GNU e sai com codigo 1. Nova evidencia: `pty.ts` ainda
     usa `script` com a forma GNU. Correcao do comportamento: spec 012/B-020.
  2) Handoff vazio produz LIMPO (B-016/T-004) — refutada: o tracker nao
     reconhece os estados vazios canonicos, incluindo `No active handoff.`, como
     `LIMPO`. Nova evidencia: `disk.ts:55` so emite `cleared` quando
     `after === ''`; o handoff canonico vazio nao gera esse diff. Correcao:
     spec 012/A-007.
  3) O status so vira `exited` pelo callback onExit (B-015/T-006) — refutada:
     `terminateRun` (`execution.ts:141-147`) seta `store.status = 'exited'`
     otimisticamente ao pedir o termino, antes da saida confirmada pelo host.
     Nova evidencia: leitura de `execution.ts` linhas 141-147. Correcao:
     spec 012/A-008.
  4) Nenhuma cor literal fora de tokens.css (B-004/T-005, B-016/T-003) —
     refutada: existem literais em `app.css:701` (`rgba(0,0,0,0.55)`) e no tema
     do xterm em `Terminal.vue:24-26` (`#070a0d`, `#e9edf3`, `#5fe3b3`). Nova
     evidencia: grep por `#hex`/`rgba(` em `app/relay-ui` fora de `tokens.css`.
     Correcao: spec 013/A-010.
- Evidence: leitura de `pty.ts`, `disk.ts`, `execution.ts` e grep de cores em
  `app/relay-ui`; as quatro observacoes estao registradas acima.
- Criteria: A-007
- Decisions: correcao de memoria e append-only (decisao da spec 011); a
  correcao do comportamento em si e non-goal desta spec e pertence as specs 012
  e 013.

## 2026-09-11 - T-001 - Regressoes de integridade e estados terminais
- Backlog: B-019
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: `app/relay-core/test/integrity.test.ts` ganhou tres regressoes: entrada
  de backlog ativa sem spec produz `spec-path-mismatch`; backlog todo `[x]`
  deriva `done`; repositorio vazio deriva `idle`. Com os testes de B-018/T-001
  (ID ausente, divergencia) e as golden files done/idle, o A-006 fica coberto
  nos cinco pontos: ID de backlog ausente, caminho de spec ausente, divergencia
  entre os tres registros, backlog todo concluido e repositorio vazio.
- Evidence: `npm test --workspace relay-core` 30/30.
- Criteria: A-006
- Decisions: estados terminais ganham teste explicito alem da golden file para o
  A-006 nao depender so de fixture.

## 2026-09-11 - T-004 - Suite verde e estado do repositorio converge
- Backlog: B-018
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: suite final do relay-core verde (27/27) e typecheck limpo apos as
  mudancas de T-001/T-002; `deriveState` aplicado ao repositorio atual devolve
  `kind: ok`, `status: in_progress`, `activeBacklogId: B-018` — o mesmo que a
  skill relay-status deriva de um handoff nao vazio.
- Evidence: `npm test --workspace relay-core` 27/27; `npm run typecheck
  --workspace relay-core` limpo; script temporario derivando o estado real do
  workspace via `readWorkspace` + `deriveState`.
- Criteria: A-002, A-005
- Decisions: A-005 fechado pela suite verde que inclui os fixtures regenerados
  `ready`/`blocked`, sob a excecao de mudanca de contrato ja registrada em
  B-017. Nenhum arquivo do protocolo foi escrito pelo script de derivacao
  (rodou fora do workspace).

## 2026-09-11 - T-003 - Skills convergem em done/idle e nas mesmas violacoes
- Backlog: B-018
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: `relay-status` incluiu `done` entre os resultados possiveis e as
  condicoes de `inconsistent` para backlog ativo ausente/divergente e tarefa
  ativa sem entrada confrontavel ou com spec divergente; `relay-session` passou
  a reportar `done` para backlog todo `[x]`, `backlog` para pendencia e `idle`
  so na ausencia dos tres registros. Limites de linha preservados: session 40,
  status 31.
- Evidence: leitura das duas skills canonicas; com isso protocolo
  (B-017/T-001), ADR-0003 (B-017/T-004), core (B-018/T-002) e as duas skills
  concordam sobre a derivacao de `done`/`idle` — A-001 fechado pela cadeia de
  evidencias.
- Criteria: A-001
- Decisions: alinhamento por texto, sem decisao nova; a semantica normativa da
  ADR-0003 (fixtures) permanece a referencia.

## 2026-09-11 - T-002 - deriveStatus conforme ao contrato de done/idle
- Backlog: B-018
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: verificacao de `deriveStatus` (`app/relay-core/src/derive.ts`) contra o
  contrato do T-001/B-017: nenhuma divergencia, nenhum ajuste necessario. O core
  ja deriva `done` de backlog nao vazio todo `[x]` e `idle` da ausencia dos tres
  registros.
- Evidence: leitura de `derive.ts` (linhas 35-52); fixtures `done` e `idle`
  passando em `npm test --workspace relay-core`.
- Criteria: none
- Decisions: sem mudanca de codigo — a spec 011 registra que o core ja estava
  correto em done/idle e que a correcao e das skills (T-003).

## 2026-09-11 - T-001 - Checks de cruzamento completos em relay-core
- Backlog: B-018
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: `app/relay-core/src/integrity.ts` passou a violar `backlog-id-mismatch`
  quando o ID de backlog ativo nao existe no backlog (alem da divergencia
  handoff/TODO), e `spec-path-mismatch` quando a tarefa ativa nao tem entrada de
  backlog confrontavel, nao tem spec ou tem spec divergente do handoff. Nenhum
  check novo: os dois identificadores existentes foram completados.
- Evidence: `npm test --workspace relay-core` 27/27 (incluindo dois testes novos
  para ID de backlog ausente e tarefa sem entrada confrontavel) e `npm run
  typecheck --workspace relay-core` limpo. Os fixtures `ready` e `blocked`
  tiveram seus inputs sinteticos ajustados ao contrato novo (backlog com a
  entrada ativa presente) e seus JSONs regenerados — excecao do A-005 para
  mudanca de contrato ja registrada em B-017.
- Criteria: A-003, A-004
- Decisions: handoff e TODO apontando juntos para um ID inexistente agora
  produzem tambem `spec-path-mismatch` alem de `backlog-id-mismatch`; a co
  ocorrencia e a consequencia da decisao da spec 011 de cobrir a ausencia.

## 2026-09-11 - T-004 - Tabela de checks da ADR-0003 alinhada, identificadores estaveis
- Backlog: B-017
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: a tabela da ADR-0003 manteve os identificadores `backlog-id-mismatch`
  e `spec-path-mismatch` — nenhum renomeio, nenhuma linha nova — e teve as
  descricoes das duas linhas alinhadas ao contrato completado nos T-002/T-003.
  A semantica de `done`/`idle` ja constava na ADR (fixtures) e concorda com o
  contrato do T-001.
- Evidence: leitura da tabela em `docs/adr/0003-contrato-do-estado-derivado.md`
  (linhas 190-191 atualizadas); os identificadores permanecem os mesmos e nao
  ha linha acrescentada ou removida.
- Criteria: none
- Decisions: a descricao de linha nao e contrato — o identificador e; alinhar o
  texto e necessario para a ADR-0003 concordar com o protocolo (A-001, que
  B-018 nomeara ao alinhar as skills e o core).

## 2026-09-11 - T-003 - spec-path-mismatch completo no contrato
- Backlog: B-017
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: a secao Integrity checks de `docs/PROTOCOL.md` passou a rejeitar
  tarefa ativa sem entrada de backlog confrontavel, sem spec ou com spec
  diferente do handoff. O identificador `spec-path-mismatch` foi mantido:
  nenhum check novo.
- Evidence: item "The active task has no confrontable backlog entry..." presente
  em `docs/PROTOCOL.md`; a tabela da ADR-0003 permanece com o mesmo
  identificador e sem linha nova.
- Criteria: none
- Decisions: impossibilidade de confrontar o caminho e a mesma violacao, nao uma
  paralela (decisao da spec 011); reutilizar o identificador mantem o contrato
  estavel.

## 2026-09-11 - T-002 - backlog-id-mismatch completo no contrato
- Backlog: B-017
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: a secao Integrity checks de `docs/PROTOCOL.md` passou a exigir que o
  ID de backlog ativo exista no backlog, alem de ser o mesmo no handoff e no
  TODO. O identificador `backlog-id-mismatch` foi mantido: nenhum check novo.
- Evidence: item "Handoff, TODO, and backlog records do not agree..." reescrito
  em `docs/PROTOCOL.md`; a tabela da ADR-0003 permanece com o mesmo
  identificador e sem linha nova.
- Criteria: none
- Decisions: ausencia do ID no backlog e a mesma violacao, nao uma paralela
  (decisao da spec 011); reutilizar o identificador mantem o contrato estavel.

## 2026-09-11 - T-001 - Derivacao de done e idle explicitada no contrato
- Backlog: B-017
- Spec: .specs/20260907-011-integridade-do-estado-e-da-evidencia.md
- Result: `docs/PROTOCOL.md` (secao Allowed statuses) passou a derivar os
  estados finais de forma explicita: backlog nao vazio integralmente `[x]` e
  `done`; ausencia de backlog, TODO e handoff e `idle`; backlog vazio nao e
  status e os demais registros decidem o estado.
- Evidence: regra presente em `docs/PROTOCOL.md` antes de qualquer mudanca de
  codigo; semantica normativa da ADR-0003 preservada (fixtures `done`/`idle`
  ja representavam as duas formas).
- Criteria: none
- Decisions: a regra e de contrato em prosa; o alinhamento de relay-session,
  relay-status e relay-core e o B-018, que nomeara A-001 quando os tres
  concordarem com o contrato.

## 2026-09-11 - T-004 - Construir e validar visualmente o observador
- Backlog: B-031
- Spec: .specs/20260910-001-observador-read-only.md
- Result: o observador foi aberto num browser real contra este repositorio
  (`--workspace`, sem `--exec`) e cada rodada de validacao virou correcao.
  (1) `TODO.md` tinha cabecalho duplicado (`# Active task` sem ID antes de
  `# Active task: B-031`), e como `parseTodo` usa `match` sem `/g` ele casava
  o primeiro: `activeBacklogId` resolvia `null`, o estado inteiro virava
  `inconsistent` e o backlog de toda spec aparecia vazio na aba Trabalho.
  (2) A aba Agora passou a caber numa viewport: as duas caixas do HandoffCard
  tem largura e altura iguais (grade `1fr 1fr` e o mesmo teto
  `--handoff-context-max-height`), cada uma com rolagem propria; o card leva o
  tom do status na moldura; subtarefas em ordem decrescente num painel com
  rolagem propria; `.app` virou `100dvh` com `.app__body` como unico filho
  rolavel. (3) A aba Trabalho virou cascata de cartoes — spec -> cartoes de
  backlog -> cartoes de changelog filtrados por `backlogId` — com rolagem
  independente por coluna, botao "Tarefas" abrindo o modal de subtarefas do
  backlog, e a aba escolhida sobrevivendo ao reload via `sessionStorage`.
  (4) A tela Escolher deixou de estar inteira atras de `execEnabled`: em
  read-only ela lista as tarefas disponiveis e diz por que a escolha e do
  usuario; so as acoes de lancamento seguem condicionadas.
- Evidence: A-003 e A-004 conferidos na tela pelo usuario — nenhum selo,
  preflight, terminal ou aviso de harness montado, com a proveniencia
  "Escrito no claude · ha N min" visivel no card; a aba Trabalho mostrando a
  lista de specs com caminho e os cartoes de backlog da spec selecionada. O
  estado voltou a `{ kind: 'ok', status: 'in_progress', activeBacklogId:
  'B-031' }` apos a correcao do `TODO.md`, verificado por `deriveState` direto.
  `node --test`: relay-core 25/25, relay-host 22/22, relay-ui 5/5; `tsc
  --noEmit`, `vue-tsc --noEmit` e `vite build` limpos.
- Criteria: A-003, A-004
- Decisions: mudanca no codigo do `relay-host` exige reiniciar o processo — o
  `dist/` da UI e relido do disco a cada request, mas o servidor e o binario
  em memoria; foi isso que fez a rota nova do changelog devolver 404 por uma
  rodada inteira. No modal de tarefas, backlog que nao e o ativo nao tem mais
  `TODO.md`: suas subtarefas so existem como registros de changelog, e o modal
  lista essas, todas concluidas, em vez de inventar marcador para subtarefa
  que nao pode mais ser observada. A-008 e A-012 ficaram sem confirmacao
  visual e seguem para B-032.

## 2026-09-10 - T-003 - Remover a superficie de execucao da composicao read-only
- Backlog: B-031
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `App.vue` deixou de montar `HarnessSelector`, `PreflightModal` e
  `KeyboardWarning` incondicionalmente e de chamar `GET /api/harnesses` sempre
  que havia host — as tres montagens e a busca agora dependem de
  `payload.environment.execEnabled` (`execEnabled` como `computed`, e a busca
  de harnesses movida do `onMounted` fixo para um `watch(execEnabled, ...,
  { immediate: true })`). `Header.vue` e `MainScreen.vue` ja gateavam o selo e
  a acao de execucao por `v-if="execEnabled"`; nada mudou neles. `activeRunId`
  so e escrito por `launchEmbedded`, chamada apenas pelo `PreflightModal`
  agora desmontado sob read-only, entao `ExecutionMode`/`BackgroundStrip`
  permanecem inalcancaveis sem exec.
- Evidence: nao ha infraestrutura de mount de SFC neste pacote (sem
  `@vue/test-utils`/jsdom nas devDependencies) — verificacao seguiu o padrao ja
  usado nas mudancas de composicao anteriores (B-014/T-005): leitura integral
  de `App.vue` confirma as tres montagens sob `v-if="execEnabled"` e a busca de
  harnesses fora do `onMounted` incondicional. `vue-tsc --noEmit` e `npm run
  build` (vite build) terminaram sem erros.
- Criteria: A-003
- Decisions: o "next step" do handoff previa um teste montando `App.vue`; a
  investigacao mostrou que este pacote nao tem harness de teste de componente
  Vue, entao a verificacao seguiu a convencao ja registrada no changelog para
  composicao condicional (leitura + typecheck + build), sem introduzir uma
  dependencia de teste nova so para esta mudanca.

## 2026-09-10 - T-002 - Tornar spec, backlog e changelog coerentes sob atualizacao
- Backlog: B-031
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `lib/work.ts` (`reconcileSpecId`, `createLatestRequest`) cai
  deterministicamente para a primeira spec quando a selecionada desaparece e
  aborta/ignora requisicoes anteriores por revisao. `WorkScreen.vue` passou a
  usar as duas unidades: `reload()` reconcilia a selecao a cada mudanca de
  `payload` e descarta specs/changelog de uma revisao que deixou de ser a
  atual antes de aplicar o resultado; `loadSpecText` busca
  `/api/specs/<id>` e exibe o texto bruto da spec selecionada (mono, acima do
  backlog filtrado), com a mesma guarda de revisao.
- Evidence: `test/work.test.ts` falhou primeiro por ausencia de `lib/work.ts`;
  depois da implementacao, `node --test` passou 5/5 (specs, host, observer,
  work). `vue-tsc --noEmit` terminou sem erros apos ligar `loadSpecText` e a
  guarda do changelog ao mesmo `listRequests`.
- Criteria: A-004, A-005, A-007
- Decisions: uma unica revisao (`listRequests`) protege specs e changelog
  juntos, porque ambos nascem do mesmo `reload()` e uma resposta tardia de
  qualquer um dos dois precisa ser descartada quando uma revisao mais nova ja
  comecou; o texto da spec usa uma revisao propria (`textRequests`) porque
  troca de selecao é independente de um novo `reload()`.

## 2026-09-10 - T-001 - Consumir snapshots e expor frescor da conexao na UI
- Backlog: B-031
- Spec: .specs/20260910-001-observador-read-only.md
- Result: a relay-ui reduz frames `snapshot`, `refreshing` e desconexao num
  unico estado de view, preserva o ultimo payload e mostra no Header os rotulos
  Conectando, Atualizado, Atualizando ou Desatualizado com regiao `status`.
- Evidence: o teste da UI falhou primeiro pela ausencia do reducer e do rotulo;
  depois passou 3/3 cobrindo preservacao do snapshot, stale e os quatro textos.
  `vue-tsc --noEmit` terminou sem erros.
- Criteria: A-008
- Decisions: frescor e estado do cliente, nao campo de apresentacao do
  `relay-core`; fixture mode aparece sempre como Atualizado.

## 2026-09-10 - T-003 - Verificar o contrato reativo do relay-host
- Backlog: B-030
- Spec: .specs/20260910-001-observador-read-only.md
- Result: o servidor retem o estado de transicao, entrega `refreshing` a quem
  conecta durante a janela e so retorna a snapshot depois de `broadcast`; as
  garantias HTTP, autenticacao e terminal permaneceram intactas.
- Evidence: a suite completa revelou a corrida ao receber snapshot sem
  `refreshing`; um teste deterministico reproduziu conexao durante transicao e
  falhou antes da correcao. Depois, o teste passou e tres execucoes completas
  concorrentes passaram 21/21; typecheck do host terminou sem erros.
- Criteria: A-006
- Decisions: `refreshing` e estado retido do servidor, nao apenas um evento
  efemero, para que clientes tardios nunca recebam snapshot intermediario.

## 2026-09-10 - T-002 - Publicar snapshot apenas apos 150 ms de quiescencia
- Backlog: B-030
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `watchWorkspace` sinaliza o primeiro evento de uma transicao, reinicia
  uma janela trailing a cada escrita e publica um unico snapshot depois de 150
  ms sem eventos; o wiring do host separa `broadcastRefreshing` de `broadcast`.
- Evidence: o teste integrado falhou primeiro porque a API antiga aceitava um
  callback unico e usava 40 ms; depois da implementacao, agrupou duas escritas,
  observou `refreshing`, recusou snapshot prematuro e passou apos a quiescencia.
- Criteria: A-006
- Decisions: o sinal `refreshing` ocorre uma vez por rajada; eventos seguintes
  apenas reiniciam a janela ate `onSettled`.

## 2026-09-10 - T-001 - Envelopar snapshots e sinalizar transicao no WebSocket
- Backlog: B-030
- Spec: .specs/20260910-001-observador-read-only.md
- Result: o canal de estado envia mensagens discriminadas `snapshot` e
  `refreshing`; conexao e broadcast estavel envelopam o `UiPayload`, e o
  servidor expoe um broadcast proprio para inicio de transicao.
- Evidence: o teste do servidor falhou primeiro ao receber o payload antigo sem
  `kind`; depois da implementacao, `test/server.test.ts` passou 9/9 e observou
  tanto o snapshot inicial quanto o frame `refreshing` real via WebSocket.
- Criteria: none
- Decisions: a semantica de transporte fica no relay-host; `relay-core`
  continua puro e alheio ao WebSocket.

## 2026-09-10 - T-003 - Verificar o contrato read-only e a entrada multi-workspace
- Backlog: B-029
- Spec: .specs/20260910-001-observador-read-only.md
- Result: o contrato documentado e o entrypoint concordam sobre read-only por
  default, opt-in de execucao, compatibilidade de `--no-exec` e workspace
  resolvido; nenhuma API programatica existente do host foi removida.
- Evidence: `npm run typecheck --workspace relay-host` terminou sem erros;
  `npm test --workspace relay-host` passou 20/20; `git diff --check` terminou
  limpo; `git status` confirma que as mudancas da nova spec seguem sem commit.
- Criteria: A-001, A-002
- Decisions: none.

## 2026-09-10 - T-002 - Implementar defaults, flags e workspace explicito no relay-host
- Backlog: B-029
- Spec: .specs/20260910-001-observador-read-only.md
- Result: `parseCliArgs` torna read-only o default, habilita execucao somente
  com `--exec`, faz `--no-exec` vencer combinacoes contraditorias e resolve
  `--workspace=<path>` contra o cwd; o entrypoint usa o resultado e informa o
  workspace observado.
- Evidence: `test/cli.test.ts` falhou primeiro porque `parseCliArgs` nao existia;
  depois da implementacao, o teste focado passou 4/4 e a suite completa do
  relay-host passou 20/20, incluindo rotas e WebSocket em loopback.
- Criteria: A-001, A-002
- Decisions: o parser e puro para testar o contrato sem mutar `process.argv`;
  `start` e `createRelayServer` mantiveram suas interfaces.

## 2026-09-10 - T-001 - Registrar a decisao arquitetural do observador read-only
- Backlog: B-029
- Spec: .specs/20260910-001-observador-read-only.md
- Result: ADR-0007 registra read-only como default seguro, `--exec` como opt-in,
  workspace explicito, snapshots apos quiescencia e frescor visivel; o design
  system passa a condicionar controles de harness a execucao e inclui o
  conteudo textual da spec na visao Trabalho.
- Evidence: a ADR contem Status, Contexto, Decisao, Consequencias, Compliance e
  Notes; o indice de ADRs aponta para o arquivo; as regras de componente foram
  atualizadas antes do codigo e nenhum token visual mudou.
- Criteria: none
- Decisions: ADR-0007 complementa, sem superseder, a ADR-0006; o transporte e
  as garantias de seguranca existentes permanecem.

## 2026-09-07 - T-001 - Foco visível em todos os controles
- Backlog: B-016
- Spec: .specs/20260907-010-acessibilidade-e-remanescentes.md
- Result: regra global `:focus-visible { outline: 1px solid var(--blue-line);
  outline-offset: 2px }` acrescentada e o `outline: none` do
  `.preflight__prompt:focus` removido — que era a única regra suprimindo o anel
  e violava a regra 3 da seção 7. Nenhum token novo: `--blue-line` já era o
  anel autorizado pela seção 7.
- Evidence: `grep` por `outline: none` em `app.css` retorna vazio; `grep` por
  `:focus-visible` retorna a regra global. Nenhum `#hex`/`rgba(` literal novo.
- Criteria: A-005
- Decisions: anel único global, em vez de `:focus` por componente — contraste
  via `--blue-line` (mais forte que `--line-2`, o outro anel permitido).

## 2026-09-07 - T-002 - Contenção de foco no PreflightModal
- Backlog: B-016
- Spec: .specs/20260907-010-acessibilidade-e-remanescentes.md
- Result: `lib/focus-trap.ts` prende Tab/Shift+Tab dentro do overlay
  (`trapFocus`), retornando o release. `PreflightModal.vue` instala o trap no
  `ref="overlay"` quando abre (`preflight.open` → `nextTick` → `trapFocus`) e
  solta no fechamento e no `onBeforeUnmount`.
- Evidence: `trapFocus` itera só elementos focáveis visíveis (offsetParent);
  o Tab cicla primeiro↔último sem vazar do modal. typecheck/build limpos.
- Criteria: A-005
- Decisions: trap em composable reutilizável; a cláusula de foco do A-005 é
  sobre o PreflightModal, então o trap só é instalado ali (o seletor standalone
  e o KeyboardWarning mantêm Esc/clique-fora, já focáveis pela regra global).

## 2026-09-07 - T-003 - Auditoria documentada por componente × cinco regras
- Backlog: B-016
- Spec: .specs/20260907-010-acessibilidade-e-remanescentes.md
- Result: os componentes existentes (StatusPill, HandoffCard, ChecklistList,
  RepairScreen, Header, EmptyState, MainScreen, seletor de harness e
  consentimento, três colunas do WorkScreen, PreflightModal, barra e controles
  do modo de execução, faixa de segundo plano) foram conferidos contra as cinco
  regras da seção 7. Única falha: "foco visível" (regra 3) — corrigida em
  T-001/T-002. Regras 1 (contraste AA), 2 (cor+texto), 4 (alvos 32-36px) e 5
  (hierarquia) passam em todos. Achado de desatualização: a spec afirma que "só
  a primária existe no HandoffCard", mas a secundária já foi criada em B-012.
- Evidence: contraste vem dos tokens AA da seção 2 e não há literal de cor fora
  de `tokens.css` (verificado em B-004, re-confirmado aqui por `grep`); todo
  status tem rótulo textual (StatusPill e ChecklistList); todo controle usa
  `--control-height`/`--control-height-sm` (36/32px).
- Criteria: A-001, A-003
- Decisions: a auditoria é manual (non-goal da spec) e não gerou entrada nova
  de backlog — nenhuma falha exigiu decisão de token nova além do já previsto
  na seção 7. Nenhuma decisão visual nova foi tomada.

## 2026-09-07 - T-004 - Painel "Gravado em disco" finalizado
- Backlog: B-016
- Spec: .specs/20260907-010-acessibilidade-e-remanescentes.md
- Result: `DiskLog.vue` passou a formatar o horário (`formatRelative` em vez do
  no-op anterior). O painel já renderiza uma entrada por escrita, mais recente
  primeiro (`reversed`), com selo `ATUALIZADO`/`LIMPO`, caminho, horário, linha
  de prosa do significado e colunas Antes/Depois — e o `LIMPO` de um handoff
  esvaziado aparece como qualquer entrada (o tracker do host emite
  `type: 'cleared'` quando `after === ''`). Contador `N arquivos alterados` é a
  contagem real (A-004); o da faixa de segundo plano usa o mesmo `writtenFiles`.
- Evidence: `disk.ts` (host) difere os quatro registros de `.orchestration/`
  por run; o cliente consome `disk` frames via `onDisk`/`useDisk()`. A variante
  `button--danger` é usada só por "Encerrar processo" (ExecutionMode), confirmado
  por `grep` — nenhuma ação não destrutiva usa perigo.
- Criteria: A-002, A-003, A-004
- Decisions: contador nunca é percentual nem posição — só `entries.length` e
  `writtenFiles` (mesma regra da ADR-0003 sobre contagem).

## 2026-09-07 - T-001 - PTY no relay-host por processo nativo
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: `relay-host/src/pty.ts` aloca um PTY por execução **sem dependência
  nativa** (decisão com o usuário, honrando a ADR-0004 contra `npm install`
  de módulo nativo): `script -q -e -F <fifo> -- <bin> <args...>` no macOS cria
  o PTY, grava o fluxo num fifo lido com `O_NONBLOCK` (nada bloqueia) e
  propaga o exit code do filho via `-e`. O handle expõe `write` (teclas →
  stdin), `scrollback()` (acumulado por run), `terminate` (SIGHUP, mata) e os
  callbacks `onData`/`onExit`. `executor.ts` mantém o conjunto de runs ativos.
- Evidence: `node --test` do host 16/16; typecheck limpo; smoke test do host
  (start/close limpos). `script -F <fifo>` validado empiricamente: alocou PTY,
  capturou `hello_pipe\r\n` e devolveu exit 0. Desanexar só fecha o WebSocket
  do cliente — o processo e o scrollback ficam no host.
- Criteria: none
- Decisions: PTY sem `node-pty` (módulo nativo) — a alternativa foi registrada
  como violação da ADR-0004 e descartada. `resize` é no-op porque o tamanho da
  PTY do `script` é fixo por processo; o fit do xterm é client-side.

## 2026-09-07 - T-002 - Canal de execução por WebSocket no relay-host
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: `server.ts` ganhou `/ws/term` (mesmo token/Origem/subprotocol do
  `/ws`), multiplexando frames `data`/`exit`/`disk` por run; `POST
  /api/launch/embedded` (404 sob `--no-exec`), `GET /api/runs`,
  `GET /api/run/<id>` e `POST /api/run/<id>` (terminate). O host envia o
  scrollback guardado no `attach` de uma reconexão, satisfazendo o replay.
- Evidence: teste novo em `server.test.ts` (launch/embedded 404 sem
  `launchEmbedded`); typecheck limpo; `/ws` de payload intacto (15 testes
  originais seguem passando).
- Criteria: none
- Decisions: um segundo endpoint WS separa Canal A (fluxo do terminal) do
  Canal B (payload de estado), mesma tese do design system — o estado vem do
  disco, nunca do stdout parseado.

## 2026-09-07 - T-003 - Componente Terminal isolado do re-render do Vue
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: `Terminal.vue` monta `@xterm/xterm` num `<div ref>` próprio com
  atualização **somente imperativa** — `term.write` no callback de dado, fit
  por `ResizeObserver`; nenhum binding reativo mira o conteúdo do container.
  `lib/term-client.ts` abre `/ws/term` e `lib/execution.ts` mantém o store de
  execução (launch/detach/reattach/terminate) com buffer de scrollback no
  cliente.
- Evidence: `vue-tsc`/`vite build` limpos; `grep` por `v-html`/`innerHTML` em
  `Terminal.vue` retorna vazio — o subárvore do xterm nunca é tocado pelo
  ciclo reativo.
- Criteria: A-001, A-003
- Decisions: xterm.js habilita alt-screen, bracketed paste e mouse tracking
  por padrão no protocolo terminal; nada foi desabilitado.

## 2026-09-07 - T-004 - Modo de execução na UI
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: `ExecutionMode.vue` ocupa a viewport (`position: fixed; inset: 0`)
  **sem** as abas Agora/Trabalho quando há run anexada; barra com selo do
  harness (tom de identidade), nome da execução, StatusPill e controles por
  estado — `em execução` tem "Deixar em segundo plano" (secundário) e
  "Encerrar processo" (perigo), `concluído` só "Fechar". `KeyboardWarning.vue`
  mostra o aviso de conflito `Cmd+W`/`Cmd+T` na primeira execução, com a
  alternativa "modo externo".
- Evidence: `v-if="exec.status === 'running'"` restringe os controles por
  estado; o `v-else` de "Fechar" só renderiza quando o status é `exited` —
  inalcançável com processo vivo (A-006). O aviso fica sob
  `firstRunThisSession` (A-004).
- Criteria: A-004, A-006, A-007
- Decisions: "Encerrar processo" exige um segundo clique que **nomeia** o que
  será descartado (`Descartar {nome}?`); desanexar nunca confirma.

## 2026-09-07 - T-005 - Faixa de segundo plano e painel "Gravado em disco"
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: `BackgroundStrip.vue` aparece só com execução desanexada, com
  contador verdadeiro de arquivos escritos e "Reconectar ao terminal" — a
  reconexão reanexa e exibe o selo `RECONECTADO À EXECUÇÃO VIVA`.
  `DiskLog.vue` é a coluna direita do modo de execução; `disk.ts` no host
  tira diff dos quatro registros de `.orchestration/` e emite `disk` frames
  pelo `/ws/term`, então o painel recebe entradas **durante** a execução, não
  só ao final.
- Evidence: `disk.ts` com `diff()` incremental por run (snapshot + comparação,
  tipo `ATUALIZADO`/`LIMPO`); frames `disk` lidos pelo `onDisk` do cliente e
  empurrados no `useDisk()`. Typecheck/build limpos.
- Criteria: A-005, A-008
- Decisions: o diff Antes/Depois completo (spec 010, A-002) compartilha o
  mesmo tracker de disco; aqui entrega a presença e o fluxo ao vivo.

## 2026-09-07 - T-006 - Evidência: testes, typecheck, build e ordem de estado
- Backlog: B-015
- Spec: .specs/20260907-009-terminal.md
- Result: fechamento da integração — o `status` da barra só vira `exited` pelo
  callback `onExit` do socket, que por sua vez só dispara no `close` do filho
  no host. O `pillStatus` deriva `in_progress`→green / `exited`→`done`, então
  "concluído" jamais antecede a saída do processo.
- Evidence: relay-core 24/24 e relay-host 16/16 em `node --test`;
  `vue-tsc --noEmit` limpo nos três pacotes; `vite build` limpo; host faz
  start/close limpos (smoke). A-009 é garantido por construção: estado → tom
  via piloto de execução, não por polling.
- Criteria: A-002, A-009
- Decisions: nenhuma.

## 2026-09-07 - T-005 - Quatro portas de lançamento num modal só
- Backlog: B-014
- Spec: .specs/20260907-008-preflight-e-lancamento.md
- Result: as quatro portas abrem o mesmo `PreflightModal` via `openPreflight`:
  "+ nova spec" (WorkScreen) e "Iniciar entrevista" (tela Escolher) →
  `relay-spec`/“Especificar uma ideia”; "Começar" (por tarefa disponível) →
  `relay-session`/“Iniciar sessão em {id}”; "Retomar" (HandoffCard) →
  `relay-session`/“Retomar sessão”. Toda porta fica oculta quando
  `environment.execEnabled` é falso, então sob `--no-exec` o modal nunca é
  renderizado.
- Evidence: `vue-tsc`/`vite build` limpos; `grep` por `openPreflight` encontra
  só as três origens, todas atrás de `v-if="execEnabled"`; `grep` por outra
  rota chamando `/api/launch` não encontra nenhuma além do `PreflightModal`.
  Smoke test do host: `/api/launch/preview` devolve o plano e `/api/harnesses`
  devolve a detecção real.
- Criteria: A-003, A-007
- Decisions: o 404 da rota sob `--no-exec` já estava testado em T-002; aqui
  fecha a outra metade do A-003 (portas ocultas → modal nunca renderizado).

## 2026-09-07 - T-004 - PreflightModal completo com argv real
- Backlog: B-014
- Spec: .specs/20260907-008-preflight-e-lancamento.md
- Result: `PreflightModal.vue` reescrito: tabela `bin`/`arg`/`prompt`/`cwd` com
  uma linha por elemento real do argv (vinda de `POST /api/launch/preview`,
  nunca string montada), campo `prompt` editável (a intenção), seletor de
  harness e consentimento embutidos, aviso de execução e um único botão
  "▶ Executar no {harness}". Trocar o harness re-compõe as linhas e o rótulo do
  botão. Novo store `lib/launch.ts` (`openPreflight`/`closePreflight`/
  `usePreflight`) e `apiPostJson` no relay-client. `App.vue` renderiza um único
  modal; `WorkScreen` abre via store.
- Evidence: `vue-tsc` e `vite build` limpos; o preview vem do host
  (autoridade única do argv) e o confirm envia `{ harness, skill, intent }`
  para `POST /api/launch`. Esc/Cancelar chamam `closePreflight` sem lançar; o
  modal não fecha por clique fora (só o seletor standalone fecha assim).
  Fixture mode usa `localPreview` dev-only.
- Criteria: A-002, A-004, A-008, A-009
- Decisions: a linha `prompt` da tabela é o elemento composto do argv (prefixo
  + intenção); o campo editável é a intenção, que re-compõe a linha via
  preview. O consentimento nunca dispensa o botão de confirmar.

## 2026-09-07 - T-003 - Seletor e selo de harness com dado real
- Backlog: B-014
- Spec: .specs/20260907-008-preflight-e-lancamento.md
- Result: `lib/harness.ts` trocou a fonte de dado de `HARNESS_FIXTURE` por um
  store reativo (`setHarnesses`/`allHarnesses`/`harnessById`); `App.vue` carrega
  `GET /api/harnesses` no modo host e cai na fixture quando não há host.
  `HarnessSelector`, `Header` e `HandoffCard` passaram a ler da lista real, sem
  mudança de forma (mesma marcação, tons e estados).
- Evidence: `vue-tsc --noEmit` limpo; os três componentes não importam mais
  `HARNESS_FIXTURE` para renderizar a lista (só como fallback de último recurso
  no `Header`); em fixture mode a lista continua exatamente a anterior.
- Criteria: A-006
- Decisions: a detecção real já existia no host (`detectHarnesses`); esta
  mudança só a conecta à view. A fixture permanece como rota de desenvolvimento.

## 2026-09-07 - T-002 - Rota de lançamento com argv[] e modo externo
- Backlog: B-014
- Spec: .specs/20260907-008-preflight-e-lancamento.md
- Result: `relay-host/src/launcher.ts` com `preview` e `launch`. `launch` grava
  um script wrapper em área de scratch (tmpdir/relay-run/<id>), com cada
  elemento do argv entre aspas simples, `echo $$ > pid` e `echo $? > exit`
  gravados pelo próprio script, e abre via `open -a <emulador>` (spawn com
  argv, sem shell). `server.ts` registra `POST /api/launch` e
  `POST /api/launch/preview`, presentes só quando `execEnabled` — sob
  `--no-exec`, requisição autenticada devolve `404`, não `403`.
- Evidence: `tsc --noEmit` limpo; 15 testes passam, incluindo os novos de
  `launcher.test.ts` (script wrapper, shellQuote, preview==launch) e o de
  integração (preview compõe `claude`/`-p`/`/relay-session …`, launch devolve
  runId, harness desconhecido → 400). `grep` por `exec(`, `shell: true`,
  `sh -c`, `bash -c` em `relay-host/src` retorna vazio.
- Criteria: A-001, A-005
- Decisions: modo externo é o único lançamento nesta spec (o embutido é a spec
  009); o emulador é detectado entre Terminal/iTerm/Ghostty/Warp no macOS e o
  `open` é chamado com argv. O script É o argv, nunca `sh -c`.

## 2026-09-07 - T-001 - Adaptador de harness: buildArgv e composePrompt
- Backlog: B-014
- Spec: .specs/20260907-008-preflight-e-lancamento.md
- Result: `relay-host/src/harness.ts` ganhou `launchArgs` e `promptPrefix` por
  harness e as duas funções separadas que a spec exige: `composePrompt(harness,
  skill, intent)` (claude `/relay-session`, codex `⟨relay-session⟩`, opencode
  `Use relay-session`) e `buildLaunchArgv(harness, skill, intent, cwd)`
  devolvendo `{ bin, args, prompt, cwd }`. Nenhum prefixo contém `--skill`.
- Evidence: `tsc --noEmit` limpo no relay-host; os três prefixos conferem com
  o design system (seção 6) e com `docs/INSTALL.md` (opencode invoca skill por
  linguagem natural, sem slash command).
- Criteria: none
- Decisions: o prompt composto é um único elemento de argv (o último), nunca
  dividido em flag; `buildArgv` e `composePrompt` separados é o que torna
  impossível reintroduzir a suposição do `--skill`.

## 2026-09-07 - T-001 - Contrato do estado derivado
- Backlog: B-001
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: ADR-0003 escrita, com o tipo do estado derivado e sete fixtures
  normativos (seis status mais `inconsistent`). Oito decisões, incluindo união
  discriminada para `inconsistent`, `available` derivado no core, e recusa
  explícita de percentual, posição, prioridade, tom e nome de tela.
- Evidence: seis seções obrigatórias do formato de ADR presentes; os dois blocos
  JSON parseiam; a afirmação "doze verificações de integridade" conferida contra
  `docs/PROTOCOL.md` (12). Spec atualizada de seis para sete fixtures.
- Decisions: o contrato carrega o que o protocolo determina; o design system
  determina como aquilo aparece. Tom e tela ficam fora do contrato para não
  inverter a governança do `docs/design-system/README.md`.

## 2026-09-07 - T-002 - Identificadores estaveis das verificacoes
- Backlog: B-001
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: decisao 9 acrescentada a ADR-0003, com as doze verificacoes de
  integridade nomeadas na ordem do protocolo. Conformidade 7 passou a apontar
  para a tabela e a exigir que verificacao nova acrescente linha na mesma
  mudanca.
- Evidence: nenhum `check` citado fora da tabela; os dois exemplos normativos
  validados por script — `completed`/`total` batem com o array de TODO,
  `available` bate com marcador mais `needs`, e o exemplo `inconsistent` nao
  carrega `status`, `handoff` nem `todo`.
- Decisions: dois identificadores citados na ADR nao existiam na tabela
  (`handoff-names-completed-todo`, `handoff-todo-mismatch`) e foram alinhados.
  O identificador e contrato: renomear exige supersedir a ADR.

## 2026-09-07 - T-003 - ADR-0003 no indice
- Backlog: B-001
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: ADR-0003 registrada no indice de ADRs do AGENTS.md.
- Evidence: o indice lista 0001, 0002 e 0003; o arquivo referenciado existe.
- Decisions: nenhuma.

## 2026-09-07 - T-001 - ADR-0004 da fronteira do app/
- Backlog: B-002
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: ADR-0004 escrita com seis decisoes: contencao num unico diretorio,
  sem package.json na raiz, sem passo de build obrigatorio, instrucoes proprias
  da pasta, a terceira camada declarada (superficie de pacote / produto /
  ferramenta deste repo), e a proibicao de escrever nos cinco registros.
- Evidence: seis secoes obrigatorias do formato de ADR presentes; a decisao 3
  registra explicitamente que trata de obrigatoriedade e nao de veto ao
  framework, para nao prejulgar a ADR-0005.
- Decisions: a estrutura fica separada do framework porque sobrevive a troca
  dele; a ADR-0005 pode revisar apenas a decisao 3.

## 2026-09-07 - T-002 - Estrutura criada e garantias verificadas
- Backlog: B-002
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: criados app/package.json (private, workspaces relay-*), app/AGENTS.md
  e o symlink app/CLAUDE.md. relay-ui/ nao foi criado: diretorio vazio nao e
  versionavel e nasce em B-004, com conteudo.
- Evidence: as seis conformidades da ADR-0004 verificadas por comando, nao por
  leitura. Num clone limpo com rm -rf app/: skills intactas, manifestos
  intactos, symlinks de skill resolvendo, guarda ainda executavel. app/CLAUDE.md
  gravado com modo 120000. Nenhum arquivo fora de app/ resolve caminho para
  dentro dele.
- Decisions: a Conformidade 3 estava imprecisa — dizia "depende" e o teste
  pegou mencoes em prosa. Reescrita para "resolve um caminho": link, import,
  symlink, manifesto ou script. Prosa que cita app/ nao e dependencia.

## 2026-09-07 - T-003 - ADR-0004 no indice e ponteiro no roteador
- Backlog: B-002
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: ADR-0004 registrada no indice de ADRs e uma linha acrescentada a
  tabela de roteamento do AGENTS.md apontando para app/.
- Evidence: indice lista 0001 a 0004; a tabela ganhou exatamente uma linha; o
  arquivo referenciado existe.
- Decisions: nenhuma.

## 2026-09-07 - T-001 - ADR-0005 do framework da relay-ui
- Backlog: B-003
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: Vue 3 escolhido; Lit, Preact e React+Vite descartados com o motivo de
  cada um. Registrado que o custo de migracao foi considerado e descartado como
  criterio, e por que o raciocinio "invista pouco porque e descartavel" estava
  errado em duas frentes.
- Evidence: seis secoes obrigatorias presentes; a decisao 1 cita a secao 8 do
  design system, que pede CSS global com classe por componente e derruba o
  argumento de shadow DOM que sustentava o Lit.
- Decisions: autoria em SFC com TypeScript, porque a garantia da ADR-0003 sobre
  `inconsistent` e do compilador; sem checagem de tipo na UI ela vira convencao.
  JSDoc com --checkJs foi considerado e descartado por custo ergonomico.

## 2026-09-07 - T-002 - ADR-0004 decisao 3 estreitada
- Backlog: B-003
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: a decisao 3 da ADR-0004 passou de "sem passo de build obrigatorio"
  para "sem passo de build entre o clone e as skills". app/ tem o proprio build.
- Evidence: a cadeia esta visivel nos dois sentidos — 0004 aponta para 0005 na
  nota de estreitamento e nas consequencias; 0005 aponta para 0004 na decisao 5
  e nas notas. Nada foi deletado.
- Decisions: estreitar, nao remover. O proposito da decisao original era
  proteger a instalacao das skills, e essa garantia continua intacta e
  verificavel; so a redacao excessiva caiu.

## 2026-09-07 - T-003 - ADR-0005 no indice
- Backlog: B-003
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: ADR-0005 registrada no indice de ADRs do AGENTS.md.
- Evidence: indice lista 0001 a 0005; todos os arquivos citados existem.
- Decisions: nenhuma.

## 2026-09-07 - T-001 - Templates de spec e changelog
- Backlog: B-005
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: criterios de aceite passaram a `A-NNN` sem marcador de checklist, com
  uma secao nova explicando que satisfacao e derivada do changelog e nunca
  escrita na spec. O registro de changelog ganhou o campo `Criteria`.
- Evidence: template de spec e de changelog atualizados em docs/PROTOCOL.md; a
  secao registra que specs com marcador continuam validas e nao sao reescritas.
- Criteria: A-001, A-002, A-007
- Decisions: o marcador foi omitido de proposito. Um `[ ]` num criterio le-se
  como pendencia que vai fechar, quando nada no protocolo o fecha — foi ele que
  permitiu marcar B-001 concluida com oito criterios aparentemente abertos.

## 2026-09-07 - T-002 - Transicoes e verificacao de integridade
- Backlog: B-005
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: transicao 4 exige que `Criteria` nomeie os criterios avancados ou
  declare `none`; transicao 5 recusa fechar a ultima entrada de backlog de uma
  spec quando algum criterio dela nao tem evidencia, com `[!]` e handoff
  bloqueado nomeando quais faltam. Verificacao de integridade nova.
- Evidence: o protocolo passou de 12 para 13 verificacoes; a transicao 5 usa o
  caminho de rejeicao que ja existia, sem status novo.
- Criteria: A-003, A-004
- Decisions: `none` e afirmacao como qualquer outra e precisa ser verdadeira —
  sem isso o campo vira carimbo.

## 2026-09-07 - T-003 - Identificador estavel da verificacao nova
- Backlog: B-005
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: `criteria-without-evidence` acrescentado a tabela da ADR-0003 na mesma
  mudanca, como a Conformidade 7 exige.
- Evidence: script conferiu 13 verificacoes no protocolo e 13 identificadores na
  ADR-0003.
- Criteria: A-005
- Decisions: as tres mencoes a "doze verificacoes" na ADR-0003 foram trocadas
  por formulacao sem contagem, e a Conformidade 7 passou a proibir declarar um
  total. Contagem em prosa apodrece a cada verificacao nova — esta mudanca ja a
  teria quebrado.

## 2026-09-07 - T-001 - Skills alinhadas ao contrato
- Backlog: B-006
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: relay-spec passou a escrever criterios como `A-NNN` sem marcador;
  relay-session passou a exigir `Criteria` no registro e a confirmar que todo
  criterio da spec tem evidencia antes de fechar a ultima entrada de backlog
  dela; relay-status passou a tratar a ausencia disso como `inconsistent`.
- Evidence: as tres skills citam o contrato novo; a regra da transicao 5 esta em
  relay-session linha 35.
- Criteria: A-006
- Decisions: nenhuma skill nova. As tres mudancas cabem nas existentes, o que
  mantem o registro de cinco skills que a instalacao ja documenta.

## 2026-09-07 - T-002 - Limites e vocabulario verificados
- Backlog: B-006
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: as cinco skills voltaram a ficar abaixo de 40 linhas apos a expansao,
  por compressao de paragrafos anteriores em relay-spec e relay-session.
- Evidence: contagem por arquivo — continue 39, session 39, setup 39, spec 39,
  status 29. Nenhuma linha de prosa larga introduzida, medido contra HEAD em
  caracteres e nao em bytes. Frontmatter intacto nas cinco. Nenhum resquicio de
  `unblocked` nem de contagem em prosa.
- Criteria: A-006
- Decisions: a expansao custou seis linhas e foram todas recuperadas cortando
  redundancia, nao conteudo — o limite forcou concisao em texto que ja estava
  prolixo.

## 2026-09-07 - T-001 - Spec 20260907-001 convertida
- Backlog: B-007
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: os oito criterios da spec da UI passaram de checkbox para A-001..A-008
  sem marcador. O A-003 foi reescrito para dizer "existem **como arquivos**",
  porque a redacao anterior era ambigua o bastante para eu ter marcado como
  satisfeito o que esta so descrito numa tabela de ADR.
- Evidence: a secao Acceptance criteria da spec 001 usa A-NNN sem marcador; o
  formato bate com o template novo em docs/PROTOCOL.md.
- Criteria: A-008
- Decisions: converter tambem serviu de teste do template. A ambiguidade do
  A-003 so apareceu quando precisei decidir se ele estava satisfeito — que e
  exatamente o efeito pretendido pela mudanca de B-005.

## 2026-09-07 - T-002 - Evidencia reconciliada para a spec da UI
- Backlog: B-007
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: atribuida evidencia retroativa aos criterios da spec 20260907-001 que
  o trabalho ja concluido de fato satisfez, por registro novo. Nenhum registro
  passado foi editado: o CHANGELOG e append-only.
- Evidence: da spec 20260907-001 — A-001, A-002 e A-006 verificados por comando
  em clone limpo durante B-002; A-007 pelas tres ADRs no indice; A-008 pela
  linha unica no roteador. Ficam SEM evidencia A-003, A-004 e A-005, todos
  dependentes de B-004.
- Criteria: A-008
- Decisions: A-003 foi examinado e considerado NAO satisfeito. Os sete fixtures
  existem na ADR-0003 como dois JSON completos mais uma tabela descrevendo os
  outros cinco; como arquivos, nao existem. Era tentador contar como pronto, e
  a regra nova e justamente o que obrigou o exame.

## 2026-09-07 - T-001 - Qualificacao de criterio entre specs
- Backlog: B-008
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: docs/PROTOCOL.md agora resolve `A-NNN` por contexto — ID sem prefixo
  pertence a spec do proprio `Spec:` do registro; criterio de outra spec usa
  `YYYYMMDD-NNN/A-NNN`. relay-session atualizado.
- Evidence: secao Acceptance criteria do protocolo tem a regra; skill cita o
  formato qualificado. Todas as cinco skills abaixo de 40 linhas.
- Criteria: A-009
- Decisions: nenhuma.

## 2026-09-07 - T-002 - Registros de B-007 corrigidos por acrescimo
- Backlog: B-008
- Spec: .specs/20260907-002-evidencia-nomeia-criterio.md
- Result: os dois registros de B-007 diziam `Criteria: A-008` sob
  `Spec: ...002` referindo-se na verdade a criterios da spec 001. Correcao:
  aqueles IDs devem ser lidos como `20260907-001/A-008` — CHANGELOG e
  append-only, entao a leitura correta fica registrada aqui, nao editada la.
- Evidence: com a qualificacao, a spec 001 (4 entradas, B-004 pendente) segue
  sem gatilhar `criteria-without-evidence`; a spec 002 (todas [x]) tem A-001 a
  A-009 todos nomeados, a maioria sem qualificador porque pertencem a ela mesma.
- Criteria: 20260907-001/A-008
- Decisions: defeito nasceu de eu mesmo escrever `Criteria: A-008` sem
  qualificar enquanto documentava trabalho de outra spec — o proprio uso da
  regra nova expos o buraco nela.

## 2026-09-07 - T-001 - Sete fixtures materializados em disco
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: os sete fixtures normativos da ADR-0003 passaram a existir como
  arquivos sob `app/fixtures/` — `idle.json`, `backlog.json`, `ready.json`,
  `in_progress.json`, `blocked.json`, `done.json` e `inconsistent.json`.
  Os blocos `in_progress` e `inconsistent` são os dois JSON normativos da ADR;
  os outros cinco foram derivados da tabela de fixtures e do tipo `UiPayload`.
- Evidence: os sete arquivos parseiam com `JSON.parse`; cada um segue o
  `UiPayload` — `ok` carrega `status`, `handoff`, `todo`, `backlog`,
  `completed` e `total`; `inconsistent` não carrega `status`, `handoff`, `todo`
  nem `backlog`, e `violations` não é vazio (decisão 2 da ADR-0003 valendo).
- Criteria: A-003
- Decisions: o `done` é a forma de conclusão — backlog todo `[x]`, TODO e
  handoff vazios — e cai na tela Escolher com estado vazio, já que o design
  system não lhe atribui tela própria.

## 2026-09-07 - T-002 - tokens.css e app.css derivados do design system
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: `app/relay-ui/src/styles/tokens.css` com todos os tokens de cor
  (seção 2), espaçamento e raio (seção 5) e tipografia (seção 4) do design
  system; `app.css` global com as classes por componente (`status-pill`,
  `handoff-card`, `header`, `checklist`, `repair`, `empty-state`) e estados em
  sufixo, consumindo apenas `var(--token)`.
- Evidence: `grep` por `#hex` e `rgba(` em `app.css` retorna vazio — nenhum
  literal fora de `tokens.css`. O mapeamento status→tom (seção 3) vive nas
  classes de modificador e referencia os tokens sem cor literal.
- Criteria: A-005
- Decisions: tipografia também foi tokenizada (`--size-*`, `--weight-*`,
  `--font-*`), e não só cor/espaçamento/raio, para que nenhum valor literal
  precise morar em `app.css`.

## 2026-09-07 - T-003 - Scaffold da relay-ui
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: `app/relay-ui/` com `package.json`, `vite.config.ts` (alias `@` e
  `@fixtures`), `tsconfig.json`, `index.html` (fontes do design system),
  `src/main.ts`, `src/env.d.ts`, `src/types.ts` (tipos da ADR-0003) e
  `src/fixtures.ts` carregando os sete JSON. `main.ts` importa `tokens.css` e
  `app.css`.
- Evidence: `npm install` concluído na raiz do workspace `app/`; `npm run
  typecheck` (vue-tsc --noEmit) passa limpo. `node_modules/` e `dist/` já
  cobertos pelo `.gitignore`.
- Criteria: A-004
- Decisions: nenhum router nem biblioteca de estado — a ADR-0005 deixou ambos
  indecisos de propósito. O alternador de fixture usa um `ref` simples.

## 2026-09-07 - T-004 - Tela principal renderiza os sete fixtures
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: componentes `Header`, `StatusPill`, `HandoffCard`, `ChecklistList`,
  `EmptyState`, `RepairScreen` e `MainScreen`. `App.vue` troca o `UiPayload`
  inteiro por um alternador de fixture; o mesmo `MainScreen` renderiza os sete
  estados — Retomar (`in_progress`, `blocked`), Escolher (`ready`, `backlog`,
  `idle`, `done`) e Reparar (`inconsistent`).
- Evidence: `npm run build` (vue-tsc + vite build) passa limpo; dev server
  serviu `index.html`, `main.ts` e resolveu os sete JSON de `app/fixtures/`
  via alias (HTTP 200). Nenhum componente calcula `available`, status ou
  contagem — tudo chega pronto no `UiPayload`; tempo relativo é formatado na
  view via `formatRelative`.
- Criteria: A-004
- Decisions: o mapeamento status→tela mora no `MainScreen` como apresentação
  (seção 5 do design system), não como lógica de protocolo — a ADR-0003 decisão
  6 tira tom/tela do contrato, e a view devolve esse mapeamento na hora de
  desenhar.

## 2026-09-07 - T-005 - Verificação dos critérios A-003, A-004 e A-005
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: os três critérios restantes da spec 20260907-001 verificados por
  comando. A-003: sete arquivos de fixture em `app/fixtures/`. A-004: build e
  typecheck passam e o dev server resolve os sete JSON via alias. A-005:
  nenhum literal de cor, tamanho ou raio fora do `tokens.css` (os tamanhos
  `1px`, `32px`, `36px`, `1280px` e `50%` foram tokenizados em
  `--border-width`, `--control-height*`, `--container-width` e
  `--radius-round`).
- Evidence: `ls app/fixtures/*.json | wc -l` = 7; `grep` de `#hex`/`rgba(`/
  `px`/`%` em `app/relay-ui/src` e `index.html` retorna vazio; `grep` de
  `fs.`/`writeFile` em `app/` retorna vazio (A-006 re-confirmado);
  `vue-tsc --noEmit` e `vite build` passam limpos.
- Criteria: A-003, A-004, A-005
- Decisions: tamanhos entram na regra de A-005 — a seção 8 do design system
  proíbe "tamanhos literais", não só cor/espaçamento/raio, então largura do
  container, alturas de controle e largura de borda também viraram token.

## 2026-09-07 - T-006 - Critérios remanescentes da spec 001 nomeados
- Backlog: B-004
- Spec: .specs/20260907-001-ui-primeiro-marco-visual.md
- Result: os critérios A-001, A-002, A-006 e A-007 da spec 001 — satisfeitos
  pelo trabalho de B-002 mas nunca nomeados num campo `Criteria` — ganham o
  registro que a transição 5 exige para fechar a spec.
- Evidence: A-001 (instalação sem `npm install`) e A-002 (`rm -rf app/`
  restaura o estado) foram verificados por comando num clone limpo durante
  B-002 (changelog B-002/T-002); A-006 (nenhum arquivo de `app/` escreve nos
  registros) confirmado por `grep` sem acesso a `fs`; A-007 pelas três ADRs
  (0003, 0004, 0005) no índice.
- Criteria: A-001, A-002, A-006, A-007
- Decisions: a evidência existia desde B-002; o B-007/T-002 pretendia
  nomeá-la retroativamente mas gravou apenas `A-008`. Faltava só o campo
  `Criteria`, não o trabalho.

## 2026-09-07 - T-001 - Proveniencia propria em Criteria
- Backlog: B-009
- Spec: .specs/20260907-003-criterio-autoevidenciado.md
- Result: docs/PROTOCOL.md ganhou a regra de que um registro so nomeia em
  Criteria o que o proprio Result/Evidence demonstra, nunca em antecipacao.
  relay-session espelha a regra na mesma frase que ja trata Criteria.
- Evidence: nenhuma linha de prosa larga nova introduzida; nenhuma skill
  excede as demais em linhas apos a mudanca (session: 40, as outras: 39/39/39/29).
- Criteria: A-001, A-002, A-003
- Decisions: regra de disciplina, sem verificacao mecanica — o proprio scope
  descartou isso, porque nada no protocolo consegue julgar prosa contra
  significado. Mesma familia da regra ja existente sobre Criteria: none.

## 2026-09-07 - T-001 - Pacote relay-core e types do contrato
- Backlog: B-010
- Spec: .specs/20260907-004-relay-core.md
- Result: criado `app/relay-core/` com `package.json` (type module, scripts
  `test`/`typecheck`), `tsconfig.json` (NodeNext estrito, sem emit) e
  `src/types.ts` com os tipos da ADR-0003: `RelayFiles`, `RelayState`
  (ok/inconsistent), `ChecklistEntry`, `Handoff`, `Violation`, `Environment`
  e `UiPayload`.
- Evidence: `tsc --noEmit -p relay-core/tsconfig.json` limpo; workspace
  `relay-*` reconhece o pacote sem tocar na raiz do repositorio.
- Criteria: A-005
- Decisions: Node >=24 com type stripping nativo dispensa passo de build no
  pacote; testes usam `node:test` — zero dependencia de framework de teste.
  Os types passam a morar no core; a relay-ui so migra na spec 007.

## 2026-09-07 - T-002 - Parser por linha dos cinco registros
- Backlog: B-010
- Spec: .specs/20260907-004-relay-core.md
- Result: `parse.ts` le por linha: checklist (marcador, id, texto, `spec`,
  `needs`), header do TODO (`Active task`), handoff (campos chave-valor mais
  secoes Objective/Next step/Context), changelog (registros com
  Backlog/Spec/Criteria) e criterios `A-NNN` de uma spec.
- Evidence: exercitado pelos testes de fixture (cada um dos cinco registros
  e parseado); nenhum import de biblioteca de AST de Markdown.
- Criteria: none
- Decisions: a gramatica e simples e inteiramente especificada no
  `docs/PROTOCOL.md`; regex por linha, sem parser generico (decisao da spec).

## 2026-09-07 - T-003 - deriveState: status, available, contador e handoff
- Backlog: B-010
- Spec: .specs/20260907-004-relay-core.md
- Result: `derive.ts` deriva os seis status mais `inconsistent`;
  `available` = marker ` ` e todo `needs` em `[x]`; `completed`/`total`
  contam apenas as subtarefas do TODO ativo; o handoff publico nao carrega
  `status` (ADR-0003 decisao 5).
- Evidence: os sete fixtures passam como golden files; a disponibilidade dos
  fixtures `backlog`/`ready` confere com a secao `## Dependencies`.
- Criteria: A-003
- Decisions: ordem de derivacao — handoff (`in_progress`/`blocked`) primeiro;
  depois TODO (`ready`, `done` ou `blocked` quando nada esta disponivel,
  conforme a secao Dependencies); depois backlog (`backlog`/`done`); senao
  `idle`.

## 2026-09-07 - T-004 - As 13 verificacoes de integridade
- Backlog: B-010
- Spec: .specs/20260907-004-relay-core.md
- Result: `integrity.ts` implementa as 13 verificacoes do protocolo, cada uma
  produzindo uma `Violation` com o `check` estavel exato da tabela da
  ADR-0003, na mesma ordem.
- Evidence: 13 testes individuais, um por `check`, cada um disparando so a
  violacao esperada; entrada valida nao produz violacao nenhuma.
- Criteria: A-002
- Decisions: verificacoes rodam antes de derivar status — qualquer violacao
  vira `inconsistent`, nunca um status de trabalho (ADR-0003 decisao 2).

## 2026-09-07 - T-005 - Golden files e pureza do pacote
- Backlog: B-010
- Spec: .specs/20260907-004-relay-core.md
- Result: testes de golden file comparam `deriveState` byte a byte contra os
  sete fixtures de `app/fixtures/`; teste de pureza verifica que `src/` nao
  importa `node:*` nem referencia `relay-host`.
- Evidence: `node --test` 23/23 passando; `grep node: relay-core/src/` e
  `grep relay-host relay-core/src/` vazios.
- Criteria: A-001, A-004
- Decisions: fixtures como teste ativo (o gap que a revisao de B-004 achou);
  o teste de pureza e o guardiao permanente da Conformidade 1 da ADR-0003.

## 2026-09-07 - T-001 - ADR-0006 do contrato HTTP/WS
- Backlog: B-011
- Spec: .specs/20260907-005-relay-host.md
- Result: `docs/adr/0006-contrato-http-ws-do-relay-host.md` escrita antes de
  qualquer codigo: bind em 127.0.0.1 em porta efemera, token aleatorio por
  execucao entregue no HTML inicial (nunca em URL/query; no WS, via
  subprotocol), same-origin em toda rota de API com `GET /` como unico
  bootstrap, rotas de conteudo bruto para a segunda visao, rota de lancamento
  reservada (spec 008) e ausente sob `--no-exec`, e watcher de diretorio
  inteiro empurrando `UiPayload` via WS.
- Evidence: ADR com sete conformidades verificaveis; indice do AGENTS.md
  atualizado; `Sec-Fetch-Site`/token/teste 404 documentados como contratos.
- Criteria: A-001
- Decisions: token no WS via subprotocol em vez de query string, porque a
  decisao da spec "nunca em URL/query" nao cabe no handshake de browser.

## 2026-09-07 - T-002 - Pacote relay-host consumindo relay-core
- Backlog: B-011
- Spec: .specs/20260907-005-relay-host.md
- Result: `app/relay-host/` criado (package.json, tsconfig NodeNext), com
  `relay-core` como dependencia de workspace (exports para `src/index.ts`) e
  `ws` como unica dependencia de runtime.
- Evidence: `tsc --noEmit` limpo; `npm install` ligou o workspace; `ws` e
  `@types/ws` registrados no lockfile.
- Criteria: A-007
- Decisions: relay-core consome Node 24 type stripping nativo — exports para
  `.ts`, sem passo de build entre pacotes do workspace.

## 2026-09-07 - T-003 - Leitor do workspace, state e deteccao de harness
- Backlog: B-011
- Spec: .specs/20260907-005-relay-host.md
- Result: `reader.ts` monta `RelayFiles` dos cinco registros e specs;
  `state.ts` importa `deriveState` de `relay-core` e monta o `UiPayload`;
  `harness.ts` detecta Claude Code, Codex e OpenCode (versao via `--version`,
  estado instalado/nao autenticado/ausente por presenca de artefatos de
  config no home).
- Evidence: smoke test de ponta a ponta sobre este repositorio derivou o
  estado real; `grep` confirma que nenhum parse de protocolo existe no host.
- Criteria: A-006
- Decisions: deteccao de autenticacao e heuristica (presenca de arquivos de
  config no home); refinada quando os tres CLIs mudarem.

## 2026-09-07 - T-004 - HttpServer em 127.0.0.1 com token, same-origin e 404 da rota de lancamento
- Backlog: B-011
- Spec: .specs/20260907-005-relay-host.md
- Result: `server.ts` sobe HTTP+WS em `127.0.0.1` em porta efemera; toda rota
  `/api/*` exige `Sec-Fetch-Site: same-origin` + `X-Relay-Token` (403 se
  faltar); `GET /` e o bootstrap; rotas `state`, `specs`, `specs/:id`,
  `changelog` e `harnesses`; rota de lancamento nao existe (autenticada devolve
  404, nao 403), inclusive sob `--no-exec`.
- Evidence: testes de integracao: bind 127.0.0.1, 403 sem token/origem, 404 de
  `/api/launch` com e sem `--no-exec`.
- Criteria: A-002, A-003, A-004
- Decisions: `server.closeAllConnections()` no close para nao prender o
  processo com conexoes keep-alive do fetch.

## 2026-09-07 - T-005 - Watcher de diretorios e push via WebSocket
- Backlog: B-011
- Spec: .specs/20260907-005-relay-host.md
- Result: `watcher.ts` observa `.orchestration/` e `.specs/` inteiros e chama
  `broadcast()`; o WebSocket entrega o `UiPayload` na conexao e a cada mudanca,
  sem recarregar a pagina.
- Evidence: teste de integracao escreve em `.orchestration/TODO.md` e recebe
  pela WS o estado novo (activeBacklogId e todo atualizados).
- Criteria: A-005
- Decisions: re-derivar tudo a cada toque e barato (registros pequenos);
  debounce de 40ms para coalescer eventos do `fs.watch`.

## 2026-09-07 - T-001 - Host serve a UI construida com as metas de bootstrap
- Backlog: B-013
- Spec: .specs/20260907-007-dado-real-e-segunda-visao.md
- Result: `server.ts` passou a servir `app/relay-ui/dist/index.html` no `GET /`
  com as metas `relay-token`, `relay-workspace` e `relay-exec-enabled` injetadas
  no `<head>`, e os assets estáticos de `/assets/*` com MIME por extensão; o
  fallback para o `bootstrapHtml` antigo permanece quando a dist não existe.
- Evidence: smoke test de ponta a ponta — host rodando serve a UI construída
  com as três metas no HTML e o asset JS responde `200 text/javascript`. Teste
  de integração novo em `server.test.ts` (metas presentes + cada asset
  referenciado responde 200). Testes do host seguem 9/9.
- Criteria: none
- Decisions: servir a dist decidido aqui (ADR-0006 deixou a forma de servir o
  HTML como assunto da spec 007); metas no HTML preservam o bootstrap do token
  sem rota de API extra. O componente `PreflightModal` mínimo nasce junto para
  a porta "+ nova spec"; o modal completo com argv e launch é a spec 008.

## 2026-09-07 - T-002 - Cliente WebSocket com reconexão e backoff
- Backlog: B-013
- Spec: .specs/20260907-007-dado-real-e-segunda-visao.md
- Result: `src/lib/relay-client.ts` conecta ao `ws://host/ws` com subprotocol
  `relay.<token>`, expõe `payload`/`connected` reativos e reconecta com backoff
  exponencial (500ms → 10s), relendo o token do bootstrap a cada tentativa
  (token é por execução do host). `App.vue` usa o cliente e renderiza o
  `UiPayload` real; sem host (modo fixture), não conecta.
- Evidence: `npm run build` e `vue-tsc` limpos; a lógica de refresh de token no
  reconnect cobre "matar e religar o host" sem recarregar a página. Cliente WS
  com mesma origem + subprotocol validado pelo teste de integração do host.
- Criteria: A-001, A-002
- Decisions: token lido de novo a cada tentativa porque o host regenera o token
  por execução; reconectar com token velho falharia para sempre após restart.

## 2026-09-07 - T-003 - Segunda visão: três colunas e PreflightModal
- Backlog: B-013
- Spec: .specs/20260907-007-dado-real-e-segunda-visao.md
- Result: `WorkScreen.vue` com três colunas simultâneas — Specs (lista de
  `GET /api/specs`, com contagem de tarefas), Backlog (filtrado do próprio
  `RelayState.backlog` pelo campo `spec` da entrada selecionada, sem rota
  nova) e Changelog (`GET /api/changelog`). Porta "+ nova spec" no cabeçalho da
  coluna Specs abrindo o `PreflightModal` mínimo; nenhum dos três componentes
  emite requisição de escrita.
- Evidence: a coluna do meio reusa `payload.state.backlog` filtrado por
  `spec === '.specs/<id>'`; a coluna de specs e a de changelog usam só `GET`
  (`grep fetch` em WorkScreen retorna apenas `/api/specs` e `/api/changelog`).
  `WorkScreen` re-busca specs e changelog a cada mudança de `payload` (watch),
  então uma escrita em disco que o watcher empurra pela WS re-renderiza as duas
  visões sem recarregar. Typecheck e build limpos.
- Criteria: A-004, A-006, A-007
- Decisions: backlog reaproveitado e não duplicado (decisão da spec); o
  PreflightModal aqui é o shell da porta — argv, launch e dados reais são a
  spec 008, e a coluna nunca escreve por conta própria (ADR-0001 ponto 5).

## 2026-09-07 - T-004 - Abas Agora/Trabalho no Header e App
- Backlog: B-013
- Spec: .specs/20260907-007-dado-real-e-segunda-visao.md
- Result: `Header.vue` passou a receber `view` e emitir `update:view`, com as
  abas Agora/Trabalho alternando entre `MainScreen` e `WorkScreen` no `App.vue`
  (ref local, sem roteamento). O `MainScreen` deixou de renderizar Header e
  HarnessSelector, que subiram para o `App` — onde o seletor e o alternador de
  abas ficam disponíveis nas duas visões.
- Evidence: alternância é um `ref` em `App.vue`; build e typecheck limpos. A
  aba Trabalho mostra as três colunas do `WorkScreen` (T-003) e selecionar uma
  spec filtra a coluna do meio.
- Criteria: A-003
- Decisions: roteamento client-side não entra (ADR-0005 deixou em aberto); a
  segunda visão é alcançada por abas, como o design system descreve.

## 2026-09-07 - T-005 - Seletor de fixture restrito a rota de desenvolvimento
- Backlog: B-013
- Spec: .specs/20260907-007-dado-real-e-segunda-visao.md
- Result: o alternador de fixtures só aparece no modo fixture — quando não há
  `relay-token` no HTML (dev server) ou quando a URL traz `?fixtures`. Servido
  pelo host, o seletor não aparece por padrão; continua existindo para trabalho
  de componente sem `relay-host` rodando.
- Evidence: `App.vue` computa `fixtureMode` de `!hostMode || ?fixtures`; com o
  host servindo a UI (smoke test), o HTML não traz o alternador. Build limpo.
- Criteria: A-005
- Decisions: `?fixtures` é a rota de desenvolvimento explícita; em dev server
  sem host, o fixture é o padrão natural.

## 2026-09-07 - T-001 - Infra de harness: tipos, fixture, tons e store de selecao/consentimento
- Backlog: B-012
- Spec: .specs/20260907-006-refinamento-visual-prototipo.md
- Result: `app/relay-ui/src/lib/harness.ts` com os tipos `Harness` e
  `ConsentLevel`, a fixture de deteccao (claude-code, codex, opencode), os
  tons de identidade (`--purple` para codex, `--orange` para claude-code,
  neutro para os demais), helpers (`harnessById`, `harnessInitials`,
  `harnessTone`) e o store reativo de selecao + consentimento com persistencia
  por escopo: `none` (nada grava), `session` (Map em memoria), `local`
  (localStorage chaveado por workspace).
- Evidence: `npm run typecheck` limpo; o store so toca `localStorage`, nunca
  `fs` — `grep fs. app/relay-ui/src/lib/harness.ts` vazio; nenhum arquivo novo
  em disco alem do proprio modulo. `--orange` faltava em `tokens.css` e foi
  alinhado ao README do design system (que ja o declara), sem decisao nova.
- Criteria: A-005
- Decisions: consentimento e estado do navegador, nao do protocolo; nada de
  escrita em arquivo, e nenhum nivel remove clique de confirmacao de preflight
  (preflight e da spec 008).

## 2026-09-07 - T-002 - HandoffCard refinado
- Backlog: B-012
- Spec: .specs/20260907-006-refinamento-visual-prototipo.md
- Result: `HandoffCard` passou a renderizar avatar com as iniciais do harness
  (ton de identidade ou neutro), timestamp absoluto junto do relativo e dos IDs
  numa linha mono, rotulo `OBJETIVO · B / T`, corpo em duas colunas (Proximo
  passo / Contexto deixado), caminho da spec no rodape e botao primario
  nomeando o harness (`▶ Retomar T-002 no {harness}`) mais o secundario
  "Trocar harness" abrindo o seletor.
- Evidence: fixture `in_progress` renderiza avatar CC, "B-001 / T-002 ·
  2026-09-07 06:49", colunas e botao "Retomar T-002 no Claude Code"; fixture
  `blocked` (escrito por opencode, ausente) cai no primeiro harness disponivel,
  nao num ausente. `npm run build` limpo.
- Criteria: A-001
- Decisions: nenhuma decisao visual nova — layout e rotulos vieram do
  `docs/design-system/README.md` (secao 6, HandoffCard).

## 2026-09-07 - T-003 - ChecklistList com rotulos textuais e contador
- Backlog: B-012
- Spec: .specs/20260907-006-refinamento-visual-prototipo.md
- Result: `ChecklistList` ganhou rotulo textual por marcador (Feito / Em
  execucao / Bloqueado / Pendente), destaque de fundo na linha `[•]` ativa e o
  contador verdadeiro "N de M concluidas" no cabecalho da lista, nunca
  percentual.
- Evidence: fixture `in_progress` mostra "1 de 3 concluidas" e a linha T-002
  `[•]` com fundo `--green-soft`; fixture `blocked` mostra T-003 `[!]` como
  Bloqueado. `npm run build` limpo.
- Criteria: A-002
- Decisions: contador e cabecalho da lista (nao dentro do card), como manda o
  design system; sem percentual nem posicao.

## 2026-09-07 - T-005 - Seletor de harness e consentimento
- Backlog: B-012
- Spec: .specs/20260907-006-refinamento-visual-prototipo.md
- Result: componente `HarnessSelector` (overlay standalone) lista os harnesses
  da fixture com nome, versao e estado, cada um com ton de identidade de
  `--purple`/`--orange` (ausente fica neutro e desabilitado); tres niveis de
  consentimento nomeados (So esta execucao / Enquanto a app estiver aberta /
  Sempre neste workspace); rodape que nomeia o escopo do nivel selecionado e
  muda junto com ele. Abre pelo selo do Header e pelo botao "Trocar harness".
- Evidence: `npm run build` limpo; selecionar "Sempre neste workspace" grava em
  `localStorage` (chave `relay.harness:<workspace>`); com "So esta execucao"
  marcado o rodape diz "Vale so para esta execucao; nada e gravado" e nenhum
  outro texto da tela promete gravacao. Nenhum componente pinta harness com
  `--blue`/`--green`/`--amber`.
- Criteria: A-004, A-006, A-007
- Decisions: tons de identidade no seletor e no selo vindo do mesmo
  `harnessTone()`; harness desabilitado (ausente) nao recebe tom; o seletor
  aqui e isolado — o preflight inline (spec 008) e que recebera este componente
  embutido.

## 2026-09-07 - T-004 - Header com abas e selo de harness
- Backlog: B-012
- Spec: .specs/20260907-006-refinamento-visual-prototipo.md
- Result: `Header` ganhou nome + caminho do workspace, abas Agora/Trabalho e um
  selo compacto do harness ativo (avatar com iniciais no tom de identidade,
  nome e escopo do consentimento) que abre o seletor ao clicar; StatusPill a
  direita. `MainScreen` renderiza o `HarnessSelector` e inicia o store com o
  workspace do payload.
- Evidence: `npm run typecheck` e `npm run build` limpos; o selo usa os mesmos
  helpers de ton do seletor, entao codex/claude-code aparecem em
  purple/orange e nenhum harness em blue/green/amber; a aba Trabalho fica
  desabilitada (a segunda visao de tres colunas e a spec 007, nao esta no
  escopo de B-012).
- Criteria: A-003
- Decisions: aba Trabalho presente mas desabilitada ate a spec 007 entregar a
  segunda visao; selo compacto conforme o design system (secao 6, Header).
## 2026-09-11 - T-003 - Layout responsivo e safe areas
- Backlog: B-027
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: regras de viewport estreita foram registradas no design system e aplicadas ao CSS: breakpoint de 760px, safe-area insets, Header/Handoff/Trabalho/modais empilháveis, rolagem interna e modo de execução em duas linhas.
- Evidence: `npm test --workspace relay-ui` 14/14; `vue-tsc --noEmit` e `vite build` limpos; auditoria Python confirmou as regras documentadas/presentes; `git diff --check` limpo. A inspeção visual automatizada não ficou disponível; a conclusão segue a autorização do usuário para prosseguir após o bloqueio do navegador.
- Criteria: A-011
- Decisions: breakpoint 760px como regra de mídia; safe areas usam `env(...)` com o espaçamento existente, sem novos tokens ou dependências.

## 2026-09-11 - T-004 - Acoes Retomar/Comecar por linha na aba Trabalho e ID correto
- Backlog: B-027
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: o cartao de backlog da aba Trabalho ganhou o botao de acao ao lado de "Tarefas" descrito na secao 6 do design system — "Retomar" para `[•]` e "Comecar" para `[ ]` disponivel, so com execucao habilitada — chamando `relay-session` com o ID daquele cartao. Na tela Escolher (aba Agora), `comecar()` deixou de enviar sempre `activeBacklogId` e passou a enviar sempre o `id` da linha clicada, entao TODOs disponiveis distintos abrem preflights com IDs distintos.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14; `npm test --workspace relay-core` 30/30; `runIntegrityChecks` contra os cinco registros reais deste repositorio retorna `[]`. A verificacao visual no browser nao ficou disponivel (extensao Chrome sem resposta); a conclusao segue leitura de codigo linha a linha contra a secao 6 do design system.
- Criteria: A-012
- Decisions: reaproveitado o mesmo par titulo/skill/intent de `HandoffCard.resume()` e `MainScreen.comecar()` para o novo botao, em vez de uma terceira variante de preflight.

## 2026-09-11 - T-001 - Linha da lista Escolher nao quebra ID nem botao
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `.choose__item` (MainScreen.vue, tela Escolher da aba Agora) ganhou `flex-wrap: wrap`; `.choose__id` e `.choose__action` ganharam `flex-shrink: 0` e `white-space: nowrap`, entao nunca mais quebram palavra (`B-\n020`, botao em duas linhas); `.choose__text` e `.choose__spec` ganharam `min-width: 0` para poderem encolher/quebrar normalmente sem espremer os vizinhos. Caminho da spec continua podendo quebrar em mais de uma linha, que o README ja autoriza para caminhos longos em cartoes.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Reproduzido antes da correcao por inspecao do CSS (flex-shrink padrao sem protecao) contra a captura de tela enviada pelo usuario mostrando "B-\n020" e o rotulo do botao partido; verificacao visual automatizada no browser nao ficou disponivel (extensao Chrome sem resposta).
- Criteria: none
- Decisions: mantido o wrap do caminho da spec (comportamento ja autorizado pelo README para cartoes) em vez de forcar `nowrap` ali, que so empurraria o overflow para fora do cartao.

## 2026-09-11 - T-003 - Lista de subtarefas nao colapsa invisivel quando o painel encolhe
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `.agora__subtasks .checklist` (app.css) trocou `min-height: 0` por `min-height: 4.5rem`, entao a lista de subtarefas sempre mostra ao menos uma linha antes de rolar internamente. Antes, com um HandoffCard alto ocupando quase toda a `.agora`, o flexbox espremia a lista ate 0 e so o cabecalho "Subtarefas"/contador ficava visivel — exatamente a captura enviada pelo usuario (handoff de T-001/B-028 aberto, painel "Subtarefas" aparentando vazio/cortado logo apos o cabecalho). O `.app__body` (ja `overflow-y:auto`) agora absorve o excesso como rolagem de pagina no pior caso, em vez do painel sumir.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Verificacao visual automatizada no browser nao ficou disponivel (extensao Chrome sem resposta); a conclusao segue leitura do CSS e do fluxo de flexbox contra a captura de tela do usuario.
- Criteria: none
- Decisions: piso de altura (`min-height`) em vez de um segundo valor de altura maxima — o README (secao 5) ja pede rolagem interna com espaco flexivel restante, sem um teto fixo dedicado para a lista de subtarefas; o piso so evita o colapso total, nao muda esse comportamento.

## 2026-09-11 - T-002 - Selo de harness do Header com espacamento e responsividade proprios
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `.harness-badge` trocou `height` fixo (`--control-height-sm`, pensado para controle de uma linha) por `min-height` com padding vertical (`var(--space-1)`), porque o selo sempre carrega duas linhas de texto (nome + escopo) ao lado do avatar de 28px e o valor fixo anterior nao sobrava espaco de respiro. Nome e escopo ganharam `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` com `min-width:0` na cadeia de pais, entao um nome de harness longo trunca em vez de estourar o selo. Em viewport estreita (760px, breakpoint ja existente), o selo agora tem `order:2; flex:1 1 auto` no `@media`, indo para a mesma segunda linha do Header que FreshnessStatus/StatusPill em vez de competir por espaco na primeira linha com a marca.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Verificacao visual automatizada no browser nao ficou disponivel (extensao Chrome sem resposta); a conclusao segue leitura de CSS/markup contra a captura de tela do usuario e o breakpoint ja documentado no README (secao 5).
- Criteria: none
- Decisions: icones proprios por harness (Claude/Codex/OpenCode) ficaram fora desta subtarefa — usuario vai fornecer os SVGs/PNGs oficiais antes de qualquer implementacao, para nao reproduzir logo de memoria.

## 2026-09-11 - T-004 - Avatar de harness usa o icone real da marca em vez de iniciais
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: novo componente `HarnessIcon.vue` embute o SVG oficial de cada harness reconhecido (claude-code, codex, opencode, fornecidos pelo usuario) e um glifo generico de terminal para qualquer harness nao reconhecido; substitui `.harness-avatar` + `harnessInitials()` nos 4 pontos que usavam iniciais (Header, HandoffCard, HarnessConsentPicker, ExecutionMode). A moldura do avatar ficou neutra (`--panel-2`/`--line`) em vez de tonalizada por harness (`--purple`/`--orange`), decisao ja registrada no README antes do codigo: colorir a moldura e mostrar o icone com a cor real da marca repetiria a identidade duas vezes. OpenCode (glifo solido sem cor propria) e o fallback generico herdam `--meta` via `currentColor`. `harnessTone()` e `harnessInitials()` saem de `lib/harness.ts` por ficarem sem nenhum chamador. `antigravity.svg`, tambem enviado pelo usuario, ficou de fora: nao e harness modelado em `lib/harness.ts` e pesa 2.3MB com raster embutido via `<use xlink:href>`, inviavel para um avatar de 28px.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Verificado visualmente no browser real (chrome-devtools MCP, ja que a extensao claude-in-chrome seguiu sem responder): Header, HandoffCard e o seletor "Harness e consentimento" mostram os tres glifos corretos (Claude Code laranja, Codex branco+gradiente, OpenCode em `--meta`), sem quebra de layout.
- Criteria: none
- Decisions: mantidas as cores originais de cada marca no glifo (pedido explicito do usuario: "o icone original deles"), mesmo a do Claude Code nao batendo exatamente com o `--orange` do design system; a moldura neutra evita a colisao dupla de identidade em vez de tentar reconciliar as duas paletas.

## 2026-09-11 - T-005 - Fundo do Codex padronizado, selo do Header simplificado e objetivo do handoff limitado
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: tres ajustes de acabamento pedidos pelo usuario apos revisao visual. (1) Removido o `<path fill="#fff">` de fundo embutido no SVG do Codex (`HarnessIcon.vue`); o glifo agora fica sobre o mesmo `--panel-2` escuro que Claude Code/OpenCode/generico, sem quadrado branco proprio. (2) O selo de harness do Header perdeu o texto de escopo de consentimento ("· nao grava"): ficou so icone+nome, com `height: var(--control-height-sm)` fixo e conteudo de uma linha so, para bater a mesma altura dos selos "Atualizado"/status vizinhos - o escopo continua explicado dentro do seletor de harness, onde cada opcao ja o nomeia. (3) `.handoff-card__objective-text` ganhou `-webkit-line-clamp: 3` para o objetivo do handoff nunca inflar o card indefinidamente quando o texto for longo, do mesmo jeito que "Proximo passo"/"Contexto" ja sao limitados.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Verificado no browser real via chrome-devtools MCP: selo do Header com icone+nome numa linha, mesma altura dos pills "Atualizado"/"Em andamento"; Codex sem fundo branco; card de handoff com o objetivo em 3 linhas.
- Criteria: none
- Decisions: README (secao "Header" e "HandoffCard") atualizado antes do codigo para descrever as tres mudancas e corrigir mencoes desatualizadas a "iniciais do harness" que sobraram da decisao do T-004.

## 2026-09-11 - T-007 - Os tres selos do Header ficam com a mesma altura e o selo de harness para de esticar
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: usuario apontou com captura de tela que "Atualizado" (`FreshnessStatus`), "Pronto"/status atual (`StatusPill`) e o selo de harness continuavam com tamanhos diferentes mesmo depois do T-005. Duas causas medidas via devtools em viewport 757px: (1) `.status-pill` nunca teve `height` explicito, so padding — renderizava 28px contra os 32px (`--control-height-sm`) de `.freshness-status`/`.harness-badge`; ganhou `height: var(--control-height-sm)` e `padding: 0 var(--space-3)`, igual aos outros dois. (2) Em viewport estreita (`@media max-width:760px`), a regra `.harness-badge{flex:1 1 auto}` que eu mesmo escrevi no T-002 fazia o selo esticar e ocupar todo o espaco restante da segunda linha do Header enquanto os outros dois ficavam compactos ao lado — removido o `flex:1 1 auto`, mantido so o `order:2`; o selo agora fica do tamanho do proprio conteudo como os outros dois. `.freshness-status` tambem trocou `min-height` por `height` para a mesma garantia.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Medido no browser real (chrome-devtools MCP, viewport 757x862): os tres selos com 32px de altura; selo de harness com 137.9px de largura (do proprio conteudo) contra 459px antes da correcao.
- Criteria: none
- Decisions: corrigido a regra que eu mesmo tinha introduzido no T-002 em vez de empilhar uma nova excecao por cima; usuario pediu para so atualizar o design-system depois de confirmar visualmente a correcao, nao antes — invertido a ordem usual desta sessao (README primeiro) so para esta subtarefa.

## 2026-09-11 - T-008 - Linhas da lista de subtarefas com ritmo uniforme
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: usuario apontou com captura de tela que as linhas de `.checklist__item` (lista "Subtarefas" da aba Agora, e o mesmo componente no modal de tarefas do backlog) tinham alturas diferentes — a altura seguia o tamanho do proprio texto da subtarefa, sem piso nem teto. `.checklist__text` ganhou `-webkit-line-clamp: 2` (mesma tecnica do objetivo do HandoffCard, T-005) e `.checklist__item` ganhou `min-height: 4.25rem` (duas linhas de `--size-body` + padding do item); `align-items: center`, que ja existia, centraliza verticalmente as linhas de uma subtarefa so.
- Evidence: `vue-tsc --noEmit` e `vite build` (workspace relay-ui) limpos; `npm test --workspace relay-ui` 14/14. Medido no browser real (chrome-devtools MCP): as 8 linhas da lista (T-001 a T-008) todas com 68px de altura, textos de 1 e 2+ linhas incluidos.
- Criteria: none
- Decisions: a causa de fundo sao descricoes de TODO longas demais para uma lista compacta (as que eu mesmo escrevi para T-006/T-007); o clamp e o piso resolvem o ritmo visual, mas o habito de escrever subtarefas mais curtas continua sendo a correcao real daqui pra frente.

## 2026-09-11 - T-006 - app/README.md descreve o modo de execucao (terminal e disk-log)
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `app/TODO-BUILD.md` ja tinha sido removido antes desta sessao (documento auto-descrito como temporario, todo o backlog que ele listava — B-010 a B-016 — ja esta `[x]`), mas nenhum changelog tinha citado A-014 ainda. `app/README.md` ja descrevia core/host/websocket/build/estrutura mas so mencionava o terminal numa linha da flag `--exec`; adicionada a secao "Modo de execucao" (preflight -> terminal embutido com detach/reanexar e aviso de teclado -> painel "Gravado em disco", faixa de segundo plano) e os componentes Terminal/DiskLog/ExecutionMode/PreflightModal na lista de estrutura.
- Evidence: `vue-tsc --noEmit` e `npm test --workspace relay-ui` (14/14) limpos (mudanca e so de documentacao, sem tocar componente). Textos citados (`Rodando em segundo plano`, `Reconectar ao terminal`, `Encerrar processo`) conferidos contra `BackgroundStrip.vue`/`ExecutionMode.vue` linha a linha.
- Criteria: A-014
- Decisions: dar credito formal a uma remocao de arquivo que ja existia no working tree em vez de refazer o trabalho, apos confirmar que o conteudo do `TODO-BUILD.md` removido nao tinha mais nenhum item pendente.

## 2026-09-11 - T-009 - Auditoria final por componente (contraste, cor+rotulo, foco, alvo, hierarquia, semantica, responsividade)
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: novo `docs/design-system/audit-2026-09-11.md`, registrado no indice do README daquela pasta, cobrindo Header, Agora (Retomar e Escolher), Trabalho, PreflightModal/HarnessSelector/HarnessConsentPicker/KeyboardWarning, modo de execucao e RepairScreen contra os sete eixos de A-015. Tres lacunas reais encontradas e corrigidas no caminho: (1) a tela Escolher (MainScreen.vue) nao tinha nenhum heading — adicionado `<h2 class="panel__title">Escolher</h2>`; (2) `StatusPill` nao tinha `role=status`/`aria-live`, diferente do `FreshnessStatus` que ja seguia essa regra — adicionado; (3) o modo de execucao (ExecutionMode.vue) nao montava `<main>` nem `<h1>` porque o Header normal nao aparece ali — a raiz virou `<main>` e o nome do processo virou `<h1>` (com `margin:0` para nao quebrar o layout da barra). Lacunas maiores (terminal ilegivel por leitor de tela, disk-log sem aria-live, anel de foco de 1px) foram registradas como aceitas/pendentes de decisao propria, nao forcadas nesta subtarefa.
- Evidence: `vue-tsc --noEmit`, `npm test --workspace relay-ui` (14/14) e `npm run build --workspace relay-ui` limpos. Contraste calculado por script Python com a formula de luminancia relativa do WCAG 2.1 contra os hex de `tokens.css` (todos os pares texto/fundo auditados ficaram acima de 4.5:1, a maioria acima de 7:1). Hierarquia de headings (`h1`->`h2`->`h3`, sem pulo) e ausencia de `main`/`h1` no modo de execucao confirmadas por leitura de codigo e depois no browser real via chrome-devtools MCP (`http://127.0.0.1:4173`).
- Criteria: A-015
- Decisions: corrigir so o que era pequeno e inequivoco (heading faltando, role ausente, landmark ausente); tres lacunas maiores viraram nota registrada no proprio arquivo de auditoria em vez de virarem escopo novo desta subtarefa.

## 2026-09-16 - T-010 - Testes de UI cobrem os seis cenarios de regressao da spec
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `docs/adr/0008-testes-de-ui-com-vitest.md` registra a decisao (usuario escolheu Vitest+jsdom apos explicacao das alternativas via AskUserQuestion, Playwright e Testing Library descartados) antes de `vitest`/`@vue/test-utils`/`jsdom` entrarem como devDependency so de `app/relay-ui`. Seis testes novos em `app/relay-ui/test/*.vitest.ts` cobrem os cenarios de A-013: preview desatualizado do preflight (resposta antiga chegando depois da nova nao sobrescreve o plano), foco preso na contencao do dialogo (Tab/Shift+Tab nunca escapam, foco restaura ao originador), downgrade de consentimento (clicar em "So esta execucao" limpa o localStorage, sobrevive a reabrir a pagina), harness ausente (botao de executar desabilitado quando so ha harness `absent`), desconexao (FreshnessStatus mostra "Desatualizado" com `role=status`/`aria-live`) e selecao de ID (clicar em Comecar de tarefas distintas envia IDs distintos, nao sempre o mesmo). `node --test` ganhou um glob explicito (`test/*.test.ts`) para nao tentar rodar os arquivos `.vitest.ts` como se fossem seus.
- Evidence: `vue-tsc --noEmit`, `vite build` e `npm test --workspace relay-ui` (14 node:test + 11 vitest, 25/25) limpos. Cada um dos testes de preview-desatualizado e selecao-de-ID foi verificado revertendo manualmente a correcao correspondente no codigo e confirmando que o teste falha (o guard de `revision`/`previewToken` do PreflightModal e o `comecar(entry.id)` do MainScreen), depois restaurado — nao sao testes vazios.
- Criteria: A-013
- Decisions: `jsdom` nao calcula layout (`offsetParent` sempre `null`), o que quebraria o filtro de `focusables()` em `focus-trap.ts` silenciosamente — descoberto so ao implementar, corrigido com um shim minimo em `test/vitest-setup.ts` e a ressalva registrada na propria ADR-0008, nao escondida.

## 2026-09-16 - T-011 - Nenhuma acao de execucao usa estado desatualizado
- Backlog: B-028
- Spec: .specs/20260907-013-ui-acessibilidade-e-conformidade.md
- Result: `runIntegrityChecks` real (fora do fixture, contra os cinco registros deste repositorio) acusou `criteria-without-evidence` para A-008 na spec 013 apos o T-010 — as duas primeiras clausulas do criterio (desconexao visivel, dado antigo identificado) ja existiam e tinham evidencia; a terceira ("nenhuma acao de execucao usa estado stale") nao. README (principio 10, secoes "Tela Escolher", HandoffCard e "Cartao de backlog") registra a decisao antes do codigo. `freshness` passou a ser propagado de App.vue para MainScreen/WorkScreen (mesmo padrao ja usado para `execEnabled`) e da MainScreen para HandoffCard; todo botao que abre um lancamento — "▶ Começar" (tela Escolher e cartao de backlog), "▶ Retomar" (HandoffCard e cartao de backlog), "Iniciar entrevista" e "+ nova spec" — desabilita quando `freshness === 'stale'`, com uma nota de texto ao lado explicando o motivo (nunca cinza silencioso). O `PreflightModal` ganhou a mesma checagem no proprio `canConfirm`, lendo `client.freshness` diretamente, como ultima linha de defesa caso a conexao caia com o modal ja aberto.
- Evidence: `vue-tsc --noEmit`, `vite build` e `npm test --workspace relay-ui` (14 node:test + 14 vitest, 28/28) limpos. Teste novo (`test/stale-actions.vitest.ts`) verificado revertendo manualmente o `:disabled="stale"` da tela Escolher e confirmando que falha, depois restaurado. Verificado tambem no browser real: matei o `relay-host` com a aba aberta, o selo do Header foi para "Desatualizado" e o botao "▶ Retomar" do HandoffCard ganhou `disabled` de verdade (`opacity:0.6`, `cursor:not-allowed`, confirmado via `getComputedStyle`), com a nota "Desatualizado — aguardando reconexão" ao lado; voltou a habilitar sozinho ao religar o host.
- Criteria: A-008
- Decisions: gatilho unico por `freshness === 'stale'` (nao por `!connected`), porque "Atualizando" (transicao breve entre arquivos) mantem dado valido e nao deveria bloquear acao, distincao que ja estava no principio 10 do README antes desta subtarefa.

## 2026-09-16 - T-001 - node-pty substitui /usr/bin/script; prompt e cwd corretos no argv embutido
- Backlog: B-020
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `app/relay-host/src/pty.ts` reescrito sobre `node-pty` (`pty.spawn`), substituindo a implementacao anterior via `/usr/bin/script -q -e -F <fifo> --` (sintaxe GNU, incompativel com o binario BSD do macOS) mais um fifo lido por polling manual. `ptyAvailable()` mantido restrito a `darwin`, mesmo comportamento de antes. `app/relay-host/src/executor.ts` corrigido: `startExec` agora chama `startPtyRun` com `[...plan.args, plan.prompt]`, entao o processo embutido recebe o prompt aprovado no preflight como ultimo elemento do argv (antes ele era descartado silenciosamente). node-pty instalado so em `app/relay-host` (ja decidido em ADR-0001 ponto 2 e na spec 009, nao precisou de ADR nova). Achado durante a instalacao: o binario `spawn-helper` do prebuild extrai sem bit de execucao neste workspace (`posix_spawnp failed` em runtime, nao em install) — corrigido com um `postinstall` proprio (`scripts/fix-node-pty-permissions.mjs`) que reaplica `chmod 755` apos todo `npm install`, para nao depender de uma correcao manual que uma instalacao limpa nao teria.
- Evidence: `tsc --noEmit` e `npm test --workspace relay-host` (27/27, dois testes novos) limpos. `test/pty.test.ts` sobe uma PTY real via `/bin/sh`, envia `echo ...; exit 7`, confere a saida e o exit code 7 (A-001). `test/executor.test.ts` chama `startExec` de verdade com um plano cujo comando imprime `$PWD` e `$0`, confirmando prompt como ultimo argv e cwd correto (A-002). Os dois foram verificados revertendo a correcao correspondente e confirmando falha (o append do prompt em executor.ts), depois restaurados. Reproduzi tambem o bug original: `/usr/bin/script -q -e -F <fifo> -- /bin/echo hi` trava indefinidamente sem um leitor concorrente do fifo (matado por timeout), confirmando a fragilidade que motivou a troca.
- Criteria: A-001, A-002
- Decisions: mantido o mesmo `ptyAvailable()` restrito a macOS (nao expandido para Linux/Windows, fora do escopo desta spec); resize da PTY continua nao encadeado do cliente ao host (nenhuma chamada `term.resize()` existe) porque nenhuma rota WebSocket de resize existe hoje — gap pre-existente, nao introduzido nem escondido aqui, e fora dos criterios A-001/A-002 desta subtarefa.

## 2026-09-16 - T-001 - Wrapper externo entra no cwd aprovado, falha observavel sem terminal, scratch restrito
- Backlog: B-021
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: tres bugs do `launcher.ts` corrigidos. (1) `buildWrapperScript` nunca mudava para o `cwd` aprovado no preflight — ganhou `cd <cwd> || exit 1` como segunda linha do script, antes de qualquer coisa rodar. (2) `defaultOpenExternal` no-opava silenciosamente quando nenhum terminal conhecido (`TERMINAL_CANDIDATES`) estava instalado, e `launch` retornava sucesso mesmo assim; `openExternal` agora retorna `boolean` e `launch` lanca erro quando nenhum terminal abriu, que a rota `/api/launch` (ja com catch->400) devolve como falha real. (3) `mkdirSync`/`chmodSync` do diretorio e do script de scratch nao tinham modo restrito (herdavam o umask padrao, ~0o755); agora usam `0o700`, e o proprio wrapper roda `umask 077` antes de escrever `pid`/`exit`, entao todo artefato nascido em runtime tambem fica restrito ao dono, nao so os dois arquivos que o Node cria antecipadamente.
- Evidence: `tsc --noEmit` e `npm test --workspace relay-host` (29/29, quatro testes novos/atualizados) limpos. Os tres testes novos verificados revertendo cada correcao individualmente e confirmando falha (`cd` ausente, `openExternal` sem checagem, `mkdirSync` sem `mode`), depois restaurados.
- Criteria: A-003
- Decisions: `launch` ainda cria o diretorio de scratch e escreve o script mesmo quando nenhum terminal sera aberto, antes de lancar o erro — o artefato serve de evidencia do que teria rodado, e o `mode:0o700` ja aplicado nele nao piora a exposicao por existir.

## 2026-09-16 - T-001 - GET /api/run/<id> e /disk: parsing de rota corrigido, run desconhecida devolve 404
- Backlog: B-022
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `server.ts` calculava `route = parts[1]` (so `'run'`, sem o runId), entao `route.startsWith('run/')` nunca era verdadeiro — o bloco inteiro das duas rotas GET era codigo morto, e toda requisicao caia no 404 final por acaso, nao pela checagem de run desconhecida. Mesmo corrigindo esse casamento, o bloco original checava `parts[2] === 'disk'` quando `/disk` na verdade fica em `parts[3]` (`/api/run/<id>/disk` → `['api','run','<id>','disk']`) — segundo bug empilhado. Reescrito para checar `route === 'run' && parts.length >= 3`, extrair o runId de `parts[2]` e distinguir `/disk` por `parts[3] === 'disk'`. `deps.runDiskEntries` mudou de devolver `[]` para devolver `null` numa run desconhecida (mesmo padrao que `runInfo` ja usava), e as duas rotas agora devolvem 404 nesse caso em vez de 200 com corpo vazio/nulo (A-005 pede 404 explicitamente pras duas).
- Evidence: `tsc --noEmit` e `npm test --workspace relay-host` (30/30, um teste novo) limpos. Teste novo cobre run existente (info 200 e disk 200 com o corpo certo) e run desconhecida (404 nos dois), verificado falhando com o parsing antigo restaurado (200 em vez de 404) e depois corrigido de novo.
- Criteria: A-005
- Decisions: nenhuma — correcao direta de dois bugs de parsing/contrato ja descritos no Problem da spec, sem ambiguidade de design a registrar.

## 2026-09-16 - T-002 - Disk tracker reconhece o estado vazio canonico de HANDOFF.md/TODO.md como LIMPO
- Backlog: B-022
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `disk.ts` classificava uma escrita como `cleared` só quando o novo conteudo era string vazia (`after === ''`), mas o estado vazio canonico de `HANDOFF.md`/`TODO.md` e texto ("No active handoff."/"No active task.", nunca string vazia) — o disk-log nunca mostrava LIMPO de verdade pra esses dois registros, so UPDATED. Adicionado `CANONICAL_EMPTY` (regex por registro) e `isCleared(name, content)`; `BACKLOG.md`/`CHANGELOG.md` continuam usando string vazia (nao tem template de vazio canonico).
- Evidence: `tsc --noEmit` e `npm test --workspace relay-host` (32/32, dois testes novos em `test/disk.test.ts`, arquivo novo pro modulo) limpos. Verificado revertendo pra `after === ''` e confirmando falha (classificava como `updated`), depois restaurado.
- Criteria: A-007
- Decisions: nenhuma — correcao direta do criterio canonico ja nomeado no Problem/Scope da spec.

## 2026-09-16 - T-003 - Terminar uma run so confirma conclusao apos saida real do host
- Backlog: B-022
- Spec: .specs/20260907-012-execucao-segura-e-ciclo-de-vida.md
- Result: `terminateRun()` (relay-ui) marcava `store.status = 'exited'` logo apos disparar o POST de termino, antes de qualquer confirmacao — removido; o status real ja e setado por `socket.onExit()` (`wire()`) quando a saida de verdade chega pelo WebSocket de terminal. `POST /api/run/<id>` com `action=terminate` (relay-host) sempre devolvia 200 mesmo pra um `runId` inexistente; agora checa `deps.runInfo(runId)` e devolve 404 antes de chamar `runTerminate`.
- Evidence: `tsc --noEmit`/`vue-tsc --noEmit` e `npm test` limpos nos dois workspaces (relay-host 33/33 com um teste novo de rota; relay-ui 14 node:test + 15 vitest com um teste novo em `test/execution-terminate.vitest.ts`). Os dois comportamentos verificados revertendo a correcao e confirmando falha (status virava `exited` sem exit real; run desconhecida devolvia 200), depois restaurados.
- Criteria: A-008
- Decisions: nenhuma — os dois bugs (otimismo no cliente, ausencia de checagem de existencia no servidor) ja estavam nomeados explicitamente no Problem da spec.

## 2026-10-02 - T-001 - ADR-0009 registra Rust, segundo core com suite de conformidade, distribuicao por binario e o canal de perguntas como evolucao
- Backlog: B-033
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criada `docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md` (Accepted, formato Status/Contexto/Decisao/Consequencias/Compliance/Notes) com sete decisoes: binario Rust read-only e passivo; Rust contra Go e TS compilado; segundo core contido por suite de conformidade, preservando a decisao 5 da ADR-0001; distribuicao por Release, reconhecendo a quarentena do macOS e o SmartScreen sem assinatura na v1; `relay-tui` nao e um Relay CLI; aparencia remetida ao design system sem copiar valores; canal de perguntas por arquivo como evolucao considerada, com emenda previa do PROTOCOL.md e spike do Claude Code como pre-requisito. A ADR-0001 ganhou uma nota sob a decisao 2 apontando a revisao, no mesmo padrao que a ADR-0005 usou na ADR-0004. A paleta ainda nao esta no design system, entao nenhum criterio da spec e satisfeito so por este registro.
- Evidence: leitura do arquivo gravado (176 linhas, 15 cabecalhos, entre eles Status, Contexto, Decisao, Consequencias, Compliance e Notes) e da nota inserida na ADR-0001.
- Criteria: none
- Decisions: A-001 exige tambem a secao de paleta no design system, por isso fica para T-002; o indice de ADRs do AGENTS.md raiz fica para T-003.

## 2026-10-02 - T-002 - Paleta do meio terminal registrada no design system, com contraste medido
- Backlog: B-033
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: `docs/design-system/README.md` ganhou a secao 10 (meio terminal, decidida e ainda nao implementada) com paleta por papel, mapeamento de status, estrutura dos cartoes, marcadores do TODO e teclas, e o cabecalho e a linha do indice sobre `design-system.html` foram ajustados. `inconsistent` e registrado como vermelho, divergencia deliberada do ambar da web. A medicao de contraste contra `#282c34` mudou a paleta mostrada nos mockups: `dim` do One Dark (`#5c6370`, 2,3:1) cai na proibicao da secao 2 e fica restrito a bordas, entao o papel de texto secundario passou ao `meta` da web (`#9ba6b4`, 5,7:1), pois o `#828997` do One Dark da 4,0:1; o vermelho (4,4:1) ficou registrado como lacuna aceita, sempre com negrito, glifo e rotulo. `tokens.css` e a paleta web nao foram alterados.
- Evidence: contraste calculado pela formula WCAG num script (fg 6,57; meta 5,67; green 6,94; blue 5,92; yellow 8,10; red 4,38; rosa 5,70; dim 2,32; bar_empty 1,43; red sobre `#1e1e1e` 5,22), e releitura da secao gravada. Juntos, este registro e o de T-001 (ADR-0009 com Rust, segundo core com suite de conformidade, distribuicao por binario e nao ser Relay CLI) cobrem os dois lados do criterio.
- Criteria: A-001
- Decisions: `design-system.html` nao foi alterado: o guarda proibe a leitura e o README passou a dizer que o arquivo cobre so os tokens web; a secao 10 nao tem CSS a derivar. Se o HTML precisar de um exemplo do terminal, e trabalho a registrar a parte.

## 2026-10-02 - T-003 - ADR-0009 entra no indice de decisoes do AGENTS.md raiz
- Backlog: B-033
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: o item da ADR-0009 foi inserido no topo da lista "Architecture decisions" de `AGENTS.md` (`CLAUDE.md` e symlink dele), em ingles como o resto do arquivo, resumindo o binario Rust passivo, o segundo core contido pela suite de conformidade, a revisao parcial da ADR-0001 e o canal de perguntas como evolucao condicionada. Nenhum outro trecho do `AGENTS.md` foi tocado.
- Evidence: releitura do trecho editado; `CLAUDE.md` segue symlink para `AGENTS.md`.
- Criteria: none
- Decisions: o ponteiro de uma linha em `app/AGENTS.md` e em `README.md` previsto no escopo da spec fica no B-039 (documentacao de uso), nao aqui.

## 2026-10-02 - T-001 - Formato de app/conformance/ e runner no relay-core em TS
- Backlog: B-034
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-core/test/conformance.test.ts` e `app/conformance/README.md`. O runner percorre `app/conformance/<caso>/`, le `workspace/` com o mesmo mapeamento do host (registro ausente = texto vazio, specs sob `.specs/<nome>`), chama `deriveState` e compara por igualdade estrutural com `expected.json` (so o `RelayState`, sem `environment`). Exige o conjunto exato de 20 casos (sete `status-*` e treze `check-*`, listados no teste) e, a mais, confere que cada `check-*` contem so a violacao que nomeia e que cada `status-*` deriva o status do nome, para que um expected mal escrito nao passe. A leitura de arquivos fica em `test/`, entao `src/` segue puro.
- Evidence: linha de base `npm test --workspace relay-core`: 30 passam. Com o runner e sem casos, 33 testes e 1 falha, justamente "a suite tem os sete casos de status e um caso por verificacao de integridade" (falha vermelha esperada antes de T-002 e T-003).
- Criteria: none
- Decisions: o `expected.json` guarda o texto completo dos detalhes de violacao, porque A-003 pede o mesmo `RelayState` no core em Rust e um texto livre divergente e uma divergencia de verdade. Os testes existentes (`fixtures.test.ts`, `integrity.test.ts`) ficam intactos: a suite os estende, nao os substitui.

## 2026-10-02 - T-002 - Sete casos de status materializados a partir dos fixtures
- Backlog: B-034
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/conformance/status-{idle,backlog,ready,in_progress,blocked,done,inconsistent}/`, cada um com `workspace/.orchestration/` (e `workspace/.specs/` no caso `done`, que precisa da spec com A-001..A-008) e `expected.json`. As entradas sao as mesmas de `fixtures.test.ts`, gravadas como arquivos. O `expected.json` de cada caso e o `state` de `app/fixtures/<status>.json`, sem `environment`, portanto um oraculo independente do `deriveState`: o core nao gerou o proprio gabarito. A geracao usou um script descartavel fora do repositorio.
- Evidence: `npm test --workspace relay-core`: 40 testes, 39 passam; os sete `conformance: status-*` passam e a unica falha restante e a que exige os 13 casos `check-*`, que e T-003.
- Criteria: none
- Decisions: nenhuma — o formato ja estava decidido em T-001.

## 2026-10-02 - T-003 - Treze casos de integridade materializados, cada um com a unica violacao que nomeia
- Backlog: B-034
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/conformance/check-<id>/` para as 13 verificacoes (handoff-names-no-pending-todo, backlog-id-mismatch, spec-path-mismatch, handoff-harness-invalid, handoff-updated-invalid, multiple-handoffs, todo-cleared-before-changelog, backlog-done-with-pending-todo, unknown-marker, needs-unknown-id, needs-cycle, needs-incomplete-on-done, criteria-without-evidence). As entradas sao as de `integrity.test.ts` gravadas em disco, mudando um ponto por caso; registros vazios nao geram arquivo (o leitor os trata como texto vazio). Nao ha oraculo independente para estes expected, entao o rascunho veio de `deriveState` e cada um dos 13 foi lido e confrontado com a secao "Integrity checks" do `docs/PROTOCOL.md`: cada caso viola a condicao que nomeia, o `detail` descreve o fato certo (IDs e valores do proprio workspace) e `records` lista os registros envolvidos.
- Evidence: `npm test --workspace relay-core`: 53 de 53 passam (30 anteriores, os sete de status, os treze de check e tres testes de cobertura do conjunto); `npm run typecheck --workspace relay-core` limpo; `app/conformance/` tem 20 casos e 97 arquivos.
- Criteria: none
- Decisions: o expected de check-todo-cleared-before-changelog lista `handoff` em `records` mesmo com o handoff vazio no caso; e comportamento do core em TS, que e a referencia, e fica gravado como esta para o core em Rust reproduzir. Um criterio diz respeito ao conjunto completo e e nomeado em T-004.

## 2026-10-02 - T-004 - Suite inteira verde no relay-core e provada sensivel a divergencia
- Backlog: B-034
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: `app/conformance/` tem os 20 casos (sete `status-*` e treze `check-*`), cada um com `workspace/` em disco e `expected.json`, e o `relay-core` em TS passa todos pelo runner `conformance.test.ts`. Para mostrar que a suite nao passa por inercia, foram aplicadas duas mutacoes numa copia de trabalho e depois revertidas: trocar o texto de um `detail` no expected de `check-needs-cycle` e mudar um marcador no `TODO.md` do workspace de `status-ready`; as duas quebraram exatamente o caso mutado.
- Evidence: `npm test --workspace relay-core`: 53 de 53 passam; sob mutacao, 51 passam e 2 falham (`conformance: check-needs-cycle` e `conformance: status-ready`); apos restaurar a copia, `diff -r` entre `app/conformance/` e o backup nao acusa diferenca e os 53 voltam a passar; `typecheck` do relay-core limpo.
- Criteria: A-002
- Decisions: nenhuma. Fica para B-035: o core em Rust le este mesmo diretorio e tem de reproduzir os `expected.json`, texto de `detail` incluido.

## 2026-10-02 - T-001 - Toolchain Rust instalada e projeto Cargo app/relay-tui criado
- Backlog: B-035
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: a maquina nao tinha Rust; por escolha do usuario foi instalado o rustup oficial (`rustup-init.sh` de sh.rustup.rs, conferido antes de rodar) com `-y --no-modify-path --profile minimal`, em `~/.cargo` e `~/.rustup`, sem editar nenhum arquivo de shell (as chamadas usam `$HOME/.cargo/bin` no PATH da propria chamada). Criado `app/relay-tui/` (package `relay-tui`, edition 2024, MIT, `publish = false`): `src/lib.rs` expondo `pub mod core`, `src/core/mod.rs` vazio, `src/main.rs` minimo, `.gitignore` com `/target` dentro do proprio projeto (para `rm -rf app/` continuar devolvendo o repositorio ao estado anterior). Dependencias: `regex` no core, `serde_json` so em dev-dependencies; nenhuma serde na producao.
- Evidence: `cargo --version` 1.99.0 (rustc 1.99.0); `cargo build` compila; `git check-ignore` confirma `app/relay-tui/target` ignorado; `npm ls --workspaces` e `npm test --workspace relay-core` (53 de 53) seguem sem alteracao com o diretorio Cargo ao lado, que o glob `relay-*` do npm nao trata como workspace por nao ter `package.json`.
- Criteria: none
- Decisions: `regex` entra como dependencia do core porque as regex de `parse.ts` tem semantica propria que uma reescrita a mao arriscaria divergir; `\d` vira `[0-9]` e `.` exclui `\r`, `\n`, U+2028 e U+2029 para igualar o JavaScript. serde nao entra: o teste monta o JSON a mao, o que tambem confere o formato da ADR-0003 com uma segunda leitura.

## 2026-10-02 - T-002 - Runner em Rust le os 20 casos de conformidade e falha antes do core existir
- Backlog: B-035
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-tui/src/core/types.rs` (os tipos da ADR-0003: WorkStatus, Handoff, ChecklistEntry, OkState, Violation, RelayState, RelayFiles), um `derive_state` provisorio que nao deriva nada, e `app/relay-tui/tests/conformance.rs`. O teste exige o conjunto exato de 20 casos e, para cada um, le `workspace/` com o mesmo mapeamento do host e do runner TS, deriva e compara com `expected.json` por igualdade de `serde_json::Value`, listando todos os casos divergentes de uma vez. A conversao do estado para JSON e escrita a mao no teste (camelCase, `spec` ausente quando `None`, `null` para handoff e backlog ativo ausentes), sem serde na producao.
- Evidence: `cargo test` com o stub: `the_suite_has_every_status_case_and_one_case_per_integrity_check` passa (os 20 nomes batem com os diretorios de `app/conformance/`) e `every_case_derives_the_expected_state` falha, como deve antes do core.
- Criteria: none
- Decisions: um unico teste percorre todos os casos em vez de um teste por caso, para nao depender de macros ou de uma crate de testes dinamicos; a mensagem lista cada caso divergente com o esperado e o obtido.

## 2026-10-02 - T-003 - Parse em Rust equivalente ao parse.ts, com os quirks da referencia
- Backlog: B-035
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/core/parse.rs` com `parse_backlog`, `parse_todo`, `parse_handoff`, `parse_changelog` (so backlog_id, todo_id, spec e criteria, que sao os campos que a derivacao le) e `parse_spec_criteria`. As regex foram ajustadas para igualar o JavaScript, nao o contrario: `\d` como `[0-9]`, `\b` ASCII, `.` sem `\r`/`\n`/U+2028/U+2029 e marcador de checklist sem caracteres fora do BMP. Os testes unitarios partem de saidas reais do parser TS rodado sobre os mesmos insumos. Dois comportamentos da referencia foram reproduzidos e merecem atencao: uma linha de checklist terminada em `\r` (arquivo CRLF) nao casa nunca, e um segundo `- Status:` dentro de uma secao do handoff conta como outro handoff, sobrescreve o status e encerra a secao.
- Evidence: `cargo test --lib`: 6 de 6 passam. Mutacao: trocar a classe `[^\n\r...]*` por `.*` na regex de checklist faz `checklist_lines_follow_the_reference_parser` falhar (5 passam, 1 falha); restaurado o arquivo, os 6 voltam a passar e `diff` nao acusa diferenca.
- Criteria: none
- Decisions: nenhuma alem das de T-001. O teste que fixa o comportamento de CRLF documenta a referencia, nao a endossa: ver o relato ao usuario sobre arquivos CRLF no Windows.

## 2026-10-02 - T-004 - Integridade e derivacao em Rust passam nos 20 casos de conformidade
- Backlog: B-035
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-tui/src/core/integrity.rs` (as 13 verificacoes na ordem do TS, no maximo uma violacao por verificacao, com o texto exato dos detalhes) e o `derive_state` real em `derive.rs` (disponibilidade, status, handoff publico, contagens). Fidelidade com o TS onde o Rust difere: o mapa por id do TS deixa o ultimo valor e a ordem da primeira aparicao (reproduzido com vetor de ordem e `HashMap`); a DFS do ciclo usa cores 0/1/2 e trilha como a original; o agrupamento por spec preserva a ordem de insercao; spec ausente ou vazia em `specs` e ignorada. `Date.parse` do V8, usado pelo core TS na validacao de `Updated`, foi medido com node e replicado em `rfc3339_is_parseable`: o V8 aceita dia 31 em qualquer mes (`02-31` passa), aceita hora 24 so como `24:00:00` e rejeita mes, minuto, segundo ou offset fora do intervalo; uma tabela de 30 strings, tirada dessas medicoes, vira teste unitario.
- Evidence: `cargo test`: 8 testes unitarios (6 de parse, 2 de integridade) e os 2 do runner passam, incluindo `every_case_derives_the_expected_state` sobre os 20 casos, que falhava com o stub de T-002. Compila sem avisos.
- Criteria: none
- Decisions: nenhuma alem das de T-001. Nao ha criterio nomeado aqui: A-003 exige tambem que o core nao acesse disco, o que T-005 verifica.

## 2026-10-02 - T-005 - Core em Rust verde na suite de conformidade e provado puro
- Backlog: B-035
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/tests/purity.rs`, equivalente do `purity.test.ts`: percorre `src/core/*.rs` e falha se o codigo (comentarios fora) usar `std::fs`, `std::env`, `std::process`, `std::net`, `std::io`, `std::time`, `std::thread`, `std::os`, `tokio` ou `notify`. Com ele, o `core` em Rust deriva o mesmo `RelayState` que o core TS nos 20 casos de `app/conformance/` (sete de status e treze de integridade, `inconsistent` e contagens inclusos) sem acessar disco; so o runner de teste le arquivos.
- Evidence: `cargo test`: 11 testes passam (8 unitarios, 2 de conformidade, 1 de pureza) e `npm test` no relay-core segue 53 de 53, sobre os mesmos 20 casos. Duas mutacoes revertidas, cada uma pega pelo teste certo: trocar `Ciclo em` por `Ciclo no` em `integrity.rs` faz `every_case_derives_the_expected_state` listar `check-needs-cycle` como divergente; acrescentar `use std::fs` em `derive.rs` faz `the_core_touches_neither_disk_nor_environment` falhar apontando o arquivo. Apos restaurar, `diff -r` de `src/core` contra o backup nao acusa diferenca.
- Criteria: A-003
- Decisions: o criterio fala de "mesma suite" e e demonstrado apenas para o que a suite cobre. A suite exercita um ramo de cada verificacao, e varias tem mais (handoff-names-no-pending-todo tem 2, spec-path-mismatch 4, backlog-id-mismatch 2): nesses ramos o Rust e um porte fiel, nao uma equivalencia provada. Recomendado acrescentar casos extras na suite, sem mudar a regra de um caso `check-*` por verificacao nem o runner TS (que lista os nomes).

## 2026-10-02 - T-001 - Leitura do workspace e resolucao do caminho observado
- Backlog: B-036
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/workspace/mod.rs` (unico lugar que toca o disco) com `resolve_workspace(arg, cwd)`, que mantem um caminho absoluto, resolve um relativo contra o cwd e usa o proprio cwd sem argumento, colapsando `.` e `..` de forma lexical como o `path.resolve` do node (ADR-0007 decisao 2), e `read_workspace(path)`, que devolve um `RelayFiles` com os quatro registros de `.orchestration/` e cada `.specs/*.md` sob a chave `.specs/<nome>`. Port de `reader.ts`: registro ausente ou nao UTF-8 le como texto vazio, e um diretorio inexistente le como workspace vazio, sem erro. `notify` entrou em dependencies e `tempfile` em dev-dependencies. Os testes de caminho usam uma raiz absoluta da plataforma, para o CI no Windows.
- Evidence: `cargo test --test workspace_read`: 7 de 7 passam. Antes da implementacao a suite nao compilava (`file not found for module workspace`), o vermelho esperado.
- Criteria: none
- Decisions: o shell da ferramenta nao le o `~/.zshrc` (nao e interativo), entao as chamadas continuam exportando `$HOME/.cargo/bin` no PATH; em terminais do usuario o `cargo` ja esta no PATH pela linha que ele pediu em `~/.zshrc`.

## 2026-10-02 - T-002 - Debounce trailing de 150 ms sobre canais, testado sem sistema de arquivos
- Backlog: B-036
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/workspace/debounce.rs`, port de `watcher.ts`: `QUIESCENCE` = 150 ms, `WorkspaceEvent { Dirty, Settled }` e `debounce(raw, out, quiescence, stop, on_idle)`, que emite `Dirty` no primeiro evento de uma rajada, reinicia a janela a cada novo evento e emite um unico `Settled` apos a quiescencia. Encerra quando o canal de eventos fecha, o receptor some ou a flag de parada e acionada (checada a cada 200 ms ocioso). O gancho `on_idle` roda a cada espera ociosa e apos cada rajada e, se devolver `true` ocioso, conta como evento: e como um diretorio que aparece depois da partida sera percebido sem evento de arquivo (T-003).
- Evidence: `cargo test --lib workspace`: 8 de 8 passam (constante 150 ms; rajada de cinco escritas a 10 ms gera um `Dirty`, um `Settled` nao antes da janela e nenhum sinal extra em 500 ms; escrita dentro da janela adia o `Settled`; duas rajadas separadas geram dois pares; sem eventos nada e emitido; fechar a origem e a flag de parada encerram a thread; algo achado ocioso vira `Dirty` e `Settled`). Seis execucoes seguidas passaram sem oscilacao. Mutacao: trocar o `Ok(()) => {}` do laco de drenagem por `Ok(()) => break` (janela que nao reinicia) faz exatamente `a_burst_of_writes...` e `a_write_inside_the_window...` falharem; arquivo restaurado e conferido com `diff`.
- Criteria: none
- Decisions: o debounce e a parte pura (canais e tempo) para que o criterio dos 150 ms seja testado de forma deterministica; o teste com arquivos reais fica em T-003, onde a latencia do sistema operacional entra.

## 2026-10-02 - T-003 - Watcher sobre notify com teste de integracao de uma rajada real de escritas
- Backlog: B-036
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/workspace/watch.rs` com `watch_workspace(workspace, quiescence)` e `WorkspaceWatcher` (`events()` devolve o canal de `Dirty`/`Settled`; o `Drop` liga a flag de parada e junta a thread). Vigia so `.orchestration/` e `.specs/` (nao o repositorio inteiro, como o `watcher.ts`), ignora todo evento `Access` (a leitura de um registro abre e fecha o arquivo, e reagir a isso faria cada releitura acordar o watcher para sempre no inotify) e anexa depois um diretorio que nao existia na partida, pelo gancho `on_idle` do debounce; um workspace inexistente nao e erro.
- Evidence: `tests/workspace_watch.rs`, 6 de 6 contra um diretorio real: cinco escritas a 20 ms geram um `Dirty`, um `Settled` nao antes da janela e nenhum sinal em 700 ms, e a releitura vê a ultima escrita; uma escrita em `.specs/` tambem e vista; 20 leituras seguidas nao acordam o watcher; um `.orchestration/` criado depois e percebido e suas escritas passam a ser vistas; workspace inexistente nao falha; o `Drop` encerra em menos de 1 s. Oito execucoes seguidas passaram sem oscilacao. Mutacao: vigiar `.specs-x` em vez de `.specs` faz o teste de `.specs/` falhar (5 passam, 1 falha); arquivo restaurado, `diff` identico. Suite total: `cargo test` 32 de 32 (16 lib, 2 conformancia, 1 pureza, 7 leitura, 6 watcher), `cargo build --all-targets` com 0 avisos, `npm test` do relay-core 53 de 53.
- Criteria: A-004
- Decisions: so macOS foi exercitado aqui. O filtro de `Access` protege o inotify do Linux e o `ReadDirectoryChangesW` do Windows tem outro comportamento; os dois sao verificados de verdade pelo CI em tres sistemas do B-039, nao por este registro. O criterio A-004 fala de "caminho explicito ou diretorio corrente": `resolve_workspace` (T-001) cobre os dois e o `read_workspace` le o que ele devolve.

## 2026-10-02 - T-001 - Decisoes da view fixadas no design system antes do codigo
- Backlog: B-037
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: `docs/design-system/README.md`, secao 10, ganhou a subsecao "View": cabecalho de uma linha (selo `relay`, caminho, frescor com ponto e palavra), cartao Handoff (status no titulo e na borda, linha de IDs com harness e tempo relativo, Objetivo/Proximo com recuo, Bloqueio em `blocked`, teto de linhas com `…`), cartao TODO (barra por item, marcadores com texto, `após T-NNN`), cartao Backlog (contagens com palavra; lista os disponiveis em `backlog`), rodape, os estados sem handoff (`ready`, `done`, `idle` e nao-e-workspace-Relay), o cartao de violacoes, o tempo relativo e as regras de altura (corte em `+N itens`, Backlog vira uma linha, abaixo de 6 linhas so cabecalho e rodape) e de largura (referencia de 58; abaixo de 40 so o status). Segue a governanca da propria secao 9: a decisao entra primeiro no README, o codigo vem depois. Escolha consciente: o selo `relay` e verde e fixo, como no mockup aprovado, e o README registra que e marca e nao status.
- Evidence: releitura da subsecao gravada contra os mockups aprovados e contra a paleta e a tabela de status ja registradas na mesma secao; nenhum valor de cor novo foi criado (so `#282c34` como texto do selo, o fundo do One Dark que a propria secao ja usa para medir contraste).
- Criteria: none
- Decisions: o criterio A-005 e satisfeito pelos snapshots de T-004, nao por este registro; aqui so ha a decisao que o codigo vai seguir.

## 2026-10-02 - T-002 - Tema e helpers de texto da view
- Backlog: B-037
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-tui/src/theme.rs` (a paleta da secao 10 como constantes `Color::Rgb`, os estilos `fg`/`bold`/`color`, `status_tone` e `status_label` com as mesmas palavras da interface web, `INCONSISTENT_LABEL`) e `src/view/text.rs` (`width`, `truncate` com `…`, `wrap` que colapsa espacos e parte palavra maior que a linha, `wrap_capped` com teto de linhas e `…`, `parse_rfc3339` sem crate via `days_from_civil`, `relative_time` em portugues com relogio injetado). `ratatui` 0.30.2 e `unicode-width` entraram em dependencies. Um ramo morto de `wrap_capped` foi removido apos a primeira passada verde.
- Evidence: `cargo test --lib`: 26 de 26 passam (os 16 anteriores mais 10 novos: hex exatos de cada cor, tom e rotulo de cada status, largura de 1 coluna dos glifos do design, truncate incluindo caractere largo `日本語`, wrap, palavra maior que a linha, teto de linhas, epoch de 1970 e 2000, offsets `-03:00` e `+05:30`, `relative_time` de agora a dias, relogio no futuro e texto nao parseavel).
- Criteria: none
- Decisions: `unicode-width` confirmou que `✓ ● ○ ◌ • … · ╭ ─ │` ocupam uma coluna, e isso virou teste, porque a moldura e os marcadores dependem disso para alinhar.

## 2026-10-02 - T-003 - View em ratatui: cartoes, estados e degradacao por altura e largura
- Backlog: B-037
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-tui/src/view/mod.rs` e `view/cards.rs`. `View { screen, workspace, freshness, now_unix }` desenha, como `Widget`, o cabecalho (selo `relay`, caminho truncado, frescor com ponto e palavra), o cartao Handoff (status no titulo e na borda, IDs com harness e tempo relativo, Objetivo/Proximo com recuo, Bloqueio em `blocked`), o TODO (barra de um segmento por item, marcadores com texto, `apos T-NNN`, corte em `+N itens` com o item em andamento), o Backlog (cartao, ou uma linha quando falta altura) e os estados sem handoff (`ready`, `done`, `idle`, `backlog` listando os disponiveis, nao-e-workspace-Relay, violacoes). Abaixo de 40 colunas so cabecalho e status; abaixo de 6 linhas so cabecalho e rodape. A revisao das saidas gerou quatro ajustes alem do plano, todos registrados no README (secao 10, subsecao View): cartoes de um assunto tem a altura do conteudo (antes preenchiam a tela com moldura vazia), o Handoff se compacta antes de o TODO sumir, o Backlog em uma linha omite o que nao cabe inteiro (em vez de cortar no meio) e `ready` ganhou o status em palavras (`● Pronto`), que nao aparecia em lugar nenhum; mais concordancia de plural.
- Evidence: `cargo build --all-targets` com 0 avisos; `tests/view_smoke.rs`, 2 de 2: todas as telas (sete estados, violacao, nao-Relay) em 14 tamanhos de 0x0 a 120x50 e nas duas frescuras, sem panico, e uma area fora da origem nao e desenhada alem de si. Leitura de cada saida em 58 e 40 colunas antes de aceitar o snapshot.
- Criteria: none
- Decisions: o selo `relay` ficou verde e fixo, como no mockup aprovado; a lista de dependencias abertas de um item (`apos T-003`) e uma busca na propria lista de itens, enquanto `available` continua vindo pronto do core.

## 2026-10-02 - T-004 - Snapshots do TestBackend e regras de cor e de rotulo
- Backlog: B-037
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `app/relay-tui/tests/view_snapshots.rs` (7 testes, 26 snapshots em `tests/snapshots/` com o texto da tela e um mapa de cor por celula, legenda em `tests/snapshots/README.md`) e `tests/view_semantics.rs` (8 regras). Os snapshots cobrem os sete estados em 58 e 40 colunas, a frescura `atualizando`, o cartao de violacoes (uma e tres, em 58 e 40 colunas e cortado por altura), um TODO longo cortado em tres alturas, e os degradados de 30 colunas e de 5 linhas; os estados vem dos casos de `app/conformance/`, e os de TODO longo e de varias violacoes sao montados a mao. As regras dizem o que os snapshots mostram: todo status aparece por extenso em 58 e 40 colunas; rotulo e marcador `●` tem o tom do status e rotulo em negrito; a borda do Handoff tem o tom do status; `Inconsistente` e `check` sao vermelhos mas o `detail` e `fg`; um item bloqueado diz `bloqueado` em amarelo; o frescor e palavra alem de ponto; `dim` e `bar_empty` so aparecem em moldura e barra; nenhuma celula usa cor fora da paleta.
- Evidence: `cargo test`: 59 testes passam (26 lib, 2 conformancia, 1 pureza, 7 leitura, 6 watcher, 2 fumaca, 8 regras, 7 snapshots), `cargo build --all-targets` com 0 avisos, `npm test` do relay-core inalterado. Duas mutacoes revertidas: tirar o rotulo da linha `Pronto` derruba `every_status_names_itself_in_words` e `the_label_and_the_marker_carry_the_tone_of_the_status`; pintar o `check` da violacao de amarelo derruba `inconsistent_is_red_but_its_body_is_not`; arquivo restaurado, `diff` identico.
- Criteria: A-005
- Decisions: nos snapshots o mapa de cor fixa a paleta celula a celula; `UPDATE_SNAPSHOTS=1` os regrava e o diff e a revisao. Foram regerados tres vezes durante o trabalho, depois de revisar a saida de cada tela, e os tres defeitos achados assim (cartao vazio, TODO sumindo, `Pronto` sem palavra) so apareceram por esse olhar, nao por um teste.

## 2026-10-02 - T-001 - Argumentos de linha de comando do relay-tui
- Backlog: B-038
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/cli.rs` com `parse_args` (`Action::{Run, Version, Help}`), `--workspace <caminho>` e `--workspace=<caminho>`, `-V`/`--version`, `-h`/`--help` (a ajuda vence o resto da linha) e erros explicitos em portugues para argumento desconhecido, `--workspace` sem valor ou vazio e `--workspace` repetido: o que nao e entendido e erro, nao palpite. O texto de ajuda diz que o programa so le. Tres argumentos nao pedem crate de parsing.
- Evidence: `cargo test --lib cli`: 6 de 6 passam (sem argumentos, as duas formas do workspace inclusive com espaco e `..`, os dois nomes de versao e de ajuda, ajuda vencendo, os quatro erros, mensagens).
- Criteria: none
- Decisions: nenhuma alem das do handoff.

## 2026-10-02 - T-002 - Estado da aplicacao e tratamento de eventos, sem terminal
- Backlog: B-038
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/src/app.rs` com `App` (workspace, caminho exibido, estado carregado ou "nao e workspace Relay", frescor, saida), `AppEvent::{Terminal, Workspace}`, `on_event`, `should_quit`, `view(now)` e `display_path` (a home vira `~`); e `workspace::is_relay_workspace` (existe `.orchestration/`). `Dirty` marca `atualizando` e mantem o ultimo snapshot; `Settled` relê o workspace inteiro, deriva e volta a `atualizado` (ADR-0007 decisao 4); um `.orchestration/` criado depois tira a tela do cartao de nao-Relay; `q`, `Q`, `Esc` e `Ctrl-C` saem, so em `Press` (o Windows manda `Release` tambem), e `Ctrl-Q`, `c` solto, Enter, resize e demais teclas nao saem. Usa o re-export `ratatui::crossterm`, sem dependencia direta de versao propria.
- Evidence: `cargo test --lib`: 41 de 41 passam (os 10 testes novos de `app`: diretorio sem `.orchestration`, workspace com registros, `Dirty` mantendo o snapshot sem reler, `Settled` relendo e chegando a `done`, workspace que aparece depois, as quatro teclas de saida, o que nao sai, `Release` nao sai, `~` com um irmao `/Users/meow` que nao e a home).
- Criteria: none
- Decisions: a `App` le o disco, mas o core segue puro (o teste de pureza continua valendo so para `src/core/`); a deteccao de "nao e Relay" mora no `workspace`, a unica camada de disco.

## 2026-10-02 - T-003 - Binario relay-tui: terminal, laco de eventos e watcher ligados
- Backlog: B-038
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: `app::run` (laco generico sobre `Terminal<B>`: desenha, espera um evento com tick de 15 s, aplica tambem tudo que ja esta na fila e redesenha uma vez, e sai em `q`/`Esc`/`Ctrl-C` ou quando a fonte de eventos some) e `src/main.rs`: argumentos pelo `cli`, `--version`/`--help` em stdout com codigo 0, erro de uso em stderr com `Use --help` e codigo 2, exigencia de stdout ser terminal, workspace resolvido contra o cwd, home por `HOME`/`USERPROFILE`, um canal unico `AppEvent` alimentado por uma thread de entrada (crossterm) e uma do watcher, `ratatui::init()`/`restore()` (modo raw, tela alternativa e gancho de panic que restaura antes da mensagem). Um gancho de panic de teste, `RELAY_TUI_TEST_PANIC`, so existe em builds debug (`cfg!(debug_assertions)`).
- Evidence: `cargo test --lib`: 43 de 43 passam (dois novos de `run` com `TestBackend`: uma rajada `Dirty`+`Settled`+`q` ja na fila termina com a tela `A escolher` e `atualizado`, sem `atualizando`; fonte de eventos que some encerra o laco). `cargo build --all-targets` sem avisos. Em `target/debug/relay-tui`: `--version` imprime `relay-tui 0.0.0`, `--help` imprime a ajuda, `--exec` sai com 2 e a dica, e fora de um terminal sai com 1 e `a saída precisa ser um terminal`.
- Criteria: none
- Decisions: o `run` devolve `Box<dyn Error + Send + Sync>` para o `main` converter em `io::Error::other`; a restauracao do terminal em panic fica por conta do gancho do `ratatui::init`, e e provada de ponta a ponta em T-004, nao suposta.

## 2026-10-02 - T-004 - Teste ponta a ponta num pty real: observar, reagir, redesenhar e restaurar
- Backlog: B-038
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/tests/e2e_pty.rs`, que roda o binario real (`CARGO_BIN_EXE_relay-tui`) num pseudo-terminal (`portable-pty`) e le a tela por um emulador de terminal (`vt100`), com um historico de todas as telas distintas para nao perder um estado de 150 ms. `portable-pty` e `vt100` entraram so em dev-dependencies. Cobre: abrir e mostrar o workspace; uma mudanca de arquivo aparecendo sem reiniciar, passando por `atualizando` e chegando a `atualizado`; uma rajada de cinco escritas virando uma unica tela estabilizada (as versoes 1 a 4 nunca sao desenhadas); um diretorio sem `.orchestration/` mostrando o cartao de nao-Relay e passando a mostrar o workspace quando ele surge; o redesenho em 58, 40, 80 colunas e o degradado de 30 colunas apos resize; `q`, `Esc` e `Ctrl-C` saindo com codigo 0 e deixando o terminal na tela normal com o cursor visivel; um panic (gancho `RELAY_TUI_TEST_PANIC`, so em debug) saindo com codigo diferente de 0, restaurando o terminal e deixando a mensagem na tela normal; e o workspace byte a byte igual depois de observado. Um teste ignorado, `print_the_real_screens`, imprime o que o terminal de fato mostra.
- Evidence: `cargo test --test e2e_pty`: 10 de 10 passam, e seis execucoes seguidas passaram sem oscilacao. Tres mutacoes revertidas, cada uma pega pelo teste certo: remover `ratatui::restore()` derruba os tres de saida (`q`, `Esc`, `Ctrl-C`); fazer `Dirty` nao marcar `atualizando` derruba o de mudanca de arquivo; baixar a quiescencia para 5 ms derruba o da rajada. Arquivos restaurados, `diff` identico. As telas reais impressas conferem com os snapshots (inclusive `ha 25 d` com o relogio real contra fixtures de 2026-09-07).
- Criteria: A-006
- Decisions: a restauracao em panic e a do gancho que o `ratatui::init` instala; o teste prova o comportamento do binario, nao a implementacao do gancho. Um sinal externo (`kill -INT`/`SIGTERM`) nao e tratado: em modo raw `Ctrl-C` e uma tecla, e so ela esta no criterio.

## 2026-10-02 - T-005 - Garantia de somente leitura no codigo
- Backlog: B-038
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/tests/read_only.rs` com duas garantias sobre o codigo que e distribuido (`src/`, sem os modulos de teste nem os comentarios). A primeira falha se algum arquivo usar API de escrita em disco (`fs::write`, `File::create`, `OpenOptions`, `remove_*`, `create_dir*`, `rename`, `copy`, `hard_link`, `soft_link`, `set_permissions`, `symlink`), de lancar processo (`process::Command`, `Command::new`) ou `unsafe`/`libc::`. A segunda falha se `core/`, `view/` ou `theme.rs` leem o disco por qualquer via, o que deixa `workspace/` como a unica camada que toca o sistema de arquivos. `std::process::ExitCode` continua permitido: so `Command` lanca processo. Do outro lado, o teste ponta a ponta de T-004 ja mostra o workspace identico byte a byte depois de uma sessao de observacao.
- Evidence: `cargo test`: 88 testes passam em 9 binarios de teste (43 lib, 2 conformancia, 1 pureza do core, 2 leitura-somente, 7 leitura, 6 watcher, 2 fumaca da view, 8 regras da view, 7 snapshots, 10 e2e) e `cargo build --all-targets` tem 0 avisos; `npm test` do relay-core segue 53 de 53. Duas mutacoes revertidas: um `std::fs::write` em `src/app.rs` derruba a primeira garantia apontando o arquivo, e um `.is_dir()` em `src/view/mod.rs` derruba a segunda; `diff` confirma os arquivos restaurados.
- Criteria: A-007
- Decisions: o criterio fala em "teste ou inspecao de codigo"; foi escolhido o teste, e ele usa texto (busca de nomes de API), nao analise de sintaxe, entao um alias ou uma chamada por caminho diferente nao seria vista. Os dois lados juntos (codigo sem API de escrita e workspace inalterado numa execucao real) cobrem o que um so deixaria passar.

## 2026-10-02 - T-001 - Spec e ADR-0009 emendadas: macOS requisito, Linux e Windows melhor esforco
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: a spec ganhou a decisao "macOS primeiro (emenda de 2026-10-02)", o bullet de distribuicao do escopo e um non-goal reescritos, e o criterio A-008 agora exige binarios para macOS arm64 e x64 e `cargo test` no macOS no CI como requisito, com Linux x64/arm64 e Windows x64 em melhor esforco que nao impede release nem CI, e a documentacao dizendo quais alvos sao requisito. A ADR-0009 teve a decisao 4 e a conformidade 4 reescritas (com a emenda datada, para a mudanca nao parecer que sempre foi assim), a consequencia da matriz de CI ajustada, e uma excecao registrada: os dois workflows em `.github/workflows/`, fora de `app/`, resolvem o caminho `app/relay-tui`, o que a conformidade 3 da ADR-0004 proibe; a excecao e limitada a dois arquivos inertes sem `app/` (CI filtrado por `app/relay-tui/**`, release por tags `relay-tui-v*`), e a ADR-0004 ganhou a nota correspondente, no mesmo padrao das emendas da ADR-0001 e da ADR-0005. A ADR-0009 ganhou a conformidade 5 (so esses dois workflows) e as seguintes foram renumeradas.
- Evidence: releitura dos trechos gravados; o texto antigo e o novo do A-008 e da decisao 4 conferidos lado a lado.
- Criteria: none
- Decisions: o titulo do B-039 no backlog ("Release multiplataforma...") nao foi alterado, porque o backlog nomeia um resultado e o resultado continua sendo o release; o que mudou foi o criterio, que vive na spec. A aprovacao do usuario ("sim, segue com o B-039 assim") cobre a proposta de reescrever o A-008 e a ADR-0009.

## 2026-10-02 - T-002 - Perfil release e script de empacotamento, testados nos dois alvos macOS
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: `app/relay-tui/Cargo.toml` ganhou `[profile.release]` (`lto`, `codegen-units = 1`, `strip`), que leva o binario de 3,0 MB para 1,9 MB. Criado `app/relay-tui/scripts/package.sh <alvo>`: le a versao do `Cargo.toml`, roda `cargo build --release --locked --target`, monta `dist/relay-tui-<versao>-<alvo>/` com o binario, a `LICENSE` da raiz e o `README.md` do projeto quando existir, gera `.tar.gz` (`.zip` pelo `7z` em alvos Windows) e o `.sha256` (com `sha256sum` ou `shasum -a 256`). `dist/` entrou no `.gitignore` do projeto. O workflow de release chama este mesmo script, entao o que o CI publica e reproduzivel num Mac.
- Evidence: o script rodou para `aarch64-apple-darwin` (alvo nativo) e `x86_64-apple-darwin` (cross-compile no Apple Silicon): dois `.tar.gz` de 0,8 MB, `shasum -a 256 -c` dando `OK` para ambos, conteudo `relay-tui` e `LICENSE`, e `file` confirmando `Mach-O 64-bit executable arm64` e `x86_64`; os dois binarios desempacotados rodaram (`relay-tui 0.0.0`; o x86_64 por Rosetta). Um teste extra mostrou que um binario baixado com o atributo `com.apple.quarantine` nao executa: o Gatekeeper o segurou ate eu matar o processo, o que confirma na pratica que o contorno e necessario. Ficou sem confirmacao o lado inverso: depois de remover o atributo, a segunda execucao ficou presa atras da avaliacao pendente do mesmo binario, entao `xattr -d com.apple.quarantine` e documentado como o contorno padrao, nao como verificado aqui. Linux e Windows nao foram construidos: nao ha toolchain `musl` nem Windows nesta maquina.
- Criteria: none
- Decisions: o script fica em `app/relay-tui/scripts/` e le `../../LICENSE`, o que e `app/` apontando para fora, a direcao permitida pela ADR-0004. O `panic` continua com unwinding no release: o gancho do `ratatui` restaura o terminal antes de qualquer saida, e `panic = "abort"` pouparia pouco.

## 2026-10-02 - T-003 - Workflows de CI e de release: macOS requisito, Linux e Windows melhor esforco
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criados `.github/workflows/relay-tui-ci.yml` e `relay-tui-release.yml`, os dois unicos arquivos de `.github/` e a excecao registrada na ADR-0009 decisao 4. O CI so dispara com mudancas em `app/relay-tui/**`, `app/conformance/**`, `app/relay-core/**` ou nele mesmo: o job `macos` (requisito) roda `cargo test --locked` e a suite de conformidade no core TS (`npm ci` so do `relay-core`), e o job `best-effort` (ubuntu e windows) tem `continue-on-error`. O release dispara so em tags `relay-tui-v*`: o job `version` falha se a tag nao for `relay-tui-v<versao do Cargo.toml>`; `macos` (aarch64 e x86_64) testa, empacota com `package.sh` e sobe o artefato; `best-effort` (linux x64 musl, linux arm64 musl em runner arm nativo, windows msvc) tem `continue-on-error` e `fail-fast: false`; `publish` roda com `always()` mas so se `macos` teve sucesso, junta os artefatos que existirem e cria o Release com `gh release create --generate-notes`. Sem ancoras YAML, por cautela.
- Evidence: os dois YAML parseiam (Ruby/Psych) com os jobs esperados. Comandos que os jobs executam, rodados aqui: a verificacao tag x versao passa com `relay-tui-v0.0.0` e falha com `relay-tui-v9.9.9`; `npm ci --workspace relay-core --include-workspace-root` numa copia limpa de `app/` instala 4 pacotes e `npm test --workspace relay-core` passa 53 de 53; `cargo test --locked` passa tudo; `package.sh` ja provado em T-002. **Nenhum workflow foi executado no GitHub Actions**: sintaxe de `matrix`, `needs`/`always()`, `continue-on-error`, permissoes e o `gh release create` so um run real confirma.
- Criteria: none
- Decisions: a regra do `publish` e "so se o macOS passou, venha o que vier dos outros"; um `continue-on-error` de job nao e usado como sinal de sucesso. `Cargo.lock` esta untracked e o `--locked` exige que seja commitado: ficou como aviso para o dono do repositorio, nao como arquivo que eu commite.

## 2026-10-02 - T-004 - Documentacao de uso do relay-tui
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: criado `app/relay-tui/README.md` (o que e e o que nao faz, a tabela de alvos com macOS requisito e Linux e Windows melhor esforco, instalacao, as duas saidas da quarentena do Gatekeeper — baixar com `curl`, que nao aplica o atributo, ou `xattr -d com.apple.quarantine` —, o aviso do SmartScreen, `.sha256`, uso e teclas, o que a tela significa, limites conhecidos, como compilar e como publicar uma versao). `app/README.md` ganhou a secao do `relay-tui`; `app/AGENTS.md` ganhou o item de leitura, a excecao da regra "nunca calcular o que o relay-core deriva" para o core em Rust (a view continua sem calcular), o vinculo dos dois workflows na regra "nunca fazer a raiz depender daqui" e a camada de download; o `README.md` da raiz ganhou uma frase com a URL das releases, sem link para dentro de `app/`; o indice de ADRs do `AGENTS.md` raiz diz que macOS e requisito. O `package.sh` passou a embarcar o README no arquivo.
- Evidence: `package.sh aarch64-apple-darwin` gera um `.tar.gz` com `relay-tui`, `LICENSE` e `README.md`; `grep` confirma que nenhum arquivo fora de `app/` (README, `docs/INSTALL.md`, `docs/PROTOCOL.md`, skills, manifestos de plugin) tem link markdown para dentro dele; a afirmacao de que `curl` nao aplica a quarentena foi conferida baixando um arquivo de teste, que veio so com `com.apple.provenance`.
- Criteria: none
- Decisions: o contorno por `xattr -d` e documentado como o padrao do macOS, nao como verificado ponta a ponta aqui (ver T-002); o README diz as duas saidas e recomenda `curl` primeiro justamente por isso.

## 2026-10-02 - T-005 - Verificacao final local do B-039
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: conferencia de tudo que pode ser verificado sem o GitHub: a suite do `relay-tui`, o core TS sobre a mesma suite de conformidade, a sintaxe do `package.sh` e dos dois workflows, `.github/` com so os dois arquivos previstos pela excecao da ADR-0009, os dois pacotes macOS gerados e conferidos (T-002) e a documentacao (T-004). O que nao pode ser verificado daqui, e portanto nao e afirmado: que o CI roda no `macos-latest`, que uma tag `relay-tui-v*` aciona o release e que o `gh release create` publica os arquivos, que o `continue-on-error` dos alvos Linux e Windows se comporta como esperado e que esses dois alvos compilam; nem que `xattr -d com.apple.quarantine` libera um binario baixado.
- Evidence: `cargo test --locked`: 88 testes passam, `cargo build --all-targets --locked` com 0 avisos; `npm test` do relay-core: 53 de 53; `bash -n scripts/package.sh` ok; os dois YAML parseiam; `ls .github/workflows` lista exatamente `relay-tui-ci.yml` e `relay-tui-release.yml`; o binario `target/release/relay-tui --version` imprime `relay-tui 0.0.0`.
- Criteria: none
- Decisions: o criterio A-008 nao e nomeado aqui. Ele diz que uma tag publica os binarios no Release e que o CI roda `cargo test` no macOS, e isso so se demonstra rodando o GitHub Actions, o que exige commit, push e uma tag, acoes externas que ninguem pediu. Como o B-039 e a ultima entrada pendente da spec, o protocolo manda deixa-lo pendente com `[!]` e um handoff `blocked`, em vez de fechar sem a evidencia: o item T-006 foi acrescentado ao TODO para isso.
