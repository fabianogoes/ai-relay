# ADR-0003 — Contrato do estado derivado

## Status

**Accepted** — 2026-09-07.

## Contexto

O `relay-core` vai interpretar os cinco registros e produzir um estado; a
`relay-ui` vai desenhá-lo. Entre os dois existe um formato, e ele ainda não
existe em lugar nenhum — nem como tipo, nem como exemplo.

Enquanto não existir, qualquer marcação escrita na interface é escrita contra um
formato imaginado, e o `relay-core` contradiz depois. Esse é o retrabalho
imediato. Há um segundo motivo, mais duradouro: este formato é a peça que
sobrevive a tudo o que já foi cogitado neste projeto — troca de framework, Rust
no core, KMP no host. É o único artefato de que isso é verdade.

A ADR-0001 fixou duas restrições que decidem quase tudo aqui. O ponto 5 diz que
a aplicação lê, deriva e lança, mas **nunca escreve** nos cinco registros. O
Contexto dela diz que o risco que domina o desenho é existir uma segunda
implementação do protocolo, e que o caso que expõe a divergência é o
`inconsistent`.

O design system determina o que a tela principal consome: proveniência no
`HandoffCard` ("escrito no {harness} · {tempo relativo}"), objetivo, próximo
passo, uma ação primária, e o contador verdadeiro (`2 de 4`) quando houver
subtarefas. O `Header` mostra workspace e o estado derivado. A tela **Reparar**
"desabilita o resto" quando o estado é `inconsistent` — restrição por
construção, não por texto.

## Decisão

### 1. String entra, estado sai

A superfície pública do `relay-core` recebe **conteúdo**, nunca caminho, e o
pacote não importa `node:fs` em lugar nenhum.

```ts
interface RelayFiles {
  backlog: string
  todo: string
  handoff: string
  changelog: string
  specs: Record<string, string>   // caminho relativo -> conteúdo
}

function deriveState(files: RelayFiles): RelayState
```

**Por quê:** transforma "a aplicação nunca escreve um registro" de regra
lembrada em propriedade estrutural. O pacote que decide o estado não alcança o
disco, e o pacote que alcança o disco não decide nada. De quebra, todo teste
roda sem sistema de arquivos.

### 2. `inconsistent` é outra forma, não outro valor

```ts
type WorkStatus =
  | 'backlog' | 'ready' | 'in_progress' | 'blocked' | 'done' | 'idle'

type RelayState =
  | { kind: 'ok';           status: WorkStatus; /* … */ }
  | { kind: 'inconsistent'; violations: Violation[] }   // violations não vazio
```

**Por quê:** a tela Reparar precisa desabilitar o resto. Com união discriminada,
a interface **não consegue** ler `status`, `handoff` ou `todo` num estado
inconsistente — o compilador recusa. É a mesma técnica da ADR-0002: a garantia
mora na forma, não no lembrete. O `docs/PROTOCOL.md` já diz que `inconsistent`
é diagnóstico derivado e não pode ser escrito como status de trabalho; aqui isso
vira tipo.

### 3. Disponibilidade é derivada, nunca calculada na view

```ts
interface ChecklistEntry {
  id: string                         // B-001 | T-001
  text: string
  marker: ' ' | '•' | '!' | 'x'
  needs: string[]
  available: boolean                 // marker === ' ' && todo needs em 'x'
  spec?: string                      // apenas no backlog
}
```

**Por quê:** `available` é a semântica que o protocolo acabou de ganhar. Se a
view a recalcular, existem duas implementações da mesma regra — exatamente o que
a ADR-0001 quer impedir. O `relay-core` decide; a view desenha.

### 4. Contador sim; posição, fila e percentual não

```ts
interface OkState {
  kind: 'ok'
  status: WorkStatus
  handoff: Handoff | null
  activeBacklogId: string | null
  todo: ChecklistEntry[]
  backlog: ChecklistEntry[]
  completed: number      // subtarefas [x] do item ativo
  total: number          // subtarefas do item ativo
}
```

O contrato expõe `completed` e `total`. Ele **não** expõe `progress`,
`percentage`, `position`, `index`, `priority` nem `order`.

**Por quê:** o que o contrato expõe, a interface uma hora desenha. A ADR-0001
corrigida registra que a ordem textual não codifica prioridade, fila,
dependência, sequência nem percentual, e que isso é decidido no negativo. Um
campo `progress: 0.5` no contrato reintroduziria pela porta dos fundos aquilo
que o protocolo recusou pela porta da frente. O design system pede o contador
verdadeiro (`2 de 4`) e proíbe fila, posição e percentual; o contrato torna a
proibição impossível de violar por engano.

### 5. Proveniência é fato absoluto; tempo relativo é apresentação

```ts
interface Handoff {
  backlogId: string
  todoId: string
  spec: string
  harness: string        // [a-z0-9][a-z0-9._-]*
  updated: string        // RFC 3339 com offset explícito ou Z
  objective: string
  nextStep: string
  context: string
}
```

**Por quê:** "há 3 minutos" muda a cada segundo e não pode viver num estado
derivado de arquivos em disco. O contrato carrega o instante; a view formata.

### 6. Tom e tela ficam fora do contrato

O contrato carrega `status`. Não carrega `tone`, `screen`, `label` nem
`variant`.

**Por quê:** o `docs/design-system/README.md` é autoridade sobre o desenho, e as
duas tabelas que fazem esse mapeamento vivem lá — status → tom (seção 3) e
status → tela (seção 5). Se o contrato carregasse o tom, mudar uma cor deixaria
de ser mudança no design system e passaria a ser mudança no `relay-core`, o que
inverte a governança. **O contrato carrega o que o protocolo determina; o design
system determina tudo sobre como aquilo aparece.**

### 7. O envelope separa protocolo de ambiente

```ts
interface UiPayload {
  state: RelayState        // derivado pelo relay-core, puro
  environment: Environment // fornecido pelo relay-host
}

interface Environment {
  workspace: string        // caminho do workspace corrente
  execEnabled: boolean     // false sob --no-exec
}
```

**Por quê:** `workspace` não é derivável dos registros, e o `relay-core` é puro
— não pode inventá-lo. Separar mantém a fronteira da ADR-0001 legível no
próprio tipo. E `execEnabled: false` faz a exigência da ADR-0001 ponto 4 da
Conformidade ("a rota de lançamento não existe quando `--no-exec` está ativo")
chegar à interface como dado, não como convenção.

### 8. Uma violação diz qual verificação falhou

```ts
interface Violation {
  check: string       // identificador estável, ex.: 'backlog-id-mismatch'
  detail: string      // o conflito concreto, com os IDs envolvidos
  records: string[]   // registros participantes, ex.: ['handoff', 'todo']
}
```

**Por quê:** a tela Reparar precisa "mostrar o conflito e a correção
determinística". Uma string solta não permite isso. O `check` estável também
permite testar cada verificação de integridade do protocolo individualmente.

### 9. Cada verificação de integridade tem um identificador estável

O `check` de uma `Violation` vem desta tabela, na ordem em que as verificações
aparecem em `docs/PROTOCOL.md`. O identificador é contrato: renomear um exige
supersedir esta ADR, porque a tela Reparar e os testes dependem dele.

| `check` | Verificação |
| --- | --- |
| `handoff-names-no-pending-todo` | Handoff não vazio não nomeia um item pendente do TODO |
| `backlog-id-mismatch` | ID de backlog ativo inexistente no backlog, ou handoff, TODO e backlog discordam do mesmo backlog ID |
| `spec-path-mismatch` | Tarefa ativa sem entrada de backlog confrontável, sem spec, ou com spec diferente do handoff |
| `handoff-harness-invalid` | Handoff não vazio sem `Harness` ou fora do formato permitido |
| `handoff-updated-invalid` | Handoff não vazio sem `Updated` ou fora do RFC 3339 exigido |
| `multiple-handoffs` | Existe mais de um registro de handoff corrente |
| `todo-cleared-before-changelog` | Item saiu do handoff antes do resultado entrar no changelog |
| `backlog-done-with-pending-todo` | Backlog `done` com item de TODO ativo não concluído |
| `unknown-marker` | Marcador desconhecido, ou item concluído que não é `[x]` |
| `needs-unknown-id` | `needs` referencia ID ausente do mesmo registro |
| `needs-cycle` | Relação de `needs` contém ciclo |
| `needs-incomplete-on-done` | Entrada `[x]` cujo `needs` não está todo `[x]` |
| `criteria-without-evidence` | Toda entrada de backlog de uma spec está `[x]` e um critério de aceite dela não é nomeado por nenhum registro de changelog |

## Fixtures

Sete fixtures normativos, um por status mais o diagnóstico. Campos omitidos
seguem o tipo. Estes são a fonte; B-004 os materializa como arquivos sob `app/`.

| Fixture | `kind` | Distintivo |
| --- | --- | --- |
| `idle` | ok | sem backlog, sem TODO, sem handoff |
| `backlog` | ok | backlog com entradas, TODO vazio, handoff vazio |
| `ready` | ok | TODO com item disponível, handoff vazio |
| `in_progress` | ok | handoff válido, um item `[•]` |
| `blocked` | ok | handoff `blocked` com `context`, um item `[!]` |
| `done` | ok | todo o backlog `[x]`, TODO vazio, handoff vazio |
| `inconsistent` | inconsistent | handoff aponta item já concluído |

```json
{
  "state": {
    "kind": "ok",
    "status": "in_progress",
    "activeBacklogId": "B-001",
    "handoff": {
      "backlogId": "B-001", "todoId": "T-002",
      "spec": ".specs/20260907-001-ui-primeiro-marco-visual.md",
      "harness": "claude-code", "updated": "2026-09-07T06:49:14Z",
      "objective": "Validar os fixtures contra o protocolo.",
      "nextStep": "Rodar as verificações de integridade.",
      "context": "T-001 concluída; o tipo está na ADR-0003."
    },
    "todo": [
      { "id": "T-001", "text": "Escrever a ADR-0003", "marker": "x",
        "needs": [], "available": false },
      { "id": "T-002", "text": "Validar os fixtures", "marker": "•",
        "needs": ["T-001"], "available": false },
      { "id": "T-003", "text": "Registrar no índice", "marker": " ",
        "needs": ["T-001"], "available": true }
    ],
    "backlog": [
      { "id": "B-001", "text": "Contrato do estado derivado", "marker": " ",
        "needs": [], "available": true,
        "spec": ".specs/20260907-001-ui-primeiro-marco-visual.md" }
    ],
    "completed": 1, "total": 3
  },
  "environment": { "workspace": "/Users/exemplo/relay", "execEnabled": false }
}
```

```json
{
  "state": {
    "kind": "inconsistent",
    "violations": [
      { "check": "handoff-names-no-pending-todo",
        "detail": "O handoff aponta T-001, que está [x]; T-002 está [•].",
        "records": ["handoff", "todo"] }
    ]
  },
  "environment": { "workspace": "/Users/exemplo/relay", "execEnabled": false }
}
```

Note que o segundo fixture **não tem** `status`, `handoff`, `todo` nem
`backlog`. Isso não é omissão: é a decisão 2 valendo.

## Consequências

### Positivas

- A interface pode ser desenhada agora, contra um formato real.
- Os sete fixtures são a suíte de testes do `relay-core` antes de existir uma
  linha dele, e o árbitro entre os dois leitores do protocolo.
- Três regras do projeto passam de texto a tipo: sem escrita, sem percentual,
  sem uso de estado inconsistente.
- Sobrevive a troca de framework, a Rust no core e a KMP no host, porque é JSON
  e uma declaração de tipo.

### Negativas e custos assumidos

- Todo campo novo exigido pela interface obriga a passar pelo `relay-core`, o
  que é mais lento que calcular na view — e é exatamente o efeito pretendido.
- `Environment` está mínimo de propósito. A lista de harnesses que o design
  system especifica (nome, versão, autenticado) fica de fora até existir uma
  spec do `relay-host`; isso é lacuna conhecida, não esquecimento.
- Os fixtures vivem nesta ADR até B-004. Enquanto isso, mudá-los é editar uma
  decisão registrada, o que é deliberadamente mais caro que editar um arquivo.
- A união discriminada exige que todo consumidor trate `kind` antes de qualquer
  outra coisa. Em TypeScript isso é gratuito; num consumidor em outra linguagem,
  não é.

### Consequência descartada explicitamente

Carregar o tom, o rótulo ou a tela no contrato para "poupar um mapeamento na
view". Pouparia poucas linhas e inverteria a governança do design system.

## Conformidade

1. `relay-core` não importa `node:fs` nem qualquer API de disco ou rede.
2. O contrato não ganha campo de progresso, percentual, posição, índice,
   prioridade ou ordem. Contador (`completed`, `total`) é o limite.
3. O contrato não ganha campo de apresentação: tom, cor, rótulo traduzido,
   nome de tela ou variante de componente.
4. `available` é sempre derivado pelo `relay-core`; nenhum consumidor o
   recalcula a partir de `marker` e `needs`.
5. Um estado `inconsistent` nunca carrega `status`, `handoff`, `todo` ou
   `backlog`, e `violations` nunca é vazio.
6. `updated` é sempre RFC 3339 com offset explícito ou `Z`. Tempo relativo é
   calculado na view e nunca armazenado.
7. Cada verificação de integridade do `docs/PROTOCOL.md` tem um `check` de
   identificador estável, listado na decisão 9. Uma verificação nova no
   protocolo acrescenta uma linha àquela tabela na mesma mudança. A tabela não
   declara um total: contagem em prosa apodrece a cada verificação nova.

## Notas

**Relação com a ADR-0005.** O framework da `relay-ui` é decisão separada e a
mais provável de ser superada. Esta ADR não a referencia, e é assim de
propósito: superar o framework não deve arrastar o contrato.

**Origem dos campos.** Cada campo existe porque um componente documentado o
consome. `objective`, `nextStep`, `harness` e `updated` vêm do `HandoffCard`;
`completed` e `total`, do contador; `workspace` e `status`, do `Header`;
`violations`, da tela Reparar; `execEnabled`, do `PreflightModal`, que não deve
existir sob `--no-exec`. Nenhum campo foi acrescentado por antecipação.

**O que não está decidido.** A forma como o estado chega à interface — mensagem
de WebSocket, rota HTTP, formato de reconexão e replay — é decisão do
`relay-host` e não desta ADR. Aqui está apenas o que trafega, não como trafega.
