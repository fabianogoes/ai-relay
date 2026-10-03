# ADR-0011 — relay-tui-split: skill de pacote fora do protocolo

## Status

**Accepted** — 2026-10-02.

Não revisa nenhuma ADR anterior. Estende a ADR-0002 (como uma instrução chega à
sessão) e respeita a decisão 7 da ADR-0009 (o `relay-tui` é só um observador).

## Contexto

Até aqui toda skill de `skills/relay-*` existe para ler ou escrever um dos cinco
registros do protocolo, e o `docs/PROTOCOL.md` lista a responsabilidade de cada
uma. A spec `20261002-003` pede uma skill de outra natureza: `relay-tui-split`
só abre um painel de terminal ao lado do harness rodando o `relay-tui`. Ela não
lê nem escreve registro algum. Além disso traz o primeiro script executável
publicado em `skills/`, o `open-split.sh`, que roda no shell do harness.

Duas perguntas estruturais ficam em aberto: uma skill que não toca o protocolo
pertence ao pacote? E um script publicado ao lado de uma skill é aceitável, já
que até agora o pacote era só Markdown?

## Decisão

1. **A skill é superfície de pacote, não do protocolo.** Vive em `skills/`, vai
   nos manifestos (que publicam `./skills/` inteiro) e nas instalações. O
   `docs/PROTOCOL.md` não muda: sua lista cobre as skills que tocam registros, e
   esta é documentada como fora dele.
2. **Nome `relay-tui-split`, distinto do binário.** `relay-tui` é o programa;
   "use relay-tui" seria lido como "rode o binário". O nome da skill descreve a
   ação.
3. **Um script POSIX `sh`, sem dependência além do terminal.** O script fica em
   `skills/relay-tui-split/scripts/`, não instala nada e não escreve registro.
   Seus testes ficam em `.agents/tests/`, fora de `skills/`, porque são
   ferramenta do repositório e não pacote (ADR-0002).
4. **Sem Relay CLI.** O script só detecta o terminal e abre o painel; não deriva
   estado nem escreve. Isso preserva a regra de não criar um CLI antes de o
   protocolo ser validado.

## Consequências

- O pacote deixa de ser só Markdown: quem instala as skills recebe um script
  executável. O risco é mantê-lo coerente com terminais que mudam (o Warp é o
  caso mais frágil); a spec o confina a uma tabela de detecção com `--dry-run`.
- A skill funciona sem o `relay-tui`: sem ele, imprime o link do guia e sai.
- Outras skills utilitárias, que não tocam registros, têm agora um lugar e uma
  regra: pacote, fora do protocolo, com os testes fora de `skills/`.

## Compliance

1. Nem a skill nem o script leem ou escrevem os cinco registros.
2. `skills/relay-tui-split/SKILL.md` tem menos de 40 linhas e o `PROTOCOL.md` não
   a cita como responsável por registro.
3. Os testes do script estão em `.agents/tests/` e nenhum arquivo de teste está
   em `skills/`.
4. O script é POSIX `sh` e roda no `/bin/sh` do macOS e no `dash`.

## Notes

Origem: spec `.specs/20261002-003-skill-relay-tui-abre-em-split.md`.
