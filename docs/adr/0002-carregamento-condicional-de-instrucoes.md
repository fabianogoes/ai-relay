# ADR-0002 — Carregamento condicional de instruções de agente

## Status

**Accepted** — 2026-09-07.

## Contexto

O `AGENTS.md` da raiz é lido inteiro no início de toda sessão, em todo harness.
Uma decisão anterior já o transformou de réplica em roteador: ele aponta para o
que deve ser lido e quando, em vez de resumir o conteúdo. Isso resolveu a
duplicação, mas não o custo — o roteador continua pagando por informação que
vale para uma fração pequena das sessões.

A pasta `docs/design-system/` tornou o problema concreto:

| Arquivo | Tamanho | Custo estimado se lido |
| --- | --- | --- |
| `README.md` | 12,4 KB | ~3,5 mil tokens |
| `ui-proposal.md` | 25,0 KB | ~7 mil tokens |
| `design-system.html` | 24,9 KB | ~7 mil tokens |
| `prototypes/*.html` (2 arquivos) | 689 KB | da ordem de 180 mil tokens |

Um único protótipo aberto por engano consome a maior parte de uma janela de
contexto. O `AGENTS.md` gastava oito linhas **sempre presentes** para avisar
disso — e o aviso não obrigava nada. O custo estava invertido em duas direções
ao mesmo tempo: paga-se em toda sessão por uma regra que vale em poucas, e
ainda assim a regra é apenas conselho.

O mecanismo que se supõe existir para resolver isso, um diretório de regras com
ativação por *glob* (`.claude/rules/`), **não existe no Claude Code**. É
convenção do Cursor (`.cursor/rules/*.mdc`). Um arquivo ali seria ignorado em
silêncio, que é o pior resultado possível: a aparência de proteção sem a
proteção. Os mecanismos que existem de fato são três, e cada um dispara num
momento diferente:

- **skill** — dispara pela *intenção*, antes de qualquer arquivo ser aberto;
- **`AGENTS.md` aninhado** — dispara pelo *acesso* à pasta, depois do fato;
- **hook de ferramenta** — dispara pela *tentativa de leitura*, e é o único que
  vincula.

Duas restrições são dadas, não estão sob avaliação. A primeira: o Relay é
neutro de harness (Claude Code, Codex, OpenCode), então uma solução que só
funcione num deles não serve como padrão do projeto. A segunda:
`.codex-plugin/plugin.json` declara `"skills": "./skills/"`, de modo que tudo
naquele diretório é distribuído para quem instala o Relay — ferramenta de
desenvolvimento deste repositório não pode morar lá.

## Decisão

### 1. A instrução vive onde ela vale, não na raiz

Cada regra é colocada no gatilho que corresponde ao momento em que ela passa a
ser necessária, e a raiz guarda no máximo um ponteiro de uma linha. Para o
design system, dois gatilhos são usados juntos porque cobrem falhas diferentes:

```text
docs/design-system/AGENTS.md  → acesso à pasta (chegou por busca)
guarda PreToolUse             → tentativa de leitura (conselho ignorado)
```

Não é redundância: o `AGENTS.md` aninhado não obriga, e o guarda só fala quando
a leitura já foi tentada. A skill `relay-design-system`, que cobria a intenção
antes do primeiro arquivo aberto, existiu e foi removida em 2026-10-03 (ver
Notas).

### 2. Uma regra, várias cascas

A regra é um script — `.agents/hooks/deny-design-system-html.sh`. O
`.claude/settings.json` e o `.opencode/plugin/design-system-guard.ts` são
apenas cascas que o invocam e traduzem o resultado para o formato de cada
harness.

O motivo é o de sempre: três cópias de uma regra divergem, e a divergência
aparece como comportamento diferente entre harnesses, que é caro de
diagnosticar. O script também é executável à mão, o que o torna testável sem
subir nenhum agente.

### 3. O nome neutro guarda o conteúdo; o harness recebe symlink

Mesmo padrão já usado na raiz, onde `AGENTS.md` é o arquivo e `CLAUDE.md` é o
symlink:

```text
.agents/skills/<skill>/     arquivo real
.claude/skills/<skill>      → symlink
.opencode/skills/<skill>    → symlink
```

O symlink é **por skill**, nunca do diretório inteiro. Linkar o diretório falha
quando ele já existe: `ln -s <origem>/skills .claude/skills` aninha o link como
`.claude/skills/skills` em vez de substituí-lo, e um projeto que tenha skills
próprias sempre cai nesse caso. As instruções de instalação do Relay tinham
essa forma e foram corrigidas junto.

### 4. Ferramenta de repositório não é superfície de pacote

`.agents/`, `.claude/` e `.opencode/plugin/` servem ao desenvolvimento *deste*
repositório e nunca são distribuídos. A fronteira é verificável: os manifestos
de distribuição referenciam `./skills/` e nada mais.

### 5. O guarda é deliberadamente estreito

O guarda nega leitura de `.html` sob a pasta e, para comandos de shell, apenas
quando o comando **começa** com um despejador (`cat`, `head`, `tail`, `less`,
`more`, `bat`, `nl`, `strings`) e **não** tem redirecionamento — o que separa
`cat <arquivo>` de `cat > <arquivo>`.

A versão ampla foi tentada primeiro e bloqueou a escrita do próprio documento
que descreve a regra, porque o documento cita o caminho. Um guarda que impede
falar sobre o guarda é um guarda que alguém desliga. Num repositório cujo
produto é documentação, o falso positivo é pior que o vazamento residual: o
falso positivo acontece por acidente e a cada tentativa, enquanto o contorno
que sobra (`cd` na pasta antes de ler) exige intenção — e contra intenção,
nenhum hook protege.

Por isso o guarda nega **leitura**, nunca escrita: `design-system.html` é
derivado do `README.md` e precisa ser editável, já que a regra de governança
manda atualizá-lo na mesma mudança que altera um token.

### 6. Enforcement desigual é aceito e registrado, não escondido

O Codex não tem gate de ferramenta por repositório: o controle dele é sandbox e
política de aprovação. Ali, o `AGENTS.md` aninhado é a única proteção, e ela é
apenas instrução.

A alternativa seria abandonar o hook nos dois harnesses que o suportam para
manter simetria. Ela foi descartada: paridade obtida rebaixando todo mundo ao
mínimo comum protege menos e não protege ninguém melhor.

## Consequências

### Positivas

- A seção de roteamento do `AGENTS.md` caiu de 20 para 16 linhas, e o que saiu
  não foi perdido — foi movido para onde dispara sozinho.
- Os 689 KB de protótipo deixam de ser alcançáveis por acidente nos dois
  harnesses com hook.
- O padrão é repetível: a próxima regra desse tipo tem forma pronta — script no
  neutro, casca por harness, symlink por item.
- A regra é testável sem agente, porque o script roda sozinho.

### Negativas e custos assumidos

- **O `AGENTS.md` não encolheu em bytes.** Só esta mudança o levou de 3842 para
  3940: a seção de roteamento perdeu quatro linhas e a fronteira de pacote
  gastou duas. A economia é de contexto *carregado*, não de tamanho de arquivo,
  e é honesto dizer isso.
- Três arquivos em três diretórios para uma regra. Com uma regra só, a
  estrutura é maior que o problema; ela se paga a partir da segunda. Aceita
  porque a alternativa já tinha falhado: o aviso puramente textual existia e não
  impedia nada.
- O plugin do OpenCode depende de `@opencode-ai/plugin` e do hook
  `tool.execute.before`, API de terceiro sem contrato de estabilidade. Uma
  mudança lá quebra a casca, não a regra.
- **O plugin do OpenCode nunca foi executado no OpenCode.** Foi verificado por
  parsing e por teste de comportamento com um `$` simulado, cobrindo dez casos.
  A verificação de ponta a ponta está pendente.
- O contorno por `cd` na pasta continua aberto, por decisão (ponto 5).
- Symlinks não sobrevivem a checkout em Windows sem `core.symlinks=true`. A
  ADR-0001 assume macOS primeiro, então isso é dívida registrada e não bloqueio.

### Consequência descartada explicitamente

Bloquear também os `.md` da pasta. O `README.md` existe para ser lido antes de
uma mudança de interface; os ~3,5 mil tokens dele são o preço de fazer a
mudança certa, não um desperdício a ser evitado.

## Conformidade

1. Uma regra de leitura tem exatamente **uma** implementação executável. Uma
   casca de harness pode traduzir o resultado; não pode reimplementar a decisão.
2. Nenhum arquivo sob `.agents/`, `.claude/` ou `.opencode/plugin/` é
   referenciado por manifesto de distribuição. Os manifestos expõem `./skills/`
   e nada mais.
3. Toda skill de desenvolvimento nasce em `.agents/skills/`, com symlink **por
   skill** em `.claude/skills/` e `.opencode/skills/`. Symlink de diretório
   inteiro é proibido, aqui e nas instruções de instalação.
4. Um guarda nega leitura, nunca escrita. Citar um caminho protegido não é
   lê-lo: documentar a regra e editar um arquivo derivado continuam permitidos.
5. Instrução de pasta é `AGENTS.md` com `CLAUDE.md` como symlink, na mesma
   direção da raiz.
6. Quando um harness não puder cumprir uma regra, a assimetria é registrada no
   documento e no ADR. Nenhum texto do projeto afirma proteção que aquele
   harness não tem.
7. O `AGENTS.md` da raiz aponta; não repete o conteúdo do gatilho. Uma regra que
   caiba num gatilho não volta para a raiz.

## Notas

**Mecanismo inexistente.** `.claude/rules/` com ativação por *glob* é convenção
do Cursor e do Windsurf, não do Claude Code. Está registrado aqui porque é uma
suposição plausível o bastante para reaparecer, e porque falharia em silêncio.

**Relação com a ADR-0001.** Aquela decide a arquitetura da UI; esta decide como
as instruções sobre essa UI chegam ao agente. São camadas diferentes e não se
sobrepõem: nada aqui altera a regra de que a aplicação nunca escreve nos cinco
registros do protocolo.

**Escopo.** Esta ADR trata do carregamento de instruções, não do conteúdo delas.
O que o design system decide sobre tokens e componentes está em
`docs/design-system/README.md`, e a autoridade dele sobre protótipos e código
não muda por causa desta decisão.

**Remoção da skill (2026-10-03).** A skill `relay-design-system` foi removida
por não se pagar: duplicava a ordem de leitura que o `docs/design-system/AGENTS.md`
já carrega e ocupava espaço na lista de skills de toda sessão. A decisão 1 fica
com dois gatilhos; o padrão da decisão 3 continua valendo para qualquer skill de
desenvolvimento futura. Com o projeto em estado inicial, o ajuste foi feito aqui
em vez de numa ADR de supersessão.

**Pendente.** A execução do plugin no OpenCode real, listada acima como custo
assumido, é a única verificação que falta para a decisão estar plenamente
exercida.
