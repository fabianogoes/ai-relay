# ADR-0007 — CI do pacote, separado do CI do relay-tui

## Status

**Accepted** — 2026-10-03.

Estende a exceção da decisão 6 da ADR-0003 (workflows fora de `app/`) e respeita
a conformidade 3 da ADR-0002 (`rm -rf app/` devolve o repositório a um estado
funcional).

## Contexto

O CI existente cobre só o `relay-tui`: o `relay-tui-ci.yml` dispara com mudanças
em `app/relay-tui/**` e o de release, com tags. O pacote em si (as skills, o
único script que uma delas distribui e o contrato que as skills copiam do
`docs/PROTOCOL.md`) não tinha verificação nenhuma. O `open-split.sh` é o único
executável publicado, o teste dele tem 145 casos e não rodava no CI, e o teste de
paridade das referências (`.agents/tests/skill-contract.test.sh`) precisa rodar
quando o protocolo muda, coisa que o CI do `relay-tui` não vê.

## Decisão

1. **Um workflow próprio do pacote**, `.github/workflows/package-ci.yml`. Ele roda
   os testes de `.agents/tests/` no macOS, que é o requisito e onde o `osascript`
   existe, e o `shellcheck` no Ubuntu, onde já vem instalado, sobre todo `*.sh` de
   `skills/` e `.agents/tests/`.
2. **Dispara só com o que o pacote usa:** `skills/**`, `.agents/tests/**`,
   `docs/PROTOCOL.md` e o próprio arquivo, em push na `main` e em pull request.
3. **`shellcheck -S warning`.** Aviso e erro reprovam. O nível `info` acusa como
   "nunca chamadas" as funções que o `open-split.sh` chama por variável, e
   silenciá-las dentro do script distribuído poria ruído no código que as pessoas
   instalam.
4. **O workflow do pacote não depende de `app/`**, e o dos `relay-tui` não depende
   do pacote. Removido `app/`, o CI do pacote continua válido.

### Alternativas rejeitadas

- **Rodar os testes do pacote no `relay-tui-ci.yml`:** ele só dispara com mudança
  em `app/`, então o teste não rodaria quando o protocolo mudasse, e acoplaria o
  pacote à interface.
- **Um comando de verificação (`relay check`):** seria um Relay CLI, que o
  `AGENTS.md` proíbe.
- **Silenciar o SC2329 no script:** põe uma diretiva de ferramenta no código
  distribuído.

## Consequências

- Mexer em `docs/PROTOCOL.md` sem atualizar as referências das skills reprova o
  CI do pacote.
- Os testes e o `shellcheck` precisam de macOS e Ubuntu hospedados; o Linux não
  roda os testes porque o `osascript` não existe lá.
- O workflow só roda no GitHub; aqui ele foi validado pelos mesmos comandos
  executados à mão.

## Compliance

1. `.github/workflows/package-ci.yml` dispara só com os caminhos da decisão 2 e
   não nomeia nada sob `app/`.
2. A lista de workflows do `relay-tui` (ADR-0003, conformidade 5) continua com
   dois arquivos, e este é o terceiro, do pacote.
3. Nenhum teste do pacote está em `skills/` (ADR-0005): ficam em `.agents/tests/`.

## Notes

Origem: spec `.specs/20261003-002-relogio-docs-ci-idioma-e-sinais.md`.
