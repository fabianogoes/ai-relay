# ADR-0004 — Fronteira e estrutura do `app/`

## Status

**Accepted** — 2026-09-07.

## Contexto

Hoje o Relay é instalado sem construir nada. `claude --plugin-dir ./relay`
funciona sobre um clone cru, e as instalações por symlink de `docs/INSTALL.md`
ligam `skills/` direto no projeto alvo. Nenhum passo intermediário, nenhuma
dependência, nenhum `npm install`.

A UI muda isso. Ela traz TypeScript, um framework, `node_modules` e, se ninguém
cuidar, um `package.json` na raiz que transforma o repositório num pacote npm.
Nada disso é necessário para quem só quer as skills — que é o uso do Relay
hoje e continuará sendo para a maioria.

A ADR-0002 estabeleceu duas camadas neste repositório: `skills/` é superfície de
pacote, distribuída pelos manifestos; `.agents/`, `.claude/` e
`.opencode/plugin/` são ferramenta de desenvolvimento, nunca distribuídas. A UI
não é nenhuma das duas. A ADR-0001 decidiu que a distribuição é por repositório,
sem loja, então a UI chega por clone — é **produto**, e precisa de uma camada
própria.

## Decisão

### 1. Um único diretório de contenção

Tudo da UI vive sob `app/`. Nada dela aparece na raiz.

```text
app/
  package.json          raiz do workspace, "private": true, nunca publicado
  AGENTS.md             instruções desta pasta (CLAUDE.md como symlink)
  relay-core/           puro: string entra, estado sai (ADR-0003)
  relay-host/           a única camada que toca o SO
  relay-ui/             a interface
```

**Por quê:** contenção transforma a promessa de não-interferência em algo
verificável. `git ls-files app/` mostra a UI inteira, e `rm -rf app/` devolve o
repositório ao estado anterior. Uma garantia que se testa vale mais que uma que
se afirma.

Os três nomes vêm da ADR-0001 ponto 1 e são mantidos literais, para que a
fronteira desenhada lá seja legível na árvore de diretórios.

### 2. Sem `package.json` na raiz

A raiz do workspace é `app/package.json`. A raiz do repositório permanece sem
manifesto npm.

**Por quê:** é o que mantém verdadeiro que instalar o Relay não exige
`npm install`. Um `package.json` na raiz também redefiniria o que o repositório
*é* — hoje ele é um pacote de skills, não um projeto Node.

### 3. Sem passo de build entre o clone e as skills

**Estreitada pela [ADR-0005](0005-framework-da-relay-ui.md) decisão 5.** A
redação original era "sem passo de build obrigatório" e alcançava qualquer
compilação, inclusive dentro de `app/`. O propósito era proteger a instalação
das skills; a redação foi além disso.

O que vale: nenhum passo de build entre `git clone` e usar as skills.
`claude --plugin-dir .` e os symlinks de `docs/INSTALL.md` funcionam num clone
cru. `app/` tem o próprio build, e ninguém precisa executá-lo para instalar ou
usar o Relay.

**Por quê:** o argumento da ADR-0001 ponto 2 contra Rust — "custaria uma segunda
toolchain no `git clone` de todo contribuidor" — vale para quem instala o
pacote, que é o uso majoritário e o que não escolheu pagar. Não vale para quem
desenvolve a interface, que aceitou esse custo ao entrar em `app/`.

### 4. `app/` carrega as próprias instruções

`app/AGENTS.md`, com `app/CLAUDE.md` como symlink, na mesma direção da raiz.

**Por quê:** é o padrão que a ADR-0002 decidiu — a instrução vive no gatilho
onde passa a valer, e o roteador da raiz gasta no máximo uma linha. O `AGENTS.md`
da raiz ganha um ponteiro para `app/` e nada mais.

### 5. A terceira camada é declarada, não inferida

| Camada | Diretórios | Como chega ao usuário |
| --- | --- | --- |
| Superfície de pacote | `skills/` | manifestos de plugin |
| Produto | `app/` | clone do repositório (ADR-0001) |
| Ferramenta deste repo | `.agents/`, `.claude/`, `.opencode/plugin/` | não chega; nunca distribuída |

**Por quê:** a ADR-0002 fixou duas camadas e a UI não cabia em nenhuma. Sem
declarar a terceira, `app/` seria empurrada para `skills/` — e passaria a ser
distribuída a quem só queria as skills.

### 6. `app/` nunca escreve nos cinco registros

Nenhum arquivo sob `app/` escreve em `.specs/` ou `.orchestration/`, deste
repositório ou de qualquer outro.

**Por quê:** é a ADR-0001 ponto 5 aplicada ao diretório, e a ADR-0003 já a fez
estrutural no `relay-core` ao proibir `node:fs`. Aqui ela vira regra de
diretório para cobrir também o `relay-host`, que **tem** acesso a disco: ele lê
os registros e escreve em qualquer outro lugar, nunca neles.

## Consequências

### Positivas

- A instalação atual continua funcionando sem alteração e sem `npm install`.
- A não-interferência é testável por comando, não por leitura.
- `app/` pode ser desenvolvida, quebrada e reconstruída sem risco para o pacote.
- Um contribuidor que só mexe em skills nunca precisa entrar em `app/`.

### Negativas e custos assumidos

- Um nível a mais de aninhamento em todo caminho da UI.
- Ferramenta que espera manifesto na raiz (alguns editores, algumas ações de CI)
  precisa ser apontada para `app/`.
- Uma terceira camada é mais para explicar que duas. O custo se paga porque a
  alternativa — empurrar `app/` para `skills/` — distribuiria a UI para quem só
  quer as skills.
- A decisão 3 era a mais provável de cair, e caiu: a ADR-0005 a estreitou ao
  escolher autoria em SFC com TypeScript, que exige compilação. A garantia que
  importava — instalar as skills sem build — permaneceu intacta.

### Consequência descartada explicitamente

Publicar `app/` no npm. `"private": true` é deliberado: a ADR-0001 decidiu
distribuição por repositório, e um pacote publicado criaria um segundo caminho
de distribuição com versionamento próprio para manter em dia.

## Conformidade

1. Não existe `package.json` na raiz do repositório.
2. `claude --plugin-dir .` e os symlinks de `docs/INSTALL.md` funcionam num
   clone cru, sem `npm install` e sem build.
3. `rm -rf app/` devolve o repositório a um estado funcional. Nenhum arquivo
   fora de `app/` **resolve** um caminho para dentro dele — link de markdown,
   import, symlink, entrada de manifesto ou linha de script. Prosa que
   simplesmente cita `app/`, como uma spec ou uma ADR, não é dependência.
   *Exceção da [ADR-0009](0009-relay-tui-observador-de-terminal-em-rust.md)
   decisão 4: os dois workflows do `relay-tui` em `.github/workflows/`, inertes
   quando `app/` não existe.*
4. Nada sob `app/` é referenciado por manifesto de distribuição.
5. Nenhum arquivo sob `app/` escreve em `.specs/` ou `.orchestration/`.
6. `app/CLAUDE.md` é symlink real para `app/AGENTS.md`, nunca arquivo regular.

## Notas

**Relação com a ADR-0005.** O framework é decisão separada e a mais provável de
ser superada. Esta ADR não o nomeia de propósito: a estrutura sobrevive à troca.
A única ligação é a decisão 3, que a ADR-0005 pode revisar.

**Escopo do que existe agora.** A spec `20260907-001` é visual e não tem
`relay-host` nem implementação de `relay-core`. Os três diretórios estão
decididos aqui; apenas os necessários ao marco atual são criados. Diretório
vazio não é versionável em git, e criar um por antecipação seria decoração.
