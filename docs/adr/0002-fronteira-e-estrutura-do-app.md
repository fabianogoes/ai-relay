# ADR-0002 — Fronteira e estrutura do `app/`

## Status

**Accepted** — 2026-09-07. Reescrita em 2026-10-03 para o estado atual, depois
da remoção da interface web.

## Contexto

O Relay é instalado sem construir nada. `claude --plugin-dir ./relay` funciona
sobre um clone cru, e as instalações por symlink de `docs/INSTALL.md` ligam
`skills/` direto no projeto alvo. Nenhum passo intermediário, nenhuma
dependência.

A interface muda isso: o `relay-tui` traz Rust, Cargo e uma matriz de CI. Nada
disso é necessário para quem só quer as skills, que é o uso do Relay hoje e
continuará sendo para a maioria.

A ADR-0001 estabeleceu duas camadas neste repositório: `skills/` é superfície de
pacote, distribuída pelos manifestos; `.agents/` e `.claude/` são ferramenta de
desenvolvimento, nunca distribuídas. A interface não é nenhuma das duas: é
**produto**, e precisa de uma camada própria.

## Decisão

### 1. Um único diretório de contenção

Tudo da interface vive sob `app/`. Nada dela aparece na raiz.

```text
app/
  AGENTS.md     instruções desta pasta (CLAUDE.md como symlink)
  README.md
  relay-tui/    o painel de terminal (ADR-0003)
```

**Por quê:** contenção transforma a promessa de não-interferência em algo
verificável. `git ls-files app/` mostra a interface inteira, e `rm -rf app/`
devolve o repositório a um estado funcional. Uma garantia que se testa vale mais
que uma que se afirma.

### 2. Sem passo de build entre o clone e as skills

Nenhum passo de build entre `git clone` e usar as skills.
`claude --plugin-dir .` e os symlinks de `docs/INSTALL.md` funcionam num clone
cru. `app/` tem o próprio build, e ninguém precisa executá-lo para instalar ou
usar o Relay; o `relay-tui` chega pronto, por download (ADR-0003).

**Por quê:** o custo de uma toolchain recai sobre quem desenvolve a interface,
que o aceitou ao entrar em `app/`, e nunca sobre quem só instala o pacote.

### 3. Sem manifesto de pacote na raiz

A raiz do repositório não tem `package.json`, `Cargo.toml` nem outro manifesto
de build. O manifesto do `relay-tui` é `app/relay-tui/Cargo.toml`.

**Por quê:** é o que mantém verdadeiro que instalar o Relay não exige build. Um
manifesto na raiz também redefiniria o que o repositório *é*: hoje ele é um
pacote de skills.

### 4. `app/` carrega as próprias instruções

`app/AGENTS.md`, com `app/CLAUDE.md` como symlink, na mesma direção da raiz. É o
padrão da ADR-0001: a instrução vive no gatilho onde passa a valer, e o `AGENTS.md`
da raiz ganha um ponteiro para `app/` e nada mais.

### 5. A terceira camada é declarada, não inferida

| Camada | Diretórios | Como chega ao usuário |
| --- | --- | --- |
| Superfície de pacote | `skills/` | manifestos de plugin |
| Produto | `app/` | clone do repositório; o `relay-tui` também por binário em Release |
| Ferramenta deste repo | `.agents/`, `.claude/` | não chega; nunca distribuída |

**Por quê:** sem declarar a terceira, `app/` seria empurrada para `skills/` e
passaria a ser distribuída a quem só queria as skills.

### 6. `app/` nunca escreve nos cinco registros

Nenhum arquivo sob `app/` escreve em `.specs/` ou `.orchestration/`, deste
repositório ou de qualquer outro. Toda mutação passa por uma skill num harness.

## Consequências

### Positivas

- A instalação das skills continua sem build e sem dependência.
- A não-interferência é testável por comando, não por leitura.
- `app/` pode ser desenvolvida, quebrada e reconstruída sem risco para o pacote.
- Um contribuidor que só mexe em skills nunca precisa entrar em `app/`.

### Negativas e custos assumidos

- Um nível a mais de aninhamento em todo caminho da interface.
- Ferramenta que espera manifesto na raiz (alguns editores, algumas ações de CI)
  precisa ser apontada para `app/relay-tui/`.
- Uma terceira camada é mais para explicar que duas; o custo se paga porque a
  alternativa distribuiria a interface a quem só quer as skills.

## Conformidade

1. Não existe manifesto de build na raiz do repositório.
2. `claude --plugin-dir .` e os symlinks de `docs/INSTALL.md` funcionam num
   clone cru, sem build.
3. `rm -rf app/` devolve o repositório a um estado funcional. Nenhum arquivo
   fora de `app/` **resolve** um caminho para dentro dele — link de markdown,
   import, symlink, entrada de manifesto ou linha de script. Prosa que
   simplesmente cita `app/`, como uma spec ou uma ADR, não é dependência.
   *Exceção da [ADR-0003](0003-relay-tui-observador-de-terminal-em-rust.md)
   decisão 6: os dois workflows do `relay-tui` em `.github/workflows/`, inertes
   quando `app/` não existe.*
4. Nada sob `app/` é referenciado por manifesto de distribuição.
5. Nenhum arquivo sob `app/` escreve em `.specs/` ou `.orchestration/`.
6. `app/CLAUDE.md` é symlink real para `app/AGENTS.md`, nunca arquivo regular.

## Notas

A primeira versão desta decisão continha três pacotes TypeScript da interface
web (`relay-core`, `relay-host`, `relay-ui`) num workspace npm em
`app/package.json`. Eles saíram em 2026-10-03; a fronteira continua a mesma.
