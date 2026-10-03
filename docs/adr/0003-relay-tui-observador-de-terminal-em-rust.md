# ADR-0003 — relay-tui: observador de terminal em Rust

## Status

**Accepted** — 2026-10-02. Reescrita em 2026-10-03 para o estado atual: a
interface web foi removida e o `relay-tui` passou a ser a única interface, com o
único leitor do protocolo.

## Contexto

A primeira interface do Relay foi web: um host local em TypeScript servindo uma
UI no navegador. Dela, só o observador read-only funcionou no uso real (o
harness numa tela, o estado durável mudando na outra sem recarregar); a execução
integrada (lançador, preflight, PTY, terminal no navegador) falhou.

O uso que importa é o split de terminal: harness de um lado, painel do outro,
sem navegador, porta nem token. O requisito que define a tecnologia é a
distribuição: **um único download**, sem clonar o repositório, sem runtime e sem
gerenciador de pacotes, com leveza como piso. A interface web foi removida em
2026-10-03, e o `relay-tui` ficou sozinho.

## Decisão

### 1. `relay-tui` é um binário Rust, somente leitura

Vive em `app/relay-tui/` (projeto Cargo), com `ratatui`, `crossterm` e `notify`.
Observa um workspace (`--workspace <path>` ou o diretório corrente), nunca
escreve nos cinco registros e não lança processo. As teclas só mudam o que a
tela mostra (ADR-0004). A ADR-0002 já aceita toolchain própria dentro de `app/`;
quem só quer as skills continua sem build.

### 2. Rust, e não Go nem TypeScript compilado

| | Binário | Core | Multiplataforma |
| --- | --- | --- | --- |
| Rust + ratatui | ~2–5 MB | reescrito | matriz de CI |
| Go + Bubble Tea | ~5–10 MB | reescrito | cross-compile trivial |
| TS com `bun build --compile` | ~60–100 MB | reaproveitado | alvos de cross-compile |

**Por quê:** os três cumprem "um download". O TS compilado era o único que
reaproveitava o core da interface web, mas leva o runtime no binário e combina
Ink com Bun, com menos estrada. Entre Rust e Go a diferença técnica é pequena;
Rust foi escolhido pela preferência do dono do projeto e porque o core é a peça
que se reescreve com menos risco, por ser pura e testada.

### 3. O core é puro e entrega um estado que a view só desenha

O módulo `core` faz parse dos cinco registros, roda as 13 verificações de
integridade do `docs/PROTOCOL.md` e deriva o `RelayState`. As regras do contrato:

- **Conteúdo entra, estado sai.** `derive_state` recebe o texto dos registros,
  nunca um caminho; o `core` não lê disco nem ambiente. Só o módulo `workspace`
  lê arquivos. Assim "a interface nunca escreve um registro" é propriedade
  estrutural, não lembrete, e todo teste do core roda sem sistema de arquivos.
- **`inconsistent` é outra forma, não outro status.** `RelayState` é
  `Ok { status, handoff, todo, backlog, … }` ou `Inconsistent { violations }`,
  com `violations` não vazio. A view não consegue ler handoff ou TODO de um
  estado inconsistente: o compilador recusa.
- **Disponibilidade é derivada no core.** Cada entrada de checklist traz
  `available`; a view nunca recalcula a regra.
- **Contagem sim; posição, fila e percentual não.** O estado expõe `completed` e
  `total`, nunca `progress`, `position`, `priority` ou `order`: o protocolo não
  define ordem nem prioridade, e o que o estado expõe a tela uma hora desenha.
- **Proveniência é instante; tempo relativo é apresentação.** O handoff carrega o
  `Updated` em RFC 3339; "há 4 min" é formatado na view, com o relógio injetado.
- **Tom e tela ficam fora do estado.** O estado carrega `status`, não cor nem
  rótulo; o `app/relay-tui/DESIGN.md` decide como cada status aparece.
- **Uma violação diz qual verificação falhou.** `Violation { check, detail,
  records }`, com `check` estável, na ordem do `docs/PROTOCOL.md`:

| `check` | Verificação |
| --- | --- |
| `handoff-names-no-pending-todo` | Handoff não vazio não nomeia um item pendente do TODO |
| `backlog-id-mismatch` | ID de backlog ativo inexistente no backlog, ou handoff, TODO e backlog discordam do mesmo backlog ID |
| `spec-path-mismatch` | Tarefa ativa sem entrada de backlog confrontável, sem spec, ou com spec diferente do handoff |
| `handoff-harness-invalid` | Handoff não vazio sem `Harness` ou fora do formato permitido |
| `handoff-updated-invalid` | Handoff não vazio sem `Updated` ou fora do RFC 3339 exigido |
| `multiple-handoffs` | Existe mais de um registro de handoff corrente |
| `todo-cleared-before-changelog` | Item saiu do handoff antes do resultado entrar no changelog |
| `backlog-done-with-pending-todo` | Backlog `done` com item de TODO ativo não concluído |
| `unknown-marker` | Marcador desconhecido, ou item concluído que não é `[x]` |
| `needs-unknown-id` | `needs` referencia ID ausente do mesmo registro |
| `needs-cycle` | Relação de `needs` contém ciclo |
| `needs-incomplete-on-done` | Entrada `[x]` cujo `needs` não está todo `[x]` |
| `criteria-without-evidence` | Toda entrada de backlog de uma spec está `[x]` e um critério de aceite dela não é nomeado por nenhum registro de changelog |

Renomear um `check` muda os fixtures e é decisão desta ADR.

### 4. Fixtures com estado esperado

Em `app/relay-tui/tests/fixtures/<caso>/` cada caso é um workspace em disco e um
`expected.json` com o `RelayState` que o core tem de derivar: um caso por status
(`status-*`) e um por verificação de integridade (`check-*`).
`tests/derive_state.rs` compara cada um. Os casos valem só se concordam com o
`docs/PROTOCOL.md`: se discordarem, o defeito é do caso. Os workspaces `status-*`
também servem de entrada para os testes da view.

**Por quê o formato neutro, sem serializar Rust:** os casos foram gravados quando
havia dois leitores (TypeScript e Rust) e continuam legíveis por qualquer
linguagem. Se um segundo leitor voltar a existir, eles voltam a ser suíte de
conformidade compartilhada sem conversão.

### 5. Reatividade: o workspace é relido inteiro depois de 150 ms de silêncio

O `relay-tui` vigia `.orchestration/` e `.specs/`, não o repositório inteiro.
Todo evento marca a tela como `atualizando`; depois de 150 ms sem evento, o
workspace é relido por inteiro e a tela volta a `atualizado`. Nunca se aplica um
evento isolado: uma skill escreve vários registros em sequência, e um estado
derivado no meio dessa sequência seria falso. Um diretório sem `.orchestration/`
mostra "Não é um workspace Relay" e continua vigiando.

### 6. Distribuição por binário em Release

Uma tag `relay-tui-v<versão>` publica no GitHub Release um binário por alvo.
**O requisito é o macOS** (arm64 e x64), com `cargo test` no macOS no CI. Linux
(x64 e arm64, `musl`, estático) é **melhor esforço**: entra no build quando
possível e uma falha nele nunca bloqueia o release nem o CI. **O Windows está
fora por enquanto.**

Um binário baixado recebe `com.apple.quarantine` no macOS. A primeira versão
**não assina nem notariza**; documenta o contorno
(`xattr -d com.apple.quarantine`). Assinatura exige uma conta Apple Developer
paga e fica como decisão adiada, junto com auto-update e pacotes (brew, nix).

**Windows adiado (2026-10-02).** O job de Windows do primeiro CI falhou duas
vezes seguidas por diferenças de plataforma e poluía o pipeline sem que ninguém o
usasse. Saiu do CI, do release, do `package.sh` e do README. Para retomar:
reincluir `windows-latest` na matriz de `best-effort` do CI e
`{ os: windows-latest, target: x86_64-pc-windows-msvc }` na do release; recuperar
do commit `6c8f484` o ramo `*-windows-*` do `scripts/package.sh` (`relay-tui.exe`
e `.zip` via `7z`); reincluir a linha do README e o aviso do SmartScreen; e
tratar a falha seguinte do `cargo test` (o PTY do teste ponta a ponta também
nunca rodou lá). O CRLF dos registros (limite conhecido do README) é o problema
mais provável de ser exigido antes.

**Exceção à conformidade 3 da ADR-0002.** Um workflow só é lido pelo GitHub em
`.github/workflows/`, fora de `app/`, e precisa nomear `app/relay-tui` para
construir. São exatamente dois arquivos, ferramenta do repositório: o de CI só
dispara com mudanças em `app/relay-tui/**` e o de release só com tags
`relay-tui-v*`. Removido `app/`, nenhum dos dois dispara, de modo que
`rm -rf app/` continua devolvendo o repositório a um estado funcional.

### 7. O nome é `relay-tui`, e não é um Relay CLI

O `AGENTS.md` proíbe criar um Relay CLI antes de o protocolo ser validado em
repositórios reais. Um CLI do Relay interpretaria comandos do protocolo ou
mutaria registros. O `relay-tui` não interpreta comando algum, não escreve e
tem um único propósito: desenhar o estado derivado.

### 8. Aparência e paleta

Layout de cartões, paleta própria do terminal e fundo herdado dele. Os valores,
o mapeamento de status, as telas e as teclas vivem em `app/relay-tui/DESIGN.md`,
que tem autoridade sobre como cada estado aparece; esta ADR não os repete.

### 9. Evolução considerada: canal de perguntas por arquivo

A ideia: o harness, ao rodar `relay-spec`, escreve uma pergunta num arquivo local
(opções com checkboxes) e espera; a TUI a exibe, o usuário responde, e a TUI
escreve a resposta noutro arquivo que o harness lê. Harness e TUI nunca falam
entre si; encontram-se em arquivos de escritor único (`ask-<id>.md`, do harness;
`answer-<id>.md`, da TUI), com escrita atômica.

Não faz parte desta entrega. Exigiria:

- **Protocolo primeiro.** A TUI passaria a escrever, e a decisão 1 diz que ela
  nunca escreve. A saída coerente é um canal **efêmero que não é um registro**
  (sem memória durável, fora do git, apagado após o uso), definido no
  `docs/PROTOCOL.md` antes de qualquer skill.
- **Skills.** `relay-spec` manda usar a interação nativa do harness; ganharia um
  modo de canal por arquivo com fallback à pergunta nativa quando a TUI não
  estiver presente.
- **Um spike antes de qualquer spec.** A pergunta é se uma skill consegue
  esperar um arquivo de forma confiável: uma skill é instrução para o modelo e
  só espera por um comando de shell bloqueante. No Claude Code o `Bash` tem
  limite de 10 minutos, com execução em segundo plano e `Monitor` como
  alternativa. O spike valida o **Claude Code primeiro**; Codex e OpenCode só
  entram depois dessa validação.

Considerados e **descartados**: embutir o harness num PTY dentro da TUI (a
superfície que já falhou na interface web) e uma interface de chat sobre os
modos headless dos harnesses (um protocolo por harness, e quebra a
neutralidade).

## Consequências

### Positivas

- Um download resolve a instalação; nada de runtime, repositório ou gerenciador.
- Leveza e partida instantânea.
- Um leitor só: cada mudança de gramática do protocolo é uma edição no `core` e
  nos fixtures, não duas implementações.
- Nenhuma superfície de execução volta pela porta dos fundos.

### Negativas e custos assumidos

- Uma toolchain (Cargo) e uma matriz de CI em `app/`, com o macOS como requisito
  e Linux como melhor esforço.
- Binários sem assinatura exigem um passo manual de quem baixa, no macOS.
- O `core` em Rust é o único leitor: um erro nele não tem uma segunda leitura
  que o denuncie. Os fixtures e o `docs/PROTOCOL.md` são a referência.
- Algumas peculiaridades do primeiro leitor (TypeScript) foram mantidas de
  propósito para os fixtures continuarem valendo, como ignorar linhas terminadas
  em CRLF (limite conhecido no README do crate).

## Compliance

1. `app/relay-tui/` não contém chamada de escrita em `.specs/` nem em
   `.orchestration/`, nem criação de processo (`tests/read_only.rs`).
2. O `core` não acessa disco nem ambiente; só `workspace` lê arquivos
   (`tests/purity.rs`).
3. `tests/fixtures/` tem os sete casos de status e um caso por verificação de
   integridade, e `tests/derive_state.rs` passa em todos.
4. A tag de release publica os binários de macOS arm64 e x64 e o CI roda
   `cargo test` no macOS: é o requisito. Linux é melhor esforço e uma falha nele
   não impede o release nem o CI.
5. `.github/workflows/` tem só os dois arquivos do `relay-tui`, com os filtros
   de caminho e de tag da decisão 6.
6. O canal de perguntas só passa a existir depois de uma emenda do
   `docs/PROTOCOL.md` e do spike do Claude Code.
