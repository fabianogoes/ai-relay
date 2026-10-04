# ADR-0006 — Changelog por spec e fechamento como arquivamento

## Status

**Accepted** — 2026-10-03.

Estende a decisão 1 da ADR-0003 (o protocolo é o contrato em disco e o
`relay-tui` só lê) e segue a regra da ADR-0001 de que cada instrução fica onde
ela se torna relevante.

## Contexto

O `.orchestration/CHANGELOG.md` é um arquivo único e append-only. Neste
repositório chegou a cerca de 2.400 linhas e é lido para verificar critérios de
aceite, de modo que o custo em tokens só cresce. O arquivo único também
concentra o conflito de merge entre branches. O `BACKLOG.md` guarda para sempre
uma linha por item concluído, o mesmo problema em escala menor.

Além disso, o protocolo não sabia abandonar: remover a interface web exigiu
apagar 12 specs e 111 registros de um changelog que deveria ser append-only.
Pelo `AGENTS.md`, essa fricção é defeito do `docs/PROTOCOL.md`.

## Decisão

1. **Um arquivo de changelog por spec**, em
   `.orchestration/changelog/<YYYYMMDD-NNN>.md`. O arquivo é a spec, então o
   registro não repete `Spec`; a coerência vem da entrada de backlog nomeada em
   `Backlog`. Uma sessão lê só o arquivo da spec ativa.
2. **Fechar é arquivar.** Quando a última entrada pendente de uma spec fecha, o
   escritor acrescenta ao changelog dela uma seção `## Closed <data>` com as
   entradas finais copiadas literalmente e só então as remove do `BACKLOG.md`. A
   seção de fechamento é a autoridade: uma entrada cujo ID já aparece no
   fechamento da própria spec está arquivada, e removê-la é limpeza. Assim a
   janela entre as duas escritas não produz estado falso.
3. **Descarte com `[-]` e motivo**, escrito só pelo `relay-spec`, nunca apagando.
   Uma entrada descartada não fica disponível e não satisfaz `needs`. Um critério
   sem evidência numa spec com descarte é dispensado com `Waived: A-NNN - <motivo>`
   no fechamento; a dispensa só existe com descarte.
4. **Legado legível, migração sob confirmação.** O `CHANGELOG.md` único continua
   válido para leitura e nunca recebe registro novo. O `relay-setup` o divide por
   `Spec`, movendo cada registro sem alterar o texto.
5. **Próximo `B-NNN`** é um a mais que o maior `B-NNN` sob `.orchestration/`,
   incluindo os arquivados e o legado.

### Alternativas rejeitadas

- **Um arquivo por registro:** zero conflito, mas centenas de arquivos pequenos.
- **Rotação por período:** não agrupa por spec, a verificação de critério
  atravessa arquivos e o arquivo corrente continua sendo ponto de conflito.
- **Manter as entradas `[x]` no `BACKLOG.md`:** o arquivo cresce para sempre.
- **Arquivar por pedido explícito:** vira um passo a lembrar, e o arquivo volta a
  crescer.
- **Apagar a spec e os registros ao abandonar:** foi o que este repositório
  precisou fazer em 2026-10-03 e viola o append-only.
- **Dispensar critério sem descarte:** viraria um atalho para não provar um
  critério; critério que deixou de fazer sentido é mudança da especificação.

## Consequências

- Com tudo fechado o estado derivado é `idle`; o `done` por "backlog todo `[x]`"
  continua valendo para backlogs anteriores a esta mudança.
- Leitores (o core do `relay-tui` e as skills) precisam ler `changelog/*.md` e o
  legado; isso é trabalho das specs de B-053 a B-057.
- O `.specs/` não muda de lugar: nenhum caminho já citado em registro quebra.
- A migração é a única movimentação de registro que o protocolo permite.

## Compliance

1. `docs/PROTOCOL.md` e `docs/PROTOCOL.pt-BR.md` descrevem a estrutura, o
   fechamento, o descarte, a dispensa e o legado, com os blocos de código
   idênticos.
2. Nenhum cliente escreve em `changelog/`; só skills do Relay.
3. Cada verificação de integridade nova entra na tabela da ADR-0003 na mesma
   mudança que a implementa (B-052 e B-053).

## Notes

Origem: spec `.specs/20261003-001-changelog-por-spec-e-contrato-nas-skills.md`.
