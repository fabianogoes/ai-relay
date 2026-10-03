# Change log

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

## 2026-10-02 - T-006 - Run real do CI e do release: tag relay-tui-v0.1.0 publica os binarios de macOS
- Backlog: B-039
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Result: o PR #1 (feat/relay-tui) foi mergeado e o CI rodou de verdade; o repositorio passou a se chamar `fabianogoes/ai-relay` e as referencias foram atualizadas (PR #2); a versao subiu para 0.1.0 (PR #3). O primeiro run real revelou defeitos que nenhuma execucao local mostrou: o job de Windows falhou (separadores de caminho em `display_path`, corrigido) e foi removido do pipeline por pedido do dono (ADR-0009, "Windows adiado"); e os dois testes de rajada mediam o tempo a partir do fim de um `sleep`, que estoura em runner carregado, falhando o job `macOS (required)` em quatro execucoes (CI #5, #6, #8 e o release #1). O PR #5 passou a medir a partir da propria escrita e colocou `fail-fast: false` na matriz do release. Com o PR #5 mergeado, a tag `relay-tui-v0.1.0`, que nunca tinha chegado a publicar, foi recriada na nova `main` e o workflow `relay-tui release` terminou verde: a tag bateu com o `Cargo.toml`, os dois alvos de macOS testaram e empacotaram, os dois de Linux musl tambem, e `publish` criou o Release.
- Evidence: Release https://github.com/fabianogoes/ai-relay/releases/tag/relay-tui-v0.1.0 (run 37078896179, todos os jobs `success`) com oito arquivos: `relay-tui-0.1.0-aarch64-apple-darwin.tar.gz`, `…-x86_64-apple-darwin.tar.gz`, `…-aarch64-unknown-linux-musl.tar.gz`, `…-x86_64-unknown-linux-musl.tar.gz` e o `.sha256` de cada um. CI: run 37078436396 (PR #5) com `macOS (required)` e `ubuntu-latest` verdes. Baixado de verdade com `curl` da URL publica: `shasum -a 256 -c` deu `OK`, o arquivo veio sem `com.apple.quarantine` (confirma que `curl` nao aplica a quarentena, como o README diz), `file` deu `Mach-O 64-bit executable arm64`, o conteudo e `relay-tui`, `LICENSE` e `README.md`, e `relay-tui --version` imprime `relay-tui 0.1.0`.
- Criteria: A-008
- Decisions: a tag foi apagada e recriada porque a 0.1.0 nunca tinha sido publicada (o release falhou antes do `publish`) e nao havia nenhum Release a invalidar. Os binarios de Linux saíram no Release, mas so o x86_64 passa por `cargo test` no CI; o arm64 foi apenas compilado e empacotado, e nenhum dos dois foi executado numa maquina Linux.

## 2026-10-03 - T-001 - Tipo Suggestion e funcao pura suggest em src/suggest.rs
- Backlog: B-048
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: `app/relay-tui/src/suggest.rs` criado e exportado em `lib.rs`: `suggest(Option<&RelayState>) -> Suggestion` com `Case` para os onze casos da tabela, na ordem da spec, fora de `core` e de `view`; usa o `available` do core e nao le relogio nem disco.
- Evidence: `cargo build` em `app/relay-tui` compila sem erro; os testes da tabela ficam para T-002. clippy e rustfmt nao estao instalados neste toolchain, entao nao rodaram.
- Criteria: none
- Decisions: none

## 2026-10-03 - T-002 - Testes de suggest sobre os fixtures e casos sinteticos
- Backlog: B-048
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: `app/relay-tui/tests/suggest.rs` com 12 testes: os sete fixtures de status de `app/conformance/` (idle, backlog, ready, in_progress, blocked, done, inconsistent), casos sinteticos para nao-workspace, `blocked` sem handoff, `done` com TODO e `backlog` sem entrada disponivel, a escolha do primeiro item disponivel em ordem textual com o titulo sem anotacoes, e o determinismo (a mesma entrada, e o mesmo estado rederivado, dao a mesma sugestao, sem relogio).
- Evidence: `cargo test --test suggest` em `app/relay-tui`: 12 passed, 0 failed.
- Criteria: A-001
- Decisions: um `[x]` sintetico precisa de registro no changelog, senao o core deriva `inconsistent` (`todo-cleared-before-changelog`); o caso `done` com TODO e montado com o registro.

## 2026-10-03 - T-003 - Verificacao final local do B-048
- Backlog: B-048
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: a suite completa de `app/relay-tui` passa com o modulo `suggest` no crate, sem regressao nos testes de conformidade, pureza, somente-leitura, view e workspace.
- Evidence: `cargo test` em `app/relay-tui`: todos os alvos `ok`, 0 failed (43 unitarios, 12 em `suggest`, 10 em `e2e_pty`, 2 em `conformance`, 8 em `view_snapshots`). clippy e rustfmt nao estao instalados neste toolchain e nao rodaram.
- Criteria: none
- Decisions: none

## 2026-10-03 - T-001 - Linha de proximo passo desenhada na visao Agora
- Backlog: B-049
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: `src/view/hint.rs` monta a frase em portugues de cada caso a partir de `Suggestion` (skill em negrito `fg`, ids em `id`, resto em `meta`). `src/view/cards.rs` reserva a ultima linha do corpo, acima do rodape: o calculo de alturas de `work_body` virou o `plan` puro, e a linha so aparece se o plano com uma linha a menos mantem o Handoff e o TODO como estao (o Backlog pode virar linha). Nos corpos sem `work_body` ela aparece se sobrar uma linha alem do cartao. `fit` passou a terminar com `…` tambem quando o corte cai entre dois trechos.
- Evidence: `cargo test` em `app/relay-tui` sem falha apos regravar os snapshots; o diff de `tests/snapshots` so acrescenta a linha (36 linhas trocadas em 18 arquivos), por exemplo `Retome T-002 de B-001 com relay-sessio…` em 40 colunas. A revisao dos snapshots por criterio fica para T-002.
- Criteria: none
- Decisions: o `…` agora marca todo corte de `fit`, e nao so o que cai dentro de um trecho; antes, um corte exato entre trechos descartava o resto sem marca.

## 2026-10-03 - T-002 - Snapshots, semantica da view e read_only para a linha de proximo passo
- Backlog: B-049
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: regra de altura corrigida em `hint_fits` (`cards.rs`): a linha so fica enquanto, sem a sua linha, o Handoff nao se compacta e o TODO nao e cortado (descer de `Full` para `Standard` nao conta como compactar); com isso a presenca da linha e monotona na altura. Snapshots regravados (a linha aparece em 58 e 40 colunas, cortada com `…` em 40) e novos `in_progress-58-h12/h17/h18/h20`: ausente ate 17 linhas, presente a partir de 18. Em `tests/view_semantics.rs`: a skill em negrito `fg` e ids/resto em `id`/`meta` nos sete status, o corte com `…` em 40 colunas e a ausencia em 39 e 30, e a cessao ordenada por altura (nunca sobre Handoff compacto nem TODO cortado). `suggest.rs` entrou na lista de modulos sem leitura de disco em `tests/read_only.rs`.
- Evidence: `cargo test` em `app/relay-tui`: 105 passed, 1 ignored, 0 failed. Revisao do diff dos snapshots: so acrescenta a linha (e os novos arquivos de altura). Em 17 linhas o Backlog e o TODO aparecem sem a linha; em 18 ela entra com o Backlog reduzido a uma linha.
- Criteria: A-002, A-003
- Decisions: a ordem de cessao por altura e Backlog vira linha, depois a linha de proximo passo some, depois o Handoff compacta e por ultimo o TODO corta; o teste cobre in_progress e blocked, que sao os estados com Handoff.

## 2026-10-03 - T-003 - Design system e guias descrevem a linha de proximo passo
- Backlog: B-049
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: a secao 10 de `docs/design-system/README.md` ganhou o item "Próximo passo" na estrutura da View e o paragrafo "Próximo passo" com a tabela dos onze casos, as cores (skill em negrito `fg`, ids `id`, resto `meta`), a ausencia na visao Historico, e a regra de altura (o Backlog vira linha, depois a linha some, depois o Handoff compacta, por ultimo o TODO corta) e de largura (`…` ao cortar, ausente abaixo de 40 colunas). `docs/TUI.md` e `docs/TUI.pt-BR.md` mencionam a linha e a ordem de cessao por altura, no mesmo conjunto de mudancas.
- Evidence: `git diff --stat docs`: 3 arquivos (TUI.md, TUI.pt-BR.md, design-system/README.md); a tabela do design system repete a da spec e as frases batem com `src/view/hint.rs`.
- Criteria: A-004
- Decisions: none

## 2026-10-03 - T-004 - Verificacao final local do B-049
- Backlog: B-049
- Spec: .specs/20261002-004-relay-tui-sugestao-de-proximo-passo.md
- Result: a linha de proximo passo esta completa e documentada; todos os criterios da spec estao nomeados por algum registro (A-001 no T-002 do B-048; A-002 e A-003 no T-002 e A-004 no T-003 do B-049).
- Evidence: `cargo test` em `app/relay-tui`: 105 passed, 1 ignored, 0 failed. clippy e rustfmt nao estao instalados neste toolchain e nao rodaram.
- Criteria: none
- Decisions: none

## 2026-10-03 - T-001 - ADR-0010 escrita
- Backlog: B-040
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `docs/adr/0010-navegacao-da-relay-tui-como-estado-local.md` registra a navegacao como estado local da tela, ainda somente leitura, como emenda da decisao 1 da ADR-0009 e dos nao-objetivos de navegacao da spec 20261002-001; fixa Histórico (e nao Trabalho), `Esc` que volta em Histórico, mouse so em Histórico com equivalente de teclado e o parse novo so no Rust.
- Evidence: o arquivo existe com Status, Contexto, Decisao, Consequencias, Compliance e Notes; a ADR-0009 nao foi alterada.
- Criteria: none
- Decisions: a ADR e datada 2026-10-03 (data da escrita); o indice do `AGENTS.md` e o design system ficam para T-002 e T-003.

## 2026-10-03 - T-002 - ADR-0010 listada no indice de ADRs do AGENTS.md
- Backlog: B-040
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: o `AGENTS.md` ganhou a entrada de `docs/adr/0010-navegacao-da-relay-tui-como-estado-local.md` (Accepted), acima da ADR-0009, resumindo a navegacao como estado local, a visao Histórico, o `Esc` que volta e a emenda so da decisao 1 da ADR-0009.
- Evidence: `grep -c 0010-navegacao AGENTS.md` retorna 1.
- Criteria: none
- Decisions: none

## 2026-10-03 - T-003 - Visao Histórico e indice de ADRs registrados
- Backlog: B-040
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: o `AGENTS.md` lista a ADR-0010 no indice de ADRs (T-002) e a secao 10 de `docs/design-system/README.md` ganhou "Visão Histórico": os quatro niveis, a ordem e a linha de cada um, o detalhe, a rolagem so em Histórico, a largura minima (40 colunas) com a linha `Histórico precisa de 40 colunas`, a tabela de teclas e cliques com equivalentes de teclado, a captura do mouse so em Histórico, a atualizacao pelo id e o rodape com o corte por indicacao inteira. A linha "Teclas" da Estrutura passou a citar `Tab` e `r`.
- Evidence: `grep` encontra a entrada `0010-navegacao` no `AGENTS.md` e o titulo `### Visão Histórico` no design system; a ADR-0010 existe em `docs/adr/` (T-001). Os textos de rodape e de linha repetem os da spec.
- Criteria: A-001
- Decisions: `design-system.html` nao foi alterado, porque nenhum token mudou; so o README descreve a visao.

## 2026-10-03 - T-001 - Extracao do changelog e do titulo da spec no core em Rust
- Backlog: B-041
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `app/relay-tui/src/core/history.rs` extrai cada registro do changelog (`TaskRecord`: data, id da tarefa, titulo e os campos Backlog, Spec, Result, Evidence, Criteria e Decisions como `Option`, com as linhas de continuacao juntadas ao campo), mantem um `T-NNN` repetido como registro proprio, e extrai id e titulo de uma spec (`spec_id`, `spec_title`) nas tres formas da spec: apos `AAAAMMDD-NNN - `, o `# ` inteiro, ou o nome do arquivo. `derive.rs`, `RelayState` e a suite de conformidade nao foram tocados.
- Evidence: `cargo test --lib history` em `app/relay-tui`: 7 passed, 0 failed, cobrindo campo com continuacao, registro sem `Criteria`, `T-NNN` repetido, linhas soltas e campo desconhecido, e os tres formatos de titulo de spec.
- Criteria: none
- Decisions: um campo termina em linha em branco, em campo desconhecido ou no proximo cabecalho; a continuacao e junta com um espaco. Campo ausente e `None`.

## 2026-10-03 - T-002 - Backlog por spec e tarefas por item no core em Rust
- Backlog: B-041
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `extract_history(&RelayFiles)` em `core/history.rs` devolve `History`: as specs do mais recente ao mais antigo (id, titulo, itens de backlog na ordem textual e `done()` para a contagem `feitos/total`), o grupo `no_spec` (item sem `spec:` ou cuja `spec:` nao e arquivo de `.specs/`) e todos os registros do changelog, com `tasks_of(backlog_id)` na ordem textual, repetidos incluidos. Nao depende de `derive_state`.
- Evidence: `cargo test --lib history` em `app/relay-tui`: 10 passed, 0 failed, incluindo o agrupamento com spec sem itens (0/0), spec sem titulo, itens orfaos e `T-NNN` repetido em `tasks_of`.
- Criteria: none
- Decisions: `HistoryItem` carrega o marcador cru e `needs`; a disponibilidade fica com o `OkState` do core, para a view nao recalcular dependencias.

## 2026-10-03 - T-003 - Extracao verificada contra os registros reais, core intacto
- Backlog: B-041
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `app/relay-tui/tests/history.rs` roda `extract_history` sobre os registros reais deste repositorio: specs do mais recente ao mais antigo com id e titulo separados, backlog agrupado sob a spec (B-040 e B-041 sob 20261002-002, anotacoes `spec:`/`needs:` fora do texto, cada item em um unico grupo), tarefas de B-040 com todos os campos e `Criteria: A-001` no T-003, e registros antigos sem `Criteria`. Os casos de campo com continuacao, `T-NNN` repetido no mesmo item e spec sem `# ` no formato esperado ficam nos testes de unidade de `history.rs`, porque o changelog real ainda nao tem repeticao dentro de um item.
- Evidence: `cargo test` em `app/relay-tui`: 119 passed, 1 ignored, 0 failed (4 testes novos em `tests/history.rs`, 10 em `core::history`); `tests/conformance.rs` e `tests/purity.rs` passam; `git status` nao lista `derive.rs`, `types.rs`, `integrity.rs` nem `app/conformance/` como alterados.
- Criteria: A-002
- Decisions: o teste real nao afirma repeticao de `T-NNN` dentro de um item, para nao depender de um dado que o repositorio nao tem.

## 2026-10-03 - T-001 - Modulo de navegacao sem terminal nem disco
- Backlog: B-042
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `app/relay-tui/src/nav.rs` define `Nav` (visao Agora/Histórico, nivel specs/itens/tarefas/detalhe, selecao por nivel guardada pela chave da linha) e `handle(Input, &Ctx) -> Effect`. `Tab`/`t` alternam preservando nivel e selecao; `Enter` desce um nivel (nada no detalhe nem em lista vazia); `Esc` e `Backspace` sobem um nivel e, no de specs, voltam a Agora; `Esc` sai so em Agora e `q`/`Ctrl-C` saem de qualquer visao; setas movem a selecao sem passar dos limites; `Reload` e um efeito que nao altera o lugar. As linhas vem de `History` e `OkState`: specs mais novas primeiro e o grupo Sem spec, itens da spec, tarefas do item com os itens do TODO atual sem registro no fim.
- Evidence: `cargo test --test nav` em `app/relay-tui`: 9 passed, 0 failed, com um teste por regra acima (toggle, ordem das specs, descida por nivel, tarefas com `Pending`, voltar por `Esc` e por `Backspace`, saidas, limites das setas, `Reload`, lista vazia).
- Criteria: none
- Decisions: o clique entra como `Input::Click(indice da linha)`, porque quem traduz coordenada em linha e a view (B-043); a rolagem do detalhe e limitada por um maximo que a view informa com `set_detail_max`.

## 2026-10-03 - T-002 - Recarga pelo id, roda, paginas, rolagem do detalhe e clique
- Backlog: B-042
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `Nav::reconcile` mantem cada selecao pela chave da linha quando as listas sao refeitas, leva a selecao a linha mais proxima da mesma lista quando a linha some e, quando some o pai do nivel aberto, sobe ate o primeiro nivel que existe. A roda move a selecao como as setas; `PgUp`/`PgDn` movem pela pagina (`set_page`) e param nas pontas; no detalhe, setas, roda e paginas rolam ate o maximo informado pela view (`set_detail_max`) e sair do detalhe zera a rolagem; `Click(i)` seleciona a linha e abre o nivel seguinte, e e ignorado fora das linhas, no detalhe e em Agora.
- Evidence: `cargo test --test nav` em `app/relay-tui`: 19 passed, 0 failed (10 novos): selecao mantida com linhas novas acima, item sumido, spec sumida com subida ao nivel de specs, tarefa que ganha registro, item que some com o detalhe aberto, historico em estado `inconsistent` sem `ok`, roda, paginas, rolagem do detalhe e cliques.
- Criteria: none
- Decisions: um item do TODO sem registro (`Pending`) que ganha registro deixa de ser achado pela chave e a selecao cai na linha mais proxima, em vez de seguir a tarefa por id entre tipos de linha diferentes.

## 2026-10-03 - T-003 - App liga a navegacao: teclas, roda e `r` que recarrega
- Backlog: B-042
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `App` guarda o `Nav` e o `History` e os refaz a cada leitura do workspace (uma leitura so alimenta o estado derivado e as listas). `key_input` traduz `Tab`/`t`, `Enter`, `Esc`, `Backspace`, setas e `j`/`k`, `PgUp`/`PgDn`, `r` e `q`/`Ctrl-C` (uma letra com Ctrl nao e tecla de navegacao) e a roda do mouse em `Input`; o `Effect` decide: `Quit` sai, `Reload` relê o workspace como depois de um `Settled`, e a selecao e reconciliada pelo id. `Esc` sai so em Agora, `q` e `Ctrl-C` de qualquer visao. `nav.rs` entrou na lista de modulos sem leitura de disco de `tests/read_only.rs`. A tela ainda desenha so Agora e o clique nao e traduzido (B-043 e B-044).
- Evidence: `cargo test` em `app/relay-tui`: 143 passed, 1 ignored, 0 failed, incluindo os testes ponta a ponta em pty e os de conformidade e pureza; 5 testes novos em `src/app.rs`: `Tab`/`t` e `Esc` que volta, `j`/`k` e roda, `r` relê sem evento do watcher mantendo nivel e selecao, navegar e recarregar nao alteram nenhum registro (conteudo identico antes e depois), Ctrl+letra ignorada. `git status` nao lista `derive.rs`, `types.rs`, `integrity.rs` nem `app/conformance/`.
- Criteria: A-003
- Decisions: a letra maiuscula equivale a minuscula nas teclas de navegacao (como ja era em `q`); `Backspace` em Agora nao faz nada.

## 2026-10-03 - T-001 - Visao Histórico desenhada: listas, rolagem e detalhe
- Backlog: B-043
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `app/relay-tui/src/view/history.rs` desenha o cabecalho e um cartao com o nivel aberto do `Nav`. Specs: id, titulo e `feitos/total`, mais a linha Sem spec; itens: id, texto e estado com o marcador e a palavra de Agora (disponivel e aguardando vem do `available` do core; sem estado derivado, `pendente`); tarefas: id, titulo e data, e os itens do TODO sem registro com `sem registro`. Cada linha ocupa uma linha e termina em `…`; a janela de rolagem mantem a selecao visivel e o cartao diz `↑ N acima · ↓ M abaixo` (so os numeros quando as palavras nao cabem). O detalhe quebra o titulo, a data e os campos pela largura sem cortar, omite o campo ausente e rola ate o fim; sem registro mostra `Sem registro no changelog ainda.`. Abaixo de 40 colunas aparece so o aviso `Histórico precisa de 40 colunas`, abaixo de 6 linhas so cabecalho e rodape, e sem specs `Nenhuma spec em .specs/`. `App::render` escolhe a visao aberta e `App::fit` informa ao `Nav` a pagina e o limite de rolagem do detalhe.
- Evidence: `cargo test` em `app/relay-tui`: todos os alvos `ok`, 0 failed; `tests/history_view.rs` com 9 testes sobre o texto desenhado: linhas de spec, corte com `…` com a contagem alinhada a direita em 58 e 40 colunas, marcadores dos itens, tarefas com data, rolagem de 30 itens com indicacao de acima e abaixo, detalhe quebrado sem `…` e sem campo inventado, rolagem do detalhe ate `Decisions`, aviso em 30 colunas e lista vazia. O rodape continua o de antes; muda em T-002 e os snapshots vem em T-003.
- Criteria: none
- Decisions: em 30 colunas a frase `Histórico precisa de 40 colunas` (31 colunas) quebra por palavras em duas linhas em vez de terminar em `…`; a janela da lista e sem estado (a selecao fica no meio quando ha folga), o que dispensa guardar o inicio da janela no `Nav`.

## 2026-10-03 - T-002 - Rodape de Agora e de cada nivel de Histórico
- Backlog: B-043
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `view/mod.rs` troca o rodape fixo `q sair` por indicacoes (`Hint` com tecla, rotulo e ordem de corte) e tres rodapes: Agora `Tab histórico · r recarregar · q sair`, listas de Histórico `↑↓ mover · Enter abrir · Esc voltar · Tab agora · r recarregar · q sair` e detalhe `↑↓ rolar · Esc voltar · Tab agora · r recarregar · q sair` (sem `Enter`). Quando nao cabem, saem indicacoes inteiras da menos para a mais importante (`r recarregar`, `↑↓`, `Enter abrir`, `Esc voltar`, `Tab`) dentro da largura util (largura - 4, a dos cartoes); `q sair` fica sempre. `HistoryScreen` escolhe o rodape do nivel aberto e o `App` desenha a visao aberta.
- Evidence: `cargo test --test history_view`: 13 passed, 0 failed (4 novos): textos inteiros em 80 colunas, cortes em 58 (`Enter abrir · Esc voltar · Tab agora · q sair`), 40 (`Esc voltar · Tab agora · q sair`) e 30 colunas, Agora a 40 colunas com `Tab histórico · q sair`, o detalhe sem `Enter`, e `q sair` presente ate em 7 colunas; `cargo test --lib app`: 18 passed, incluindo o teste de que a visao aberta e a desenhada. Os snapshots de Agora ainda mostram o rodape antigo e sao regravados em T-003.
- Criteria: none
- Decisions: a frase inteira do rodape de Agora tem 37 colunas e a spec manda mostrar `Tab histórico · q sair` em 40; por isso o limite do rodape e a largura util dos cartoes (largura - 4) e nao a largura da tela.

## 2026-10-03 - T-003 - Snapshots de cada nivel de Histórico e do rodape de Agora
- Backlog: B-043
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `tests/history_snapshots.rs` grava 23 snapshots (texto e mapa de cor) de um workspace montado a mao: specs em 58 e 40 colunas (e com a segunda selecionada), itens com um item em cada estado (feito, em curso, aguardando, disponivel, bloqueado) e o grupo Sem spec, tarefas de um item com correcao (`T-002` repetido) e do item atual com `sem registro`, detalhe com todos os campos quebrado em 58 e 40 colunas e o de um item do TODO sem registro, lista de 31 itens no inicio, no meio e no fim (`↑ N acima · ↓ M abaixo`, so os numeros em 40 colunas), detalhe em 40 colunas e 12 linhas no inicio e no fim, o aviso em 30 colunas, alturas 5 e 6, workspace vazio e spec sem itens (`0/0`). Os snapshots de Agora foram regravados: so a linha do rodape mudou (`Tab histórico · r recarregar · q sair`, e `Tab histórico · q sair` em 40 colunas). A secao 10 do design system descreve o novo rodape de Agora.
- Evidence: `cargo test` em `app/relay-tui`: todos os alvos `ok`, 0 failed (`history_snapshots` 7, `history_view` 13, `view_snapshots` 9, `app` 59 na lib); revisei o texto de cada snapshot novo e o mapa de cor de `hist-items-58` (selecao `▸` em `U`, ids em `i`, estado em tom e negrito); os cortes com `…` aparecem em 58 e 40 colunas, o detalhe nao tem `…` e a indicacao de acima e abaixo aparece na lista e no detalhe maiores que a tela. `git status` nao lista `derive.rs`, `types.rs`, `integrity.rs` nem `app/conformance/`.
- Criteria: A-004
- Decisions: o aviso de 30 colunas e gravado quebrado em duas linhas (`Histórico precisa de 40` / `colunas`), porque a frase tem 31 colunas; o teste `view_semantics` ja tinha um aviso de funcao `card_rows` sem uso, que nao e desta mudanca.

## 2026-10-03 - T-001 - Clique na tela vira linha da lista
- Backlog: B-044
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `view::row_at` traduz a coordenada de um clique no indice da linha de lista que esta desenhada ali, pela mesma janela de rolagem do desenho; cabecalho, rodape, moldura do cartao, area vazia abaixo da ultima linha e qualquer ponto do detalhe nao sao linhas. O `App` guarda o tamanho do ultimo `fit` e, com Histórico aberto, entrega `Input::Click(indice)` ao `Nav` para o botao esquerdo; em Agora e com outro botao o clique e ignorado.
- Evidence: `cargo test --lib app` em `app/relay-tui`: 21 passed, 0 failed (3 novos): clique na linha seleciona e abre o nivel seguinte; clique em cabecalho, borda de cima, bordas laterais, area vazia, rodape, fora da tela, em Agora e com o botao direito e ignorado; numa lista de 30 itens rolada o clique mapeia pela janela (B-017 e B-021) e no detalhe nada acontece.
- Criteria: none
- Decisions: um clique so conta no botao esquerdo, ao apertar (`Down`); soltar o botao nao faz nada.

## 2026-10-03 - T-002 - Captura do mouse so com Histórico aberto
- Backlog: B-044
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `App::wants_mouse()` e verdadeiro so com Histórico aberto, e `app::run_with_mouse` chama `set_mouse(bool)` quando esse desejo muda (`run` segue igual, com um `set_mouse` vazio). O `main.rs` liga e desliga a captura (`EnableMouseCapture`/`DisableMouseCapture`) por esse gancho, desliga de novo antes de `ratatui::restore()` ao sair e instala um hook de panic que desliga a captura e entao chama o hook de restauracao do ratatui. Em builds de debug, `RELAY_TUI_TEST_PANIC=history` abre Histórico, liga a captura e entra em panico, para o teste em pty provar a restauracao.
- Evidence: `cargo test` em `app/relay-tui`: todos os alvos `ok`, 0 failed (63 na lib); o teste `the_mouse_is_wanted_only_while_historico_is_open` dirige o laco com eventos um a um e ve `[ligar, desligar, ligar, desligar]` para `Tab`, `Esc` (do nivel de specs a Agora), `Tab`, `Tab`. A restauracao no binario real (sair e panic) e o teste em pty de T-003.
- Criteria: none
- Decisions: quem fornece `set_mouse` e quem desliga a captura no fim do laco; assim a restauracao no `main.rs` cobre a saida por `q` e `Ctrl-C` em Histórico mesmo quando o laco acaba num lote de eventos.

## 2026-10-03 - T-003 - Teste em pty da captura do mouse e documentacao de Histórico
- Backlog: B-044
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: o binario real, lido por um emulador de terminal, liga a captura do mouse com `Tab`, abre o nivel seguinte com um clique SGR numa linha da lista, e a desliga ao voltar a Agora (`Esc` do nivel de specs e `Tab`/`t`), ao sair de Histórico por `q` ou `Ctrl-C` e num panic em Histórico; o terminal sai restaurado (tela normal, cursor visivel, mouse nao reportado) e o workspace nao muda. O README do app, `docs/TUI.md` e `docs/TUI.pt-BR.md` descrevem a visao Histórico, os quatro niveis, a tabela de teclas e cliques (`Esc` volta em vez de sair nela, `r` recarrega, `q` e `Ctrl-C` saem de qualquer visao), a captura do mouse so em Histórico e o aviso abaixo de 40 colunas.
- Evidence: `cargo test --test e2e_pty` em `app/relay-tui`: 15 passed, 1 ignored, 0 failed, com 5 testes novos (Tab liga o mouse e o clique em B-002 abre `Tarefas · B-002`; clique no cabecalho e no rodape nao faz nada; voltar a Agora desliga; `q` e `Ctrl-C` em Histórico desligam e restauram; `RELAY_TUI_TEST_PANIC=history` restaura e desliga antes da mensagem; navegar por teclas e mouse deixa o conteudo do workspace identico). Tirando o `DisableMouseCapture` do hook de panic o teste de panic falha com `the mouse was left reported`, e com ele volta a passar. `cargo test` completo: todos os alvos `ok`, 0 failed.
- Criteria: A-005, A-006
- Decisions: o aviso `muitos terminais pedem Shift (Option no iTerm2) para selecionar com a captura ligada` esta na documentacao como fato geral de terminal, sem prometer um comportamento por terminal; a lista de teclas do README do app esta em portugues, como o resto dele.

## 2026-10-03 - T-001 - Script open-split.sh
- Backlog: B-045
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: `skills/relay-tui-split/scripts/open-split.sh [--dry-run] [DIR]` e um script POSIX `sh` (sem arrays, `local`, `[[` nem `echo -e`) que confere `relay-tui` no `PATH` (sem ele imprime o link do guia no GitHub e sai com 1), resolve o diretorio para um caminho absoluto, detecta o terminal pela primeira variavel que casa (`TMUX`, `ZELLIJ`, `WEZTERM_PANE`, `KITTY_WINDOW_ID`, `TERM_PROGRAM=iTerm.app`, `TERM_PROGRAM=WarpTerminal`) e abre um painel a direita com `relay-tui --workspace '<dir>'`: `tmux split-window -h`, `zellij action new-pane --direction right --`, `wezterm cli split-pane --right --pane-id`, `kitten @ launch --location=vsplit`, AppleScript do iTerm2 (na sessao de `ITERM_SESSION_ID`, `split vertically with default profile` e `write text`) e AppleScript do Warp via System Events (confere o app em primeiro plano, `Cmd+D`, digita o comando e Enter). `--dry-run` imprime o comando e sai com 0 sem executar; terminal nao reconhecido ou split que falha imprime a instrucao manual do terminal e sai com 2; nunca tenta outro terminal. Nao le nem escreve registro e nao instala nada.
- Evidence: rodado a mao em `sh` e em `dash` com `env -i` para cada terminal em `--dry-run`: a saida de cada um e a da tabela, o caminho `/tmp/dir with space/it's` sai citado e correto no tmux (`"relay-tui --workspace '/tmp/dir with space/it'\\''s'"`) e no kitty, sem `relay-tui` no `PATH` sai 1 com o link e sem terminal reconhecido sai 2 com a instrucao; os dois scripts AppleScript impressos compilam com `osacompile`. Os testes automatizados sao T-002.
- Criteria: none
- Decisions: o dry-run de iTerm2 e Warp imprime `osascript <<'APPLESCRIPT' ... APPLESCRIPT` (o script real, que e o que vai para o `osascript`) em vez de `-e` repetido; sem `ITERM_SESSION_ID` o iTerm2 divide a sessao corrente da janela corrente; o Warp e reconhecido pelo bundle id `dev.warp.` e nao pelo nome do processo; uma opcao desconhecida ou um diretorio que nao existe sai com 2.

## 2026-10-03 - T-002 - Testes do script por terminal, em sh e em dash
- Backlog: B-045
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: `.agents/tests/open-split.test.sh` (ferramenta do repositorio, fora de `skills/`) roda o script sob `sh` e `dash` com `env -i` e um `PATH` que so tem terminais falsos que registram a chamada. Para cada terminal da tabela (tmux, zellij, WezTerm, kitty, iTerm2, Warp) compara a saida de `--dry-run`, com `--workspace` absoluto e citado para um caminho com espaco, e confere que o dry-run nao executa nada; confere a precedencia (tmux dentro do iTerm2 e tmux, e cada multiplexador ou emulador vence os que vem depois); mostra que o caminho com apostrofo sobrevive aos dois shells; executa de verdade cada acao pelos falsos e compara o `argv` (e o script AppleScript recebido pelo `osascript`); falha do split imprime a instrucao daquele terminal, sai com 2 e nao tenta outro; terminal nao reconhecido sai com 2; sem `relay-tui` no `PATH` sai com 1 com o link do guia, tambem em `--dry-run`, sem instalar nada; opcao desconhecida, diretorio inexistente e dois diretorios saem com 2; nenhum registro e citado no script e nenhum arquivo de teste fica dentro de `skills/`.
- Evidence: `sh .agents/tests/open-split.test.sh`: `145 passed, 0 failed` (a suite inteira em `sh` e em `dash`). Tres mutacoes do script derrubam o teste e o script restaurado e identico ao original: sem o `-h` do tmux (14 falhas), iTerm2 antes do tmux na ordem (6) e caminho sem aspas (24).
- Criteria: A-001, A-002
- Decisions: o teste de A-005 (nenhum registro nomeado) ja cobre `skills/relay-tui-split/SKILL.md`, que ainda nao existe; o criterio so e nomeado quando a skill existir (B-046). A verificacao nos terminais reais (A-004) e do B-047.

## 2026-10-03 - T-001 - Skill relay-tui-split, links por item e linha do AGENTS.md
- Backlog: B-046
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: `skills/relay-tui-split/SKILL.md` (30 linhas, em ingles, neutra de harness) diz que a skill esta fora do protocolo e nao le nem escreve registro, manda rodar `scripts/open-split.sh` a partir do diretorio da skill com o repositorio do usuario como unico argumento (e `--dry-run` so quando a pessoa pergunta o que ele faria) e relatar o que o script imprimiu por codigo de saida (0 painel aberto, 1 sem `relay-tui` no `PATH` com o link, 2 terminal nao suportado ou split falhou com a instrucao manual), sem instalar o binario, sem tentar outro terminal e sem gerenciar o painel depois. Os links relativos `.agents/skills/relay-tui-split` e `.claude/skills/relay-tui-split` apontam para `../../skills/relay-tui-split`, como os das demais skills (ADR-0002), e a skill aparece na lista de skills da sessao. A linha das skills do `AGENTS.md` deixa de dizer "all five".
- Evidence: `wc -l skills/relay-tui-split/SKILL.md`: 30; `ls .claude/skills/relay-tui-split/` mostra `SKILL.md` e `scripts` pelo link; `sh .agents/tests/open-split.test.sh`: `145 passed, 0 failed`, e o teste de que nenhum registro e citado agora inspeciona a `SKILL.md` existente alem do script.
- Criteria: none
- Decisions: o nome do diretorio de scripts na `SKILL.md` e relativo a skill, porque o caminho absoluto muda por harness e por instalacao; a documentacao e os lacos de instalacao ficam em T-002 e T-003.

## 2026-10-03 - T-002 - Lacos de instalacao e tabela de skills com relay-tui-split
- Backlog: B-046
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: os quatro lacos de `docs/INSTALL.md` e os quatro de `docs/INSTALL.pt-BR.md` (Claude Code, Codex, OpenCode global e OpenCode local) passaram a incluir `relay-tui-split`, com os blocos de codigo identicos nas duas linguas; a tabela de skills de `README.md` e `README.pt-BR.md` ganhou a linha `relay-tui-split` com a nota de que ela fica fora do protocolo e nao toca registro. O `.opencode/INSTALL.md`, que listava so quatro skills, passou a listar tambem `relay-continue` (que faltava) e `relay-tui-split`.
- Evidence: `rtk proxy grep -c relay-tui-split`: 4 em `docs/INSTALL.md`, 4 em `docs/INSTALL.pt-BR.md`, 1 em cada README e 2 em `.opencode/INSTALL.md`; `git diff --stat` dos cinco arquivos: 14 insercoes e 9 remocoes (so as linhas dos lacos, as linhas da tabela e as do adaptador do OpenCode).
- Criteria: none
- Decisions: alinhar o `.opencode/INSTALL.md` e incluir o `relay-continue` que faltava e uma correcao de uma divergencia ja existente, feita aqui para manter a orientacao de instalacao igual nos tres harnesses (AGENTS.md, "Keep installation guidance aligned").

## 2026-10-03 - T-003 - Secao de split dos guias do relay-tui com a skill e as permissoes
- Backlog: B-046
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: a secao "Open it in a split" de `docs/TUI.md` e "Abrir num split" de `docs/TUI.pt-BR.md` agora comecam pela skill (`/relay-tui-split` no Claude Code; "Use relay-tui-split" no Codex e no OpenCode), explicam que ela e do pacote, fora do protocolo, nao toca registro nem instala nada, listam a deteccao e a acao de cada terminal (tmux, zellij, WezTerm, kitty, iTerm2, Warp) com a ordem de precedencia, dizem as permissoes do macOS (Automacao para o iTerm2, Acessibilidade para o Warp, que so recebe as teclas se for o app em primeiro plano), descrevem o script, `--dry-run` e os codigos de saida 0, 1 e 2, e mantem a abertura manual como alternativa. Nao afirmam que Codex e OpenCode repassam o ambiente do terminal: isso e a verificacao do B-047.
- Evidence: `rtk proxy grep -c relay-tui-split` retorna 3 em cada guia; as duas secoes tem as mesmas tabelas e os mesmos comandos. Para A-003: a `SKILL.md` tem 30 linhas, os links `.agents/skills/relay-tui-split` e `.claude/skills/relay-tui-split` existem, e a skill consta nos quatro lacos de `docs/INSTALL.md` e de `docs/INSTALL.pt-BR.md`, na tabela de `README.md` e `README.pt-BR.md` e nas secoes de split dos dois guias (T-001, T-002 e este registro). Para A-005: `grep` por `.orchestration`, `.specs`, `BACKLOG.md`, `TODO.md`, `HANDOFF.md` e `CHANGELOG.md` em `SKILL.md` e `open-split.sh` nao acha nada, e `sh .agents/tests/open-split.test.sh` repete essa inspecao e da `145 passed, 0 failed`.
- Criteria: A-003, A-005
- Decisions: A-004 (verificacao com o harness real em cada terminal) continua do B-047; a documentacao so promete o que o script faz e o que os testes cobrem.

## 2026-10-03 - T-001 - Ambiente do Claude Code no Warp e tmux real
- Backlog: B-047
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: Claude Code + Warp, so deteccao: o shell que o Claude Code usa nesta sessao herda o ambiente do Warp (`TERM_PROGRAM=WarpTerminal`, `TERM_PROGRAM_VERSION=v0.2026.09.16.08.27.stable_02`, `__CFBundleIdentifier=dev.warp.Warp-Stable`), entao a hipotese da spec (o ambiente do terminal chega ao shell do harness) se confirma para o Claude Code. O script rodado dai com `--dry-run` e `relay-tui` no `PATH` escolhe o Warp e imprime o AppleScript do Warp (`Cmd+D`, depois digita `relay-tui --workspace '/Users/fabiano/Developer/relay'`), saida 0. Script + tmux 3.6b, de verdade: num servidor tmux isolado (`tmux -L relay-b047 -f /dev/null`, que nao toca as sessoes da pessoa) o script dentro de um painel abriu o painel a direita: `Opened relay-tui in a tmux split, watching /Users/fabiano/Developer/relay`, `EXIT=0`, e o painel novo mostra o `relay-tui` desenhando o handoff deste trabalho.
- Evidence: comando 1: `PATH=<repo>/app/relay-tui/target/release:$PATH sh skills/relay-tui-split/scripts/open-split.sh --dry-run` no shell do Claude Code: `TERM_PROGRAM=WarpTerminal TMUX=unset`, exit 0, AppleScript do Warp. Comando 2: `tmux -L relay-b047 -f /dev/null new-session -d -s b047 -x 200 -y 40 sh`, `send-keys` do script e `list-panes -F`: painel 0 `start=[sh]` 100x40 e painel 1 `start=["relay-tui --workspace '/Users/fabiano/Developer/relay'"] cmd=relay-tui` 99x40; `capture-pane` do painel 1 mostra `relay`, `● atualizado` e o cartao `Handoff` com `B-047 · T-001`. O servidor isolado foi encerrado (`no server running`).
- Criteria: none
- Decisions: o tmux foi verificado num servidor proprio (`-L relay-b047`) para nao dividir nenhuma sessao da pessoa; o painel do tmux que roda o script faz o papel do shell do harness, e o harness de verdade no tmux e o T-002.

## 2026-10-03 - T-002 - Claude Code, Codex e OpenCode no tmux
- Backlog: B-047
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: cada harness rodou de verdade dentro de um painel de um servidor tmux isolado (`tmux -L relay-b047 -f /dev/null`), num repositorio descartavel com `.claude/skills`, `.agents/skills` e `.opencode/skills` apontando para `skills/relay-tui-split`, com o `relay-tui` do `target/release` no `PATH`, e recebeu um pedido curto para usar a skill. Claude Code 2.1.288 (`claude -p --model haiku --allowedTools Bash Skill Read`): abriu o painel e relatou o resultado. OpenCode 1.18.32 (`opencode run`): carregou a skill pela ferramenta nativa (`→ Skill "relay-tui-split"`), rodou o script (`./scripts/open-split.sh`) e o painel abriu. Codex 0.156.1 (`codex exec --skip-git-repo-check`): a sandbox padrao NAO deixa o script falar com o tmux (`error connecting to /private/tmp/tmux-501/relay-b047 (Operation not permitted)`); o script tratou como split que falhou, imprimiu a instrucao manual, saiu com 2 e o Codex a repassou; com `-s danger-full-access` o mesmo pedido abriu o painel. O tmux foi detectado nos tres (o `TMUX` do painel chegou ao shell de cada harness, inclusive na sandbox do Codex).
- Evidence: `tmux list-panes -F` apos cada execucao: Claude Code, Codex com `danger-full-access` e OpenCode mostram dois paineis, o novo com `start=["relay-tui --workspace '<repositorio descartavel>'"] cmd=relay-tui` 99x40; o `capture-pane` do painel do Claude Code mostra `relay`, `● atualizado` e `Sem trabalho`. Codex com a sandbox padrao: so um painel (`sleep`), saida `Could not open the tmux split.` e a instrucao manual com `relay-tui --workspace '...'`. Os servidores isolados foram encerrados e nao sobrou nenhum processo `relay-tui` dos testes (so existe um `target/debug/relay-tui --workspace /Users/fabiano/Developer/relay`, de outra origem, que nao foi tocado).
- Criteria: none
- Decisions: a limitacao do Codex vai para `docs/TUI.md` e `docs/TUI.pt-BR.md` no T-003, como o A-004 manda; uma primeira tentativa do Claude Code com `env -i` falhou com `Not logged in`, defeito do ambiente do teste (a autenticacao precisa do ambiente do usuario) e nao da skill, e foi refeita desfazendo so as variaveis `CLAUDE*`, `WARP*`, `TERM_PROGRAM*`.

## 2026-10-03 - T-003 - Bug do AppleScript do Warp corrigido pela execucao real, guarda verificada e limitacao do Codex documentada
- Backlog: B-047
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: ao rodar o script de verdade no shell do Claude Code no Warp, o AppleScript do Warp falhou na hora com `Não é possível ajustar insertion point 1 a application process 1 whose frontmost = true. (-10006)`: `front` e palavra reservada do AppleScript, e o script so tinha sido compilado, nao executado. A variavel virou `frontApp` em `open-split.sh`. Com isso a guarda do Warp foi verificada de verdade: com o Arc (`company.thebrowser.Browser`) em primeiro plano o script parou com `Warp is not the frontmost application (-2700)`, imprimiu a instrucao manual do Warp, saiu com 2 e nao digitou nada em outro aplicativo; o trecho da guarda sozinho responde `guard: not Warp (company.thebrowser.Browser)`. `docs/TUI.md` e `docs/TUI.pt-BR.md` ganharam a nota das sandboxes dos harnesses: Claude Code e OpenCode abrem o painel no tmux, a sandbox padrao do Codex bloqueia o socket do tmux (o script cai na instrucao manual com saida 2) e o painel abre fora da sandbox, e as variaveis do terminal chegam ao shell do harness em todos os casos verificados.
- Evidence: saida real antes da correcao (`execution error ... (-10006)`, `Could not open the warp split.`, exit 2) e depois (`execution error: Warp is not the frontmost application (-2700)`, exit 2); `sh .agents/tests/open-split.test.sh`: `145 passed, 0 failed` com o script corrigido; `rtk proxy grep -c sandbox` retorna 3 em cada guia. O link temporario `~/.local/bin/relay-tui` que criei para o teste foi removido, e os arquivos de apoio em `/tmp` tambem.
- Criteria: none
- Decisions: nao trouxe o Warp para o primeiro plano nem simulei teclas na janela da pessoa (ela recusou essa acao); por isso o `Cmd+D` do Warp e o iTerm2 continuam sem execucao real e viram o T-004, bloqueado.

## 2026-10-03 - T-004 - iTerm2 verificado de verdade e Warp executado ate a permissao de Acessibilidade
- Backlog: B-047
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: com a autorizacao explicita da pessoa ("prefiro que rode"), o script rodou de verdade nos dois terminais. iTerm2: o painel abriu. Rodado dentro de uma janela de teste propria do iTerm2 (aberta por AppleScript, cuja sessao tem `ITERM_SESSION_ID`), o script imprimiu `Opened relay-tui in a iterm2 split, watching /Users/fabiano/Developer/relay` e saiu com 0; a aba passou a ter 2 sessoes e a segunda mostra o `relay-tui` desenhando o handoff deste trabalho (`B-047 · T-004`, `● atualizado`); nenhuma permissao extra foi pedida. A janela de teste foi fechada e o processo de teste terminou. Warp: com o Warp em primeiro plano (`dev.warp.Warp-Stable`, trazido por `open -a Warp`) a guarda passou e o Cmd+D foi tentado, mas o macOS negou o envio de teclas ao `osascript` (`osascript não tem permissão para acionar teclas. (1002)`): o script imprimiu a instrucao manual do Warp (Cmd+D e a permissao de Acessibilidade) e saiu com 2, nada foi digitado e nenhum painel abriu. Falta conceder a Acessibilidade e repetir.
- Evidence: iTerm2: saida do script `Opened relay-tui in a iterm2 split ...` com `EXIT=0`; `count of sessions of current tab` = 2; `text of session 2` com o cartao `Handoff` e `B-047 · T-004`; `pgrep` mostrou um `relay-tui --workspace /Users/fabiano/Developer/relay` novo (e nenhum depois de fechar a janela). Warp: `frontmost: dev.warp.Warp-Stable`, erro `1002`, `Could not open the warp split.`, exit 2, e o `pgrep` sem processo novo. O link temporario `~/.local/bin/relay-tui` e a saida de apoio foram removidos.
- Criteria: none
- Decisions: nao concedi nem contornei a permissao de Acessibilidade, que e uma escolha da pessoa em Ajustes do Sistema; por isso o Warp vira o T-005, bloqueado ate ela conceder a permissao ao Warp (o processo que esta na frente) e pedir que eu repita. A permissao negada tambem e um resultado verificado: o script cai na instrucao manual e nao digita nada.

## 2026-10-03 - T-005 - Warp com a Acessibilidade concedida: split que executa a linha sozinho
- Backlog: B-047
- Spec: .specs/20261002-003-skill-relay-tui-abre-em-split.md
- Result: com a Acessibilidade concedida (`AXIsProcessTrusted()` passou a `true`) e o Warp em primeiro plano, o script abriu o split do Warp (`Cmd+D` funcionou), mas a primeira versao so deixou o texto `relay-tui --workspace '/Users/fabiano/Developer/relay'` digitado no painel novo, sem executar; a pessoa viu e rodou a linha a mao, e o `relay-tui` abriu certo. O `key code 36` (Enter) logo apos a digitacao nao foi aceito pelo Warp; o script do Warp passou a esperar 0,4 s e enviar `keystroke return`. Com a correcao, o mesmo comando abriu o split e executou a linha sozinho: o `relay-tui` ficou rodando no painel novo.
- Evidence: antes: `Opened relay-tui in a warp split` com exit 0 e nenhum processo `relay-tui --workspace` depois de 4 s (o shell novo existia, sem filho), e o relato da pessoa de que a linha estava digitada e nao executada. Depois da correcao: `Opened relay-tui in a warp split, watching /Users/fabiano/Developer/relay`, exit 0 e `ps` mostra `35510 ttys003 00:04 relay-tui --workspace /Users/fabiano/Developer/relay`, num tty novo. `sh .agents/tests/open-split.test.sh`: `145 passed, 0 failed` (o teste do Warp agora exige `keystroke return`). O link temporario `~/.local/bin/relay-tui` foi removido. A-004 reunido: tmux com Claude Code, Codex (so fora da sandbox) e OpenCode (T-001 e T-002), iTerm2 (T-004) e Warp (este registro).
- Criteria: A-004
- Decisions: no iTerm2 o script rodou dentro de uma sessao propria (com `ITERM_SESSION_ID`) e nao com o Claude Code aberto la; no Warp foi o proprio Claude Code, pela skill, que rodou o script; o que A-004 pede de fundamental (o ambiente do terminal chega ao shell do harness e o painel abre) fica verificado em tmux e Warp com o harness, e no iTerm2 pelo script. Os splits de teste ficaram abertos na janela do Warp da pessoa (varios com o texto digitado e um rodando o `relay-tui`) para ela fechar.

## 2026-10-03 - T-001 - Agrupamento do backlog por spec, puro, e o historico no View
- Backlog: B-050
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: `app/relay-tui/src/view/specs.rs` agrupa o backlog por spec, so com os dados que o core ja entrega (marcador e `available` de `OkState`, e os arquivos e titulos de spec do `History`): a spec atual e a do item ativo do TODO (`em curso`) ou, sem item ativo, a do primeiro item disponivel em ordem textual (`a seguir`, sem afirmar prioridade); ela traz os itens nao feitos na ordem do backlog e `feitos/total`; as specs pendentes seguem a ordem em que aparecem no `BACKLOG.md`, so com as que tem item nao feito, e os itens sem spec valida formam a linha Sem spec; sem item que ancore a spec atual (backlog sem disponivel, ou item ativo sem spec valida) ela nao existe e todas as pendentes vao para a lista. `View` ganhou o campo `history` (e `view::no_history()` para telas sem specs); o `App` o preenche e os sete pontos de teste que construiam `View` foram atualizados.
- Evidence: `cargo test --lib specs` em `app/relay-tui`: 6 passed, 0 failed (spec do item ativo com feitos so na contagem, spec do primeiro disponivel com as outras na ordem do backlog, nenhum disponivel sem spec atual, Sem spec, item ativo sem spec valida, spec toda feita fora das pendentes). Os cartoes sao T-002.
- Criteria: none
- Decisions: a escolha da spec atual usa o `available` do core e nunca recalcula dependencias, como manda a spec; um item ativo cuja spec nao e um arquivo de `.specs/` nao ancora cartao nenhum.

## 2026-10-03 - T-002 - Cartoes Spec atual e Specs pendentes e a ordem de ceder por altura
- Backlog: B-050
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: o cartao Backlog de Agora foi substituido por dois cartoes em `in_progress`, `blocked`, `ready` e `backlog`. Spec atual: na borda `AAAAMMDD-NNN` e o titulo da spec (cortado com `…` antes do lado direito), a direita `em curso` (verde) ou `a seguir` (azul, sem item ativo) e `feitos/total`; dentro, os itens nao feitos na ordem do backlog com marcador e palavra a direita (`em curso`, `disponível`, `aguardando · após B-NNN`, `bloqueado`), e os feitos so na contagem. Specs pendentes: uma linha por spec (id, titulo e `feitos/total`), Sem spec por ultimo. Em `backlog` a linha `● A escolher  Sem handoff ativo` fica no lugar do Handoff e os dois cartoes no lugar do cartao `A escolher`. A altura cede nesta ordem: Specs pendentes vira `N specs pendentes`, Spec atual corta em `+N itens` (sempre com ao menos um item), e os dois viram a linha de contagem do Backlog de antes; so entao o Handoff compacta e o TODO corta. A linha de proximo passo continua depois do Backlog virar linha. `done`, `idle`, `inconsistent` e nao-workspace nao mudaram. O codigo morto do cartao Backlog antigo foi removido.
- Evidence: `cargo test --test agora_specs` em `app/relay-tui`: 6 passed, 0 failed, num workspace montado a mao (spec toda feita, spec com um item bloqueado, a atual com todos os estados, uma com um item, e um item sem spec): o titulo e a contagem do cartao, os itens com a palavra e `após B-041`, o feito fora da lista, a ordem das pendentes e Sem spec por ultimo, `a seguir` sem item ativo, o corte com `…` em 58 e 40 colunas, a sequencia 0 (dois cartoes), 1 (linha de pendentes), 2 (atual cortada) e 3 (linha de contagem) sem nunca voltar atras entre as alturas 44 e 14, e o TODO e o Handoff intactos ate chegar a 3. Os testes de unidade de `specs` seguem em 6 passed. Os snapshots e os testes antigos que citam o cartao Backlog sao regravados e ajustados em T-003.
- Criteria: none
- Decisions: o corte do cartao Spec atual nao deixa um cartao so com `+N itens`: abaixo de frame, um item e a linha de corte (4 linhas, mais a de pendentes) ele vira a linha de contagem; o `em curso`/`a seguir` fica na borda, antes do `feitos/total`.

## 2026-10-03 - T-003 - Snapshots da visao Agora agrupada por spec e documentacao
- Backlog: B-050
- Spec: .specs/20261002-002-relay-tui-navegacao-pelo-historico.md
- Result: a visao Agora mostra o cartao Spec atual (id e titulo na borda, `feitos/total`, itens nao feitos com marcador e palavra) e, abaixo, o cartao Specs pendentes (id, titulo e `feitos/total` por spec, e a linha Sem spec), em `in_progress`, `blocked`, `ready` e `backlog` (este com `a seguir` e a spec do primeiro item disponivel, e sem o cartao Spec atual quando nao ha entrada disponivel). 16 snapshots novos (`agora-specs-*`, em 58 e 40 colunas e de altura reduzida) mostram cada passo da ordem de ceder (dois cartoes, pendentes em uma linha, atual cortada em `+N itens`, linha de contagem), e os snapshots de `done`, `idle`, `inconsistent` e nao-workspace seguem sem cartoes de spec. As frases longas do proximo passo ganharam uma forma curta para a skill nunca ser o que se corta (`Próximo item: B-NNN (título). Comece com relay-session.`, `Nenhum item disponível: relay-continue.`, `Registros em conflito: relay-status e relay-continue.`). O design system (secao 10: estrutura, "Agrupado por spec", altura, proximo passo), `docs/TUI.md` e `docs/TUI.pt-BR.md` descrevem os dois cartoes no lugar do cartao Backlog; os testes em pty que dependiam do texto do cartao antigo passaram a criar o arquivo de spec.
- Evidence: `cargo test` em `app/relay-tui`: todos os alvos `ok`, 193 passed, 0 failed (`agora_specs` 9, `view_snapshots` 9, `e2e_pty` 16, lib 71); `sh .agents/tests/open-split.test.sh`: `145 passed, 0 failed`; `grep` por `Specs pendentes`, `a seguir` e `em curso ·` nos snapshots de `done`, `idle`, `inconsistent` e `not-relay` nao acha nada. A escolha da spec atual usa o `available` e os marcadores do core (`specs.rs` nao recalcula dependencias). Revisei o texto dos snapshots novos nas quatro etapas de altura e o do estado `backlog` sem item disponivel.
- Criteria: A-007
- Decisions: o cartao Spec atual so aparece se o item ancora tem spec valida (um arquivo de `.specs/`); o Sem spec vem sempre por ultimo, como no nivel de specs de Histórico; a forma curta das frases e usada so quando a longa nao cabe, e nenhuma frase longa (a mensagem da tabela da spec 004) mudou para quem tem largura.
