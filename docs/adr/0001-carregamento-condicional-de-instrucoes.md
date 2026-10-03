# ADR-0001 — Carregamento condicional de instruções de agente

## Status

**Accepted** — 2026-10-03.

## Contexto

O `AGENTS.md` da raiz é lido inteiro no início de toda sessão, em todo harness.
Ele é um roteador: aponta para o que deve ser lido e quando, em vez de resumir
o conteúdo. Mesmo assim, toda linha nele é paga em toda sessão, inclusive as que
valem para uma fração pequena delas.

O mecanismo que se supõe existir para resolver isso, um diretório de regras com
ativação por *glob* (`.claude/rules/`), **não existe no Claude Code**. É
convenção do Cursor (`.cursor/rules/*.mdc`). Um arquivo ali seria ignorado em
silêncio, que é o pior resultado possível: a aparência de instrução sem a
instrução. Os mecanismos que existem de fato disparam em momentos diferentes:

- **skill** — dispara pela *intenção*, antes de qualquer arquivo ser aberto;
- **`AGENTS.md` aninhado** — dispara pelo *acesso* à pasta;
- **hook de ferramenta** — dispara pela *tentativa* de uma ação, e é o único que
  vincula.

Duas restrições são dadas. O Relay é neutro de harness (Claude Code, Codex,
OpenCode), então uma solução que só funcione num deles não serve como padrão. E
`.codex-plugin/plugin.json` declara `"skills": "./skills/"`: tudo naquele
diretório é distribuído a quem instala o Relay, então ferramenta de
desenvolvimento deste repositório não pode morar lá.

## Decisão

### 1. A instrução vive onde ela vale, não na raiz

Cada regra fica no gatilho que corresponde ao momento em que ela passa a ser
necessária, e a raiz guarda no máximo um ponteiro de uma linha. Hoje o caso é o
`app/AGENTS.md`: as regras da interface só entram em contexto quando o agente
chega em `app/`.

### 2. O nome neutro guarda o conteúdo; o harness recebe symlink

O mesmo padrão da raiz, onde `AGENTS.md` é o arquivo e `CLAUDE.md` é o symlink,
vale para pastas e para skills de desenvolvimento:

```text
.agents/skills/<skill>/     arquivo real
.claude/skills/<skill>      → symlink
```

O symlink é **por item**, nunca do diretório inteiro. Linkar o diretório falha
quando ele já existe: `ln -s <origem>/skills .claude/skills` aninha o link como
`.claude/skills/skills` em vez de substituí-lo, e um projeto que tenha skills
próprias sempre cai nesse caso. As instruções de instalação seguem a mesma
regra.

### 3. Uma regra executável, várias cascas

Quando uma regra precisar vincular por hook, ela é **um** script em `.agents/`,
e cada harness recebe apenas uma casca que o invoca e traduz o resultado. Três
cópias de uma regra divergem, e a divergência aparece como comportamento
diferente entre harnesses, caro de diagnosticar. Hoje nenhuma regra usa hook.

### 4. Ferramenta de repositório não é superfície de pacote

`.agents/` e `.claude/` servem ao desenvolvimento *deste* repositório e nunca são
distribuídos. A fronteira é verificável: os manifestos de distribuição
referenciam `./skills/` e nada mais.

### 5. Enforcement desigual é registrado, não escondido

O Codex não tem gate de ferramenta por repositório: o controle dele é sandbox e
política de aprovação. Uma regra que dependa de hook é só instrução ali, e isso
é dito no documento da regra. Paridade obtida rebaixando todo harness ao mínimo
comum protege menos e não protege ninguém melhor.

## Consequências

### Positivas

- O roteador da raiz fica curto; o que saiu dele dispara sozinho onde vale.
- O padrão é repetível: instrução de pasta em `AGENTS.md` aninhado, regra
  executável no neutro com casca por harness, symlink por item.

### Negativas e custos assumidos

- Uma regra de pasta só é vista por quem entra na pasta; quem decide sem abrir
  arquivo algum não a lê. Para isso existe a skill, que dispara por intenção.
- Symlinks não sobrevivem a checkout em Windows sem `core.symlinks=true`. O
  projeto assume macOS primeiro, então isso é dívida registrada e não bloqueio.

## Conformidade

1. Uma regra de hook tem exatamente **uma** implementação executável. Uma casca
   de harness pode traduzir o resultado; não pode reimplementar a decisão.
2. Nenhum arquivo sob `.agents/` ou `.claude/` é referenciado por manifesto de
   distribuição. Os manifestos expõem `./skills/` e nada mais.
3. Toda skill de desenvolvimento nasce em `.agents/skills/`, com symlink **por
   skill** em `.claude/skills/`. Symlink de diretório inteiro é proibido, aqui e
   nas instruções de instalação.
4. Instrução de pasta é `AGENTS.md` com `CLAUDE.md` como symlink, na mesma
   direção da raiz.
5. Quando um harness não puder cumprir uma regra, a assimetria é registrada.
   Nenhum texto do projeto afirma proteção que aquele harness não tem.
6. O `AGENTS.md` da raiz aponta; não repete o conteúdo do gatilho. Uma regra que
   caiba num gatilho não volta para a raiz.

## Notas

**Mecanismo inexistente.** `.claude/rules/` com ativação por *glob* é convenção
do Cursor e do Windsurf, não do Claude Code. Está registrado aqui porque é uma
suposição plausível o bastante para reaparecer, e porque falharia em silêncio.
