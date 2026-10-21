# ADR-0011 — Dependências de backlog arquivadas

## Status

**Accepted** — 2026-10-21.

## Context

A ADR-0006 tornou o fechamento de uma spec um arquivamento: suas entradas saem
do `BACKLOG.md` depois de copiadas para `## Closed` no changelog. Uma entrada
pendente de outra spec pode depender de um desses IDs. A leitura literal de
"mesmo registro" na regra de `needs` passava a marcar esse trabalho como
`inconsistent`, embora a dependência tivesse sido concluída. O core da TUI já
considerava IDs arquivados ao derivar disponibilidade, mas o texto do protocolo
não definia essa exceção.

## Decision

- Um `needs` de TODO continua restrito ao próprio TODO.
- Um `needs` de backlog resolve primeiro os IDs vivos no `BACKLOG.md` e também
  os IDs arquivados nas seções `## Closed` de todos os changelogs por spec. Os
  IDs `B-NNN` são únicos no workspace; a spec da entrada dependente não revela
  a spec da dependência.
- A seção de fechamento continua sendo a autoridade durante a janela em que uma
  cópia da entrada ainda está no backlog. Apenas `[x]` satisfaz uma dependência;
  `[-]` continua produzindo violação para uma entrada pendente.

## Consequences

- Uma entrada pendente fica disponível depois que sua dependência `[x]` é
  arquivada, inclusive quando as entradas pertencem a specs diferentes.
- Leitores que validam `needs` precisam consultar as seções de fechamento de
  todos os changelogs por spec. O `relay-tui` já lê esses arquivos para derivar
  estado; a mudança alinha sua checagem com sua disponibilidade.
- A regra não cria uma operação para editar `needs` nem altera o formato dos
  registros. A verificação de IDs duplicados continua protegendo a resolução.

## Compliance

1. `docs/PROTOCOL.md` e `docs/PROTOCOL.pt-BR.md` descrevem a resolução em
   changelogs e a diferença entre `[x]` e `[-]`.
2. As referências das skills repetem literalmente as seções alteradas do
   protocolo em inglês.
3. Os fixtures do core cobrem um ID `[x]` arquivado em outra spec e um ID `[-]`
   arquivado que mantém o estado inconsistente.

## Notes

Origem: issue #13 (`fabianogoes/ai-relay`). Estende a ADR-0006; não a substitui.
