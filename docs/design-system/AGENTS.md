# Instruções desta pasta

Esta pasta é o design system do Relay. Ela traz as próprias instruções para não
ocupar contexto no `AGENTS.md` da raiz: você só lê isto quando chega aqui.

## Ordem de leitura

1. `README.md` — a autoridade. Tokens, componentes e as regras que os governam.
   Leia antes de qualquer mudança de interface. Para o `relay-tui`, leia
   `terminal.md`, que não depende do `README.md`.
2. `../adr/0001-arquitetura-inicial-da-ui.md` — a arquitetura que emoldura a UI.
   Leia quando a mudança for de estrutura, não de superfície.
3. `ui-proposal.md` — análise exploratória e alternativas descartadas (~25 KB).
   Leia **só** quando a decisão não estiver em nenhum dos dois acima.

## Nunca leia os `.html`

`design-system.html` e `prototypes/*.html` são artefatos de browser, não entrada
de agente. Os protótipos têm ~350 KB cada e consumiriam a maior parte de uma
janela de contexto. Nada neles está fora de registro: o que foi **decidido**
está na ADR-0001, o que foi apenas **observado** está em `ui-proposal.md`, e os
tokens estão no `README.md`.

Um guarda (`../../.agents/hooks/deny-design-system-html.sh`) nega essas leituras
no Claude Code e no OpenCode. O Codex não tem gate de ferramenta por
repositório: ali esta instrução é a única proteção. Se você bateu na negação,
pediu o arquivo errado — vá para o `README.md`.

Editar `design-system.html` continua permitido, e é obrigatório: ele é derivado
do `README.md` e muda junto quando um token muda.

## Ordem de mudança

Token ou componente muda **primeiro** no `README.md`, depois no código. Um
protótipo não vira token por existir; ele entra no `README.md` por decisão. O
`README.md` vence sobre protótipos e sobre código. As regras completas de
governança estão na seção 9 dele.
