# 20260907-002 - Evidência nomeia o critério de aceite

## Problem

O template de spec tem `## Acceptance criteria`. O `relay-spec` os **escreve**.
E acaba aí: nenhuma skill os lê, nenhuma transição os marca, nenhuma das doze
verificações de integridade os menciona. São write-once.

A prova está viva neste repositório: um item de backlog foi marcado `[x]` em
`.orchestration/BACKLOG.md` enquanto os oito critérios da spec dele seguiam
todos `[ ]`. Não foi descuido pontual: **não existe mecanismo que obrigasse a
olhar**.

O buraco é maior que ele parece. As doze verificações de integridade são todas
estruturais e de referência cruzada — nenhuma olha para o conteúdo de
`Evidence`. Uma subtarefa pode ir a `[x]` com `Evidence: pronto`. Hoje a
qualidade do changelog é alta por disciplina, e disciplina não é protocolo.

Há ainda um dano ativo: o marcador `[ ]` num critério de aceite **mente**. Ele
parece pendência que vai fechar, quando nada no sistema é capaz de fechá-lo.

## Scope

- Critérios de aceite ganham identificador (`A-001`, `A-002`) e deixam de usar
  checkbox.
- O registro de changelog ganha um campo que nomeia os critérios que a subtarefa
  avançou, ou declara explicitamente que não avançou nenhum.
- Fechar a **última** entrada de backlog de uma spec exige que todo critério
  dela tenha ao menos um registro de changelog o nomeando.
- Uma verificação de integridade nova, com identificador estável acrescentado à
  tabela da ADR-0003.
- As skills alinhadas ao contrato, depois dele, nunca antes.

## Non-goals

- **Nenhum status novo.** O caminho de rejeição já existe: `[!]` mais handoff
  `blocked` com causa e condição de retomada.
- **Nenhuma revisão de código.** O alvo é o critério de aceite da spec, não
  qualidade geral de implementação. Revisar cada subtarefa duplicaria custo para
  achar nada em subtarefas triviais.
- **Nenhum campo `covers` no backlog.** Critérios pertencem à spec, não a
  entradas individuais.
- **Nenhuma mutação de `.specs/` durante o trabalho.** A spec continua mudando
  só quando a especificação muda.
- Nada sobre subagents ou paralelismo. Foram avaliados em separado e recusados.

## Decisions

**Satisfação é derivada, não marcada.** Critérios viram lista com ID e sem
checkbox; a evidência mora no changelog e a satisfação é computada. Alternativa
descartada: marcar `[x]` na própria spec, que daria progresso visível ao custo
de um segundo escritor em `.specs/` e de uma regra de mutação nova num arquivo
que hoje só muda quando a especificação muda. Some de quebra o `[ ]` que mente.

**A verificação morde ao fechar a spec inteira.** Quando a última entrada de
backlog de uma spec vai a `[x]`, todo critério dela precisa ter evidência.
Alternativa descartada: cada entrada declarar `covers:`, que daria retorno mais
cedo ao custo de um campo novo no `BACKLOG.md` e de repartir critérios entre
entradas — repartição que muitas vezes não existe, porque um critério só é
verificável quando várias entradas fecharam.

**Contrato antes das skills**, como o `AGENTS.md` exige. `docs/PROTOCOL.md`
muda primeiro; as skills seguem.

**Retrocompatibilidade.** Specs existentes com critérios em checkbox continuam
válidas e não são reescritas só para adotar a convenção, na mesma linha da regra
que já vale para nomes de arquivo de spec.

## Acceptance criteria

- A-001 - O template de spec em `docs/PROTOCOL.md` usa `A-NNN` sem checkbox
- A-002 - O template de changelog tem campo que nomeia critérios, e admite
  declarar explicitamente que nenhum foi avançado
- A-003 - A transição 4 exige nomear os critérios ao concluir uma subtarefa
- A-004 - A transição 5 recusa fechar a última entrada de backlog de uma spec
  quando algum critério dela não tem evidência
- A-005 - Existe uma verificação de integridade nova com identificador estável,
  e a tabela da ADR-0003 ganhou a linha correspondente na mesma mudança
- A-006 - `relay-spec`, `relay-session` e `relay-status` refletem o contrato, e
  nenhuma skill passa de 39 linhas
- A-007 - Specs com critérios em checkbox continuam válidas
- A-008 - As specs existentes foram convertidas e seus critérios têm evidência
  atribuída retroativamente ou registrados como pendentes
- A-009 - Um critério nomeado no changelog identifica sem ambiguidade a spec a
  que pertence, mesmo quando a subtarefa corre sob outra spec

## Backlog candidates

- B-005: Protocolo exige que a evidência nomeie o critério de aceite
- B-006: Skills alinhadas ao novo contrato (needs: B-005)
- B-007: Specs existentes convertidas e seus critérios reconciliados
  (needs: B-005)
- B-008: Critério nomeado no changelog é qualificado pela spec (needs: B-005)
