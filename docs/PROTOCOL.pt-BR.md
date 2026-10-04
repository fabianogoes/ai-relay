# Protocolo do Relay

**[English](PROTOCOL.md)** · Português

> Tradução de [PROTOCOL.md](PROTOCOL.md). Em caso de divergência, vale o
> original em inglês. Os trechos de código são o contrato em disco e ficam
> idênticos nos dois arquivos: nomes de arquivos, títulos, campos e status não
> são traduzidos.

Este documento é o contrato em disco da versão um. O Relay guarda a intenção e
o estado de execução duráveis em Markdown, para que harnesses diferentes leiam o
mesmo trabalho sem depender de um histórico de chat compartilhado.

## Arquivos e responsabilidade

| Caminho | Finalidade | Regra de alteração |
| --- | --- | --- |
| `.specs/YYYYMMDD-NNN-<slug>.md` | Intenção detalhada e aceitação | Criado por `relay-spec`; atualizado só quando a especificação muda. `NNN` começa em `001` a cada data. |
| `.orchestration/BACKLOG.md` | Trabalho aberto selecionável de forma independente | Acrescenta tarefas de checklist a partir de uma spec; marca uma tarefa como `done` só quando o seu TODO estiver completo; descarta uma tarefa com `[-]`; remove as entradas de uma spec quando ela fecha (ver "Fechamento de uma especificação"). |
| `.orchestration/TODO.md` | Subtarefas executáveis da tarefa atual | Substituído quando uma tarefa do backlog é selecionada; esvaziado só quando todos os itens estão `done`. |
| `.orchestration/HANDOFF.md` | Exatamente uma subtarefa atual ou retomável | Escrito antes de o trabalho começar; esvaziado depois de o registro de conclusão existir. |
| `.orchestration/changelog/<YYYYMMDD-NNN>.md` | Trabalho concluído e evidência de uma spec | Somente acréscimo, um arquivo por spec. Uma sessão lê só o arquivo da spec ativa. |
| `.orchestration/CHANGELOG.md` | Changelog único legado | Somente leitura: continua válido para ler e nunca recebe registro novo. Só o `relay-setup` move registros para fora dele (ver "Changelog legado"). |

## Responsabilidades das skills

- `relay-setup` inicializa ou atualiza o protocolo de forma idempotente e migra
  um `CHANGELOG.md` legado depois de confirmação.
- `relay-spec` transforma uma conversa em uma especificação e em entradas de
  backlog, e depois apresenta o próximo caminho como uma escolha nativa
  selecionável. Só ele descarta uma entrada de backlog ou uma especificação
  inteira, e só ele escreve uma dispensa.
- `relay-status` valida e informa o estado sem alterar nada.
- `relay-continue` deriva o próximo passo e apresenta uma única escolha nativa
  recomendada; não altera o estado enquanto apresenta as opções.
- `relay-session` inicia ou retoma a implementação e garante a integridade do
  handoff.

## Idioma do workspace

Um arquivo opcional `.orchestration/SETTINGS.md` guarda o idioma padrão do
workspace:

```markdown
# Settings

- Language: en
```

`Language` aceita `en` ou `pt-BR`. Isto é configuração do workspace, não um dos
cinco registros do Relay, e fica fora das verificações de integridade. Só o
`relay-setup` o escreve. Um valor válido controla o texto livre da entrevista e
a prosa de especificações e sessões. Se o arquivo não existir ou o valor for
desconhecido, as skills usam o idioma da conversa. Um valor existente só muda
quando o usuário pede explicitamente ao `relay-setup` que o altere. Títulos de
seção, chaves, status, marcadores e IDs continuam em inglês.
Esta regra vale para prosa nova; o texto existente nos registros não é traduzido
implicitamente.

## Fronteira de autoridade

Os cinco locais de registro do Relay são `.specs/*.md`,
`.orchestration/BACKLOG.md`, `.orchestration/TODO.md`,
`.orchestration/HANDOFF.md` e o changelog, que é `.orchestration/changelog/*.md`
ou o legado `.orchestration/CHANGELOG.md`. Clientes como uma interface gráfica, um lançador, um observador de arquivos ou outra integração
podem lê-los, validar referências cruzadas, derivar e exibir o estado e lançar
um harness com a skill apropriada do Relay. Os clientes não podem escrever
nesses registros diretamente.

As skills do Relay executadas dentro de um harness são as escritoras de
autoridade. Uma skill só pode alterar os registros e as transições que este
contrato lhe atribui; as skills de leitura continuam só de leitura. Se um
cliente não consegue derivar um estado necessário a partir dos registros, mude
este protocolo antes de acrescentar lógica de escrita privada ao cliente.

## Trabalho fora do fluxo

O Relay governa o trabalho acompanhado por ele, não toda alteração do
repositório. O usuário pode pedir uma mudança diretamente, sem invocar uma skill
do Relay. Esse trabalho não cria especificação, entrada de backlog, item de
TODO, handoff nem registro de changelog, e deixa o estado derivado inalterado.

Um agente que recebe um pedido direto não precisa ler antes os registros do
Relay e não deve encaminhar o trabalho para o fluxo por conta própria. Ele pode
sugerir `relay-spec` quando o trabalho se mostrar maior que uma sessão ou
atravessar harnesses, mas quem decide é o usuário. Ler os registros só é
obrigatório antes de uma skill do Relay agir, ou antes de um trabalho que o
usuário vincule a uma tarefa do backlog ou ao handoff ativo.

O trabalho direto nunca escreve nos cinco registros, então não pode tornar o
estado `inconsistent`: as verificações de integridade comparam os registros
entre si, não com o código. Quando há um handoff ativo, o trabalho direto não faz
parte dele e não é registrado sob ele.

## Status permitidos

`backlog`, `ready`, `in_progress`, `blocked`, `done` e `idle` são valores de
status em inglês. `inconsistent` é um diagnóstico derivado e não deve ser
escrito como status de trabalho.

`done` e `idle` são os estados terminais e são derivados, nunca escritos. Um
backlog não vazio cujas entradas são todas `[x]` deriva `done`; backlogs
escritos antes de as especificações serem arquivadas ao fechar ainda derivam.
Quando uma especificação fecha, as entradas dela saem do backlog, então com tudo
fechado o estado é `idle`. A ausência de entradas de backlog, TODO e handoff
deriva `idle`. Um backlog vazio não é um status: sem entradas pendentes no
backlog, os outros registros decidem o estado.

Os nomes dos arquivos de especificação usam `YYYYMMDD-NNN-<slug>.md`: a data é a
da criação e `NNN` é uma sequência de três dígitos que recomeça em `001` a cada
dia. As especificações existentes com nomes mais antigos continuam válidas e não
devem ser renomeadas só para adotar esta convenção.

O estado entre o último item do TODO e o passo 5 das regras de transição (todo
item do TODO `[x]`, sem handoff, entrada de backlog ainda aberta) é `done` da
tarefa ativa, e o próximo passo é a transição 5. Nenhum status é acrescentado
para ele.

## Leitura dos registros

Os leitores tratam `\r\n` como `\n`: um registro escrito com fins de linha CRLF
deriva o mesmo estado que o mesmo registro escrito com LF.

## Datas e horários

Uma data ou um horário num registro vem do relógio do sistema no momento da
escrita, lido com o comando abaixo. Nunca vem da memória, da conversa nem de
metadados do sistema de arquivos, porque um valor inventado por quem escreve passa
na verificação de formato e mesmo assim mente sobre quando o trabalho aconteceu.

| Campo | Comando | Exemplo |
| --- | --- | --- |
| `Updated` do handoff | `date -u +%Y-%m-%dT%H:%M:%SZ` | `2026-09-05T23:41:00Z` |
| Data de um registro de changelog e de `## Closed` | `date +%Y-%m-%d` | `2026-09-05` |
| Data no nome de uma especificação | `date +%Y%m%d` | `20260905` |

`Updated` é escrito em UTC com `Z`: o `%z` do `date` imprime `-0300`, sem os
dois-pontos que o RFC 3339 exige.

## Modelo de especificação

```markdown
# 20260905-001 - <Specification title>

## Problem
<Who needs what and why.>

## Scope
<Included behavior and technical constraints.>

## Non-goals
<Explicit exclusions.>

## Decisions
<Decision, rationale, and alternatives rejected.>

## Acceptance criteria
- A-001 - <Observable result>
- A-002 - <Another observable result>

## Backlog candidates
- B-001: <Independent outcome>
- B-002: <Another independent outcome>
```

## Critérios de aceitação

Os critérios de aceitação de uma especificação têm identificadores estáveis
`A-NNN` e **nenhum marcador de checklist**. Um critério está satisfeito quando
pelo menos um registro do changelog o nomeia em `Criteria`; a satisfação é
derivada, nunca escrita na spec.

O marcador é omitido de propósito. Um `[ ]` num critério parece um trabalho
pendente que será concluído, enquanto nada no protocolo jamais o conclui — a
especificação permanece inalterada enquanto o trabalho acontece ao redor dela.

`A-NNN` é único apenas dentro de uma especificação, então `Criteria` o resolve
pelo contexto: um ID sem qualificação pertence à especificação do `Spec` do
registro. Um critério de outra especificação é qualificado com o prefixo
`YYYYMMDD-NNN` dessa especificação, como em `20260905-001/A-003`. O trabalho
feito sob uma especificação pode satisfazer um critério de outra, e a
qualificação é o que torna isso registrável em vez de ambíguo.

As especificações existentes cujos critérios usam marcadores de checklist
continuam válidas e não devem ser reescritas só para adotar esta convenção.

Um registro nomeia em `Criteria` apenas o que o seu próprio `Result` e o seu
`Evidence` já demonstram, nunca em antecipação de um registro posterior. Nomear
um critério antes do trabalho que o satisfaz é uma afirmação como qualquer outra
e precisa ser verdadeira quando feita, não só quando a especificação for
encerrada.

## Modelo de backlog

```markdown
# Backlog

- [ ] B-001 - <Independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-002 - <Another independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [ ] B-003 - <Outcome that requires B-001> (spec: `.specs/20260905-001-<slug>.md`) (needs: B-001)
```

Use `[ ]` para `backlog`, `[x]` para `done`, `[!]` para uma entrada bloqueada e
`[-]` para uma descartada. Uma entrada descartada leva `(dropped: <motivo>)`
depois da spec, nunca é apagada, não está disponível e não satisfaz `needs`.
Mantenha o resultado e os detalhes
de aceitação na spec de origem; cada entrada aponta para exatamente uma spec. A
ordem textual pode definir apenas a recomendação padrão determinística: a
primeira entrada disponível. Ela não codifica prioridade, fila nem dependência.
Uma dependência é declarada com `needs`, nunca deduzida pela posição, e o usuário
pode selecionar qualquer entrada disponível.

O próximo ID de backlog é um a mais que o maior `B-NNN` em qualquer lugar sob
`.orchestration/`, incluindo as seções arquivadas e o changelog legado; resolva
com `grep`, sem ler os arquivos inteiros. Nenhum ID existente é renumerado.

## Modelo de TODO

```markdown
# Active task: B-001

- [ ] T-001 - <Small executable outcome>
- [•] T-002 - <Current executable outcome>
- [!] T-003 - <Blocked executable outcome>
- [x] T-004 - <Completed executable outcome>
- [ ] T-005 - <Outcome that requires T-004> (needs: T-004)
```

Quando não há tarefa selecionada, use exatamente este estado vazio:

```markdown
# Active task

No active task.
```

A ordem dos itens do TODO não codifica dependência, sequência de execução,
esforço nem porcentagem de progresso. Uma dependência é declarada com `needs`.
Quando mais de um item está disponível, o primeiro em ordem textual é apenas a
recomendação padrão determinística; o usuário pode selecionar qualquer item
disponível.

## Dependências

Uma entrada de checklist pode declarar dependências explícitas com
`(needs: <ID>[, <ID>]...)`, referenciando outros IDs do mesmo registro. Essa é a
única forma de expressar que uma entrada exige outra; a posição textual nunca
carrega esse significado.

Uma entrada está **disponível** quando é `[ ]` e todo ID de que ela precisa é
`[x]`. Todo padrão determinístico seleciona a primeira entrada disponível em
ordem textual. Uma entrada que não está disponível nunca é oferecida como padrão
e nunca é selecionada em silêncio.

Como uma entrada `[!]` não é `[x]`, as entradas que dependem dela ficam
indisponíveis enquanto ela estiver bloqueada. Quando nenhuma entrada está
disponível, nenhuma é `[•]` e há entradas incompletas, o registro está
`blocked`: o trabalho não pode prosseguir até que uma entrada bloqueada seja
resolvida.

`needs` é opcional e retrocompatível. Um registro que o omite se comporta
exatamente como antes.

## Modelo de handoff

```markdown
# Handoff

- Status: in_progress
- Backlog: B-001
- TODO: T-001
- Spec: .specs/20260905-001-<slug>.md
- Harness: claude-code
- Updated: 2026-09-05T23:41:00-03:00

## Objective
<What this subtask must achieve.>

## Next step
<The next concrete action.>

## Context
<Decisions, files inspected, command output, or blocker details needed to resume.>
```

`Harness` identifica o harness cuja skill do Relay escreveu por último o handoff
não vazio. Use um identificador estável que case com `[a-z0-9][a-z0-9._-]*`,
como `codex`, `claude-code` ou `opencode`. Quem lê deve aceitar identificadores
desconhecidos que sigam esse formato.

`Updated` registra o horário dessa mesma escrita. Use a forma RFC 3339
`YYYY-MM-DDTHH:MM:SSZ` ou `YYYY-MM-DDTHH:MM:SS±HH:MM`, por exemplo
`2026-09-05T23:41:00-03:00` ou `2026-09-06T02:41:00Z`. Toda alteração de um
handoff não vazio deve atualizar os dois campos juntos. Quem lê não deve deduzir
nenhum dos dois valores dos metadados do sistema de arquivos.

Use `Status: blocked` somente quando `Context` declara o bloqueio e a condição
necessária para retomar.

Os metadados do handoff são as linhas `- Chave: valor` antes do primeiro `##`;
o que vem nas seções é texto livre e nunca altera os metadados, então uma linha
`- Status:` dentro de `## Context` é ignorada. `Status` aceita apenas
`in_progress` e `blocked`; qualquer outro valor é violação, nunca lido como
`in_progress`. O handoff só é vazio quando é exatamente a forma vazia abaixo:
citar "No active handoff" em outro lugar não o esvazia. Um handoff vazio é:

```markdown
# Handoff

No active handoff.
```

## Modelo de changelog

O changelog é um arquivo por especificação:
`.orchestration/changelog/<YYYYMMDD-NNN>.md`, nomeado pela data e pela sequência
da spec. O arquivo é a spec, então o registro não repete `Spec`: a spec da
entrada de backlog nomeada em `Backlog` tem de ser a do próprio arquivo. Uma
sessão lê só o arquivo da spec ativa.

```markdown
# Change log 20260905-001

## 2026-09-05 - T-001 - <Subtask title>
- Backlog: B-001
- Result: <What changed.>
- Evidence: <Test, inspection, commit, or other verifiable result.>
- Criteria: <Criterion IDs advanced, qualified when from another spec, or none.>
- Decisions: <Decision retained for future sessions, or none.>
```

Um critério não qualificado pertence à spec do arquivo; `YYYYMMDD-NNN/A-NNN`
continua qualificando um critério de outra spec. O arquivo é criado junto com o
primeiro registro.

## Fechamento de uma especificação

Fechar é arquivar. Quando a última entrada pendente do backlog de uma spec passa
a `[x]` ou `[-]`, o escritor acrescenta ao changelog dessa spec uma seção, e só
então remove as entradas da spec do `BACKLOG.md`:

```markdown
## Closed 2026-09-05
- [x] B-001 - <Independent outcome> (spec: `.specs/20260905-001-<slug>.md`)
- [-] B-002 - <Another outcome> (spec: `.specs/20260905-001-<slug>.md`) (dropped: <reason>)
- Waived: A-003 - <reason>
```

As entradas são copiadas literalmente. **A seção de fechamento é a autoridade:**
uma entrada de backlog cujo ID já aparece na seção de fechamento da própria spec
está arquivada, e removê-la do `BACKLOG.md` é limpeza. Uma sessão interrompida
entre as duas escritas deixa, portanto, um estado coerente.

`Waived` existe só no fechamento de uma spec com ao menos uma entrada `[-]`, e só
o `relay-spec` a escreve. Um critério sem evidência numa spec com descarte
precisa de uma dispensa com motivo antes de a spec fechar. Um critério que
deixou de fazer sentido sem descarte é mudança da especificação e sai da spec.

Descartar uma spec é descartar cada uma de suas entradas pendentes; descartar a
última a fecha. Uma entrada pendente que precise de uma descartada é violação,
então descartar exige ajustar ou descartar as dependentes.

## Changelog legado

Um `.orchestration/CHANGELOG.md` existente continua válido para leitura e nunca
recebe registro novo. Depois de confirmação, o `relay-setup` o divide por `Spec`
em arquivos por spec, movendo cada registro com o texto inalterado. Registros
sem `Spec`, ou com uma spec que não existe, ficam no arquivo legado e são
reportados. Esta é a única movimentação de registro que o protocolo permite, e
uma segunda execução não muda nada.

## IDs únicos

Um `B-NNN` aparece uma vez entre o `BACKLOG.md` e as seções de fechamento do
changelog (fora a janela de fechamento descrita em "Fechamento de uma
especificação"), e um `T-NNN` aparece uma vez no `TODO.md`.

## Branches

Os registros viajam com o branch, e cada branch deriva o próprio estado. Um
branch é integrado com o handoff vazio. Quando dois branches alocam o mesmo
`B-NNN`, o ID em colisão é renumerado no branch que entra, e a verificação de ID
repetido acusa a colisão até lá.

## Regras de transição

1. `relay-spec` escreve uma spec e uma ou mais entradas `backlog`, e depois
   pergunta se deve criar outra spec, implementar a spec criada ou parar.
2. Selecionar qualquer item de backlog não marcado cria o seu `TODO.md` com
   subtarefas compactas em checklist. Se o usuário pede o padrão, use o primeiro
   item disponível em ordem textual, sem tratá-lo como de maior prioridade.
3. Antes de uma subtarefa começar, escreva um handoff que referencie o ID do
   TODO, o ID do backlog, o caminho da spec, o harness de origem e o horário de
   atualização; a sessão passa então a `in_progress`. Se o usuário pede o padrão
   entre vários itens de TODO disponíveis, use o primeiro em ordem textual.
4. Para concluir uma subtarefa, acrescente o seu registro ao changelog da spec, mude o
   marcador dela no TODO para `[x]` e então esvazie o handoff. O `Criteria` do
   registro nomeia todo critério de aceitação da spec que a subtarefa fez
   avançar, ou `none`. `none` é uma afirmação como qualquer outra e precisa ser
   verdadeira.
5. Depois que todos os itens do TODO estão `done`, marque a tarefa do backlog
   como `done` e substitua o TODO pelo seu estado vazio. Quando essa entrada era a última
   pendente da spec, feche a spec como descrito em "Fechamento de uma
   especificação". Marcar como `done` a
   **última** entrada pendente do backlog de uma especificação exige, além
   disso, que todo critério de aceitação dessa especificação seja nomeado por
   pelo menos um registro do changelog. Quando algum não é, a entrada continua
   pendente: marque `[!]` e escreva um handoff bloqueado nomeando os critérios
   sem evidência e o que os satisfaria.

**A ordem de escrita nos passos 4 e 5 não é uma sugestão.** O handoff é
esvaziado no passo 4, estritamente antes de o TODO ou o BACKLOG serem reescritos
no passo 5. Uma ferramenta ou harness que escreve primeiro o estado vazio do
TODO (ou a marca `done` do BACKLOG) e depois o handoff vazio produz exatamente a
janela que as verificações de integridade abaixo existem para pegar: um handoff
não vazio nomeando um ID de TODO ou de backlog que já não tem entrada
correspondente. Isso não é uma corrida a tolerar — é uma ordem de escrita para
acertar de primeira. Quando vários registros mudam juntos, esvazie ou atualize o
handoff no mesmo passo que o torna obsoleto, nunca num passo posterior.

6. `relay-spec` pode descartar uma entrada pendente do backlog com `[-]` e um
   motivo, ou uma spec inteira. Subtarefas de um TODO nunca são descartadas.

`relay-continue` pode ser usado antes de uma sessão para resumir esta máquina de
estados. Ele executa apenas a opção escolhida pelo usuário; iniciar ou retomar o
trabalho é delegado a `relay-session`.

Quando um handoff nomeia um item de TODO concluído e exatamente um outro item de
TODO está `[•]`, `relay-continue` pode oferecer uma recuperação de handoff
obsoleto. Depois da confirmação, ele atualiza apenas os metadados do handoff e
registra o conteúdo anterior como contexto de recuperação. Conflitos ambíguos
continuam bloqueados para reparo manual.

## Verificações de integridade

Trate o estado como `inconsistent` quando qualquer condição abaixo falhar:

- Um handoff não vazio não nomeia um item de TODO pendente.
- O ID de backlog ativo não existe no backlog, ou os registros de handoff, TODO e
  backlog não concordam sobre o mesmo ID de backlog.
- A tarefa ativa não tem entrada de backlog confrontável, não tem spec, ou tem
  uma spec diferente do caminho de spec do handoff.
- Um handoff não vazio omite `Harness` ou usa um identificador fora do formato
  permitido.
- Um handoff não vazio omite `Updated` ou o valor dele não é um timestamp RFC
  3339 com segundos e um deslocamento explícito ou designador UTC.
- Existe mais de um registro de handoff atual.
- O `Status` de um handoff não é `in_progress` nem `blocked`.
- Um `B-NNN` aparece mais de uma vez entre o backlog e as seções de fechamento,
  ou um `T-NNN` aparece mais de uma vez no TODO.
- Um registro de changelog nomeia uma entrada de backlog cuja spec não é a do
  arquivo que guarda o registro.
- Um item de TODO é removido do handoff antes de o seu resultado concluído ser
  acrescentado ao changelog.
- Uma tarefa do backlog está `done` enquanto um item de TODO ativo dela não está
  `done`.
- Um item de checklist usa um marcador desconhecido, ou um item concluído não é
  `[x]`.
- Uma referência `needs` nomeia um ID ausente do mesmo registro.
- Uma relação `needs` contém um ciclo.
- Uma entrada está `[x]` enquanto um ID de que ela precisa não está `[x]`.
- Uma entrada pendente do backlog precisa de uma entrada descartada (`[-]`).
- Uma entrada descartada não tem motivo.
- Uma linha `Waived` aparece numa seção de fechamento sem nenhuma entrada `[-]`.
- Toda entrada de backlog de uma especificação está `[x]` ou `[-]` enquanto um
  critério de aceitação dessa especificação não é nomeado por nenhum registro do changelog.
