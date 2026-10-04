# ADR-0010 — visibilidade da entrevista da spec na TUI

## Status

**Accepted** — 2026-10-04.

## Contexto

Depois de `relay-setup`, o workspace não tem trabalho aberto. A `relay-spec`
entrevista a pessoa antes de gravar a spec e as entradas do backlog. Nesse
intervalo, a TUI continua mostrando `idle`. A tela anterior dizia apenas que
não havia trabalho, o que podia sugerir que o painel não estava funcionando.

## Decisão

A tela `idle` explica que as perguntas acontecem no agente e que a spec aparece
no painel depois de salva. A orientação para criar uma spec continua visível em
40 colunas. O estado derivado permanece `idle` enquanto não houver novos
registros.

Um indicador de entrevista ativa exigiria um registro de atividade escrito
pela skill e uma transição nova no protocolo. Sem esse registro, a TUI não
consegue distinguir uma entrevista em curso de um workspace simplesmente
parado, então não afirma atividade em tempo real.

## Consequências

- A pessoa entende por que a TUI ainda não apresenta a spec durante as perguntas.
- O painel permanece somente leitura e não cria um estado inferido sem evidência.
- A mensagem é uma orientação estática; ela não mostra o avanço da entrevista.

## Compliance

1. `app/relay-tui/DESIGN.md` define o conteúdo e a altura do cartão `idle`.
2. Os testes de renderização verificam a orientação em 40 colunas e os
   snapshots a exibem em português e inglês, em 40 e 58 colunas.
3. A derivação de `idle` e os registros do protocolo não mudam.

## Notes

Origem: feedback de teste após `relay-setup` e durante as perguntas de
`relay-spec`.
