# 20260907-003 - Critério nomeado só quando o próprio registro o demonstra

## Problem

`docs/PROTOCOL.md` (B-005) exige que `Criteria` nomeie o critério de aceite
avançado, e que a **última** entrada de backlog de uma spec só feche quando
todo critério estiver nomeado por algum registro. O mecanismo verifica
**presença** — existe algum registro nomeando o ID — nunca **procedência**: se
foi o próprio registro que nomeou, ou outro depois dele, que de fato demonstrou
o critério.

Um exemplo do furo: o registro de um scaffold nomeia `Criteria: A-004` — "a
tela principal renderiza os sete fixtures" — enquanto o `Result` daquele mesmo
registro lista só arquivos de configuração; nenhum componente de tela existe
ainda. O critério só fica verdadeiro dois registros depois. O mecanismo de B-005
não acusa nada, porque os registros seguintes também nomeiam A-004 — "algum
registro nomeia" fica satisfeito, e a reivindicação falsa passa sem deixar
rastro de erro.

## Scope

- Regra de proveniência: um registro só nomeia em `Criteria` o que o próprio
  `Result`/`Evidence` **daquele registro** demonstra — nunca em antecipação a
  um registro futuro.
- Atualizar `relay-session`, que é quem escreve `Criteria`.

## Non-goals

- **Nenhuma verificação mecânica de veracidade.** Nada no protocolo consegue
  ler prosa e julgar se a evidência realmente demonstra o critério — isso é
  revisão humana ou de outro agente, explicitamente fora do escopo de B-005
  ("Nenhuma revisão de código"). Esta regra é disciplina de proveniência, do
  mesmo tipo que "`none` é afirmação como qualquer outra e precisa ser
  verdadeira" — sem checagem automática, mas com a frase certa para apoiar
  quem revisa.
- Nenhuma verificação de integridade nova. Não há como computar "este registro
  demonstra aquele ID" sem entender linguagem natural.
- Nenhuma mudança na sintaxe de qualificação entre specs (`YYYYMMDD-NNN/A-NNN`)
  criada em B-008.

## Decisions

**Regra de proveniência, não de mecanismo.** A alternativa seria alguma
verificação estrutural — por exemplo, exigir uma palavra-chave em `Evidence`
espelhando o ID citado em `Criteria`. Descartada: forçaria uma sintaxe
artificial (`Evidence: [A-004] ...`) para simular verificação que continuaria
sendo de fachada, já que nada impede citar a palavra sem o conteúdo bater.
Mais barato e mais honesto declarar a regra em prosa, no mesmo padrão que já
rege `none`.

**Contrato antes da skill**, como o `AGENTS.md` exige.

## Acceptance criteria

- A-001 - `docs/PROTOCOL.md` declara que `Criteria` só nomeia o que o próprio
  registro demonstra, nunca em antecipação
- A-002 - `relay-session` reflete a regra
- A-003 - Nenhuma skill excede as linhas das demais após a mudança

## Backlog candidates

- B-009: Protocolo e relay-session exigem proveniência própria em Criteria
