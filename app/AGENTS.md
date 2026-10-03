# Instruções desta pasta

`app/` é a interface do Relay: o painel de terminal `relay-tui`. Ela traz as
próprias instruções para não ocupar contexto no `AGENTS.md` da raiz: você só lê
isto quando chega aqui.

## Ordem de leitura

1. `relay-tui/README.md` — estrutura do crate, compilar, testar e publicar.
2. `../docs/adr/0009-relay-tui-observador-de-terminal-em-rust.md` — por que o
   painel existe, em Rust, e só lê.
3. `../docs/design-system/terminal.md` — autoridade sobre a aparência da tela.
   Leia antes de mudar cor, rótulo ou layout.
4. `relay-tui/tests/fixtures/README.md` — os casos que o core precisa derivar.
   Leia antes de mudar uma regra do protocolo no `core`.

## O que esta pasta não pode fazer

- **Nunca escrever nos cinco registros.** Nenhum arquivo daqui escreve em
  `.specs/` ou `.orchestration/`, deste repositório ou de qualquer outro. Toda
  mutação passa por uma skill num harness.
- **Nunca fazer a raiz depender daqui.** `rm -rf app/` tem de devolver o
  repositório a um estado funcional; o único vínculo permitido é o ponteiro de
  uma linha no `AGENTS.md` da raiz, mais os dois workflows do `relay-tui` em
  `.github/workflows/` (sem `app/` eles não disparam).
- **Nunca calcular protocolo na `view`.** `available`, status e contagem chegam
  prontos do `relay-tui/src/core/`, que é o único lugar onde as regras do
  `docs/PROTOCOL.md` viram código. Mudar a gramática do protocolo exige mudar o
  `core` e os casos de `relay-tui/tests/fixtures/` juntos.

## Camadas

`skills/` é superfície de pacote e vai pelos manifestos. `app/` é produto: vai
por clone e o `relay-tui` também por download de binário, em Release.
`.agents/` e `.claude/` são ferramenta deste repositório e não vão a lugar
nenhum.
