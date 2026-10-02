# ADR-0009 — relay-tui: observador de terminal em Rust

## Status

**Accepted** — 2026-10-02.

Revisa a ADR-0001 em dois pontos, **só para o `relay-tui`**: a decisão 2
(TypeScript ponta a ponta, inclusive no core) e a premissa de distribuição
exclusivamente por repositório. A ADR-0001 continua valendo para `relay-core`,
`relay-host` e `relay-ui`, e a decisão 5 dela é preservada integralmente.

## Contexto

A ADR-0007 definiu a primeira entrega como um observador read-only: o harness
numa tela, a interface noutra, o estado durável mudando sem recarregar. Na
prática, só essa parte da UI web funcionou; a execução integrada (launcher,
preflight, PTY, terminal) falhou e permanece atrás de `--exec`.

O uso que importa é o split de terminal: harness de um lado, painel do outro,
sem navegador, porta nem token. O requisito que define a tecnologia é a
distribuição: **um único download**, sem clonar o repositório, sem Node e sem
gerenciador de pacotes, com macOS, Linux e Windows como alvos e leveza como
piso. Os requisitos de aparência são "apresentável, mesmo no terminal" e
mapeamento de status coerente com o design system.

Dois pontos da ADR-0001 colidem com isso. A decisão 2 recusou Rust porque
custaria uma segunda toolchain no `git clone` de todo contribuidor, em troca de
um binário único que a distribuição por repositório dispensava; a própria ADR
registrou que "Rust volta à mesa se essa premissa mudar". E o Contexto da
ADR-0001 nomeia como risco dominante a existência de **duas implementações do
protocolo**, cujo pior caso é duas leituras divergirem em `inconsistent`.

## Decisão

### 1. `relay-tui` é um binário Rust, read-only e passivo

Vive em `app/relay-tui/` (projeto Cargo), com `ratatui`, `crossterm` e `notify`.
Observa um workspace (`--workspace <path>` ou o diretório corrente), nunca
escreve nos cinco registros, não lança processo e só reage às teclas de saída.
A ADR-0004 já aceita toolchain própria dentro de `app/`; quem só quer as skills
continua sem build.

### 2. Rust, e não Go nem TypeScript compilado

| | Binário | Core | Multiplataforma |
| --- | --- | --- | --- |
| Rust + ratatui | ~2–5 MB | reescrito | matriz de CI |
| Go + Bubble Tea | ~5–10 MB | reescrito | cross-compile trivial |
| TS com `bun build --compile` | ~60–100 MB | reaproveitado | alvos de cross-compile |

**Por quê:** os três cumprem "um download". O TS compilado é o único que mantém
uma implementação do protocolo, mas leva o runtime no binário e combina Ink com
Bun, com menos estrada. Entre Rust e Go a diferença técnica é pequena; Rust foi
escolhido pela preferência do dono do projeto e porque a ADR-0001 já apontava o
core como a peça que se reescreve com menos risco, por ser pura e testada.
Reaproveitar o `relay-core` por um processo Node derrotaria o download único.

### 3. Segundo core, contido por uma suite de conformidade

O `core` em Rust reimplementa parse, as 13 verificações de integridade e a
derivação do `RelayState` (ADR-0003), puro: conteúdo entra, estado sai.

A divergência entre dois leitores é o risco que a ADR-0001 chamou de dominante;
aceitá-lo exige um mecanismo, não esperança. Em `app/conformance/<caso>/` cada
caso é um workspace em disco com um `expected.json`. O `relay-core` em TS e o
`core` em Rust executam **a mesma suite**; qualquer divergência quebra o CI. O
core em TS é a referência enquanto esta ADR não for substituída, de modo que
"qual está certo?" tem resposta: o que a suite — e, por trás dela, o
`docs/PROTOCOL.md` — dizem.

A decisão 5 da ADR-0001 fica intacta: o `relay-tui` lê, deriva e desenha; só as
skills mutam.

### 4. Distribuição por binário em Release

Uma tag publica no GitHub Release um binário por alvo. **O requisito é o macOS**
(arm64 e x64), com `cargo test` no macOS no CI. Linux (x64 e arm64, `musl`,
estático) e Windows (x64) são **melhor esforço**: entram no build quando
possível e uma falha neles nunca bloqueia o release nem o CI. *(Emenda de
2026-10-02: a redação original exigia os cinco alvos e o CI nos três sistemas;
o dono do projeto trabalha no macOS e Linux e Windows são desejo, não
impedimento.)*

Isso cria um canal que a ADR-0004 não cobre — a camada "produto" chegava só por
clone — e reintroduz o que a ADR-0001 tinha descontado: um binário baixado
recebe `com.apple.quarantine` no macOS e o aviso do SmartScreen no Windows.

A primeira versão **não assina nem notariza**; documenta o contorno
(`xattr -d com.apple.quarantine` e "Executar assim mesmo"). Assinatura exige uma
conta Apple Developer paga e fica como decisão adiada, junto com auto-update e
pacotes (brew, scoop, nix).

**Exceção à conformidade 3 da ADR-0004.** Um workflow só é lido pelo GitHub em
`.github/workflows/`, fora de `app/`, e precisa nomear `app/relay-tui` para
construir. São exatamente dois arquivos, ferramenta do repositório (a terceira
camada da ADR-0004, nunca superfície de pacote): o de CI só dispara com
mudanças em `app/relay-tui/**` e o de release só com tags `relay-tui-v*`.
Removido `app/`, nenhum dos dois dispara, de modo que a regra que importa —
`rm -rf app/` devolve o repositório a um estado funcional — se mantém.

### 5. O nome é `relay-tui`, e não é um Relay CLI

O `AGENTS.md` proíbe criar um Relay CLI antes de o protocolo ser validado em
repositórios reais. Um CLI do Relay interpretaria comandos do protocolo ou
mutaria registros. O `relay-tui` não interpreta comando algum, não escreve e
tem um único propósito: desenhar o estado derivado. É o mesmo argumento que a
ADR-0007 usou para aceitar `--workspace` no host.

### 6. Aparência e paleta

Layout de cartões, paleta própria do meio terminal e fundo herdado do terminal.
Os valores e o mapeamento de status vivem em `docs/design-system/`, que tem
autoridade sobre como cada estado aparece; esta ADR não os repete. Uma
divergência deliberada: `inconsistent` é vermelho no terminal, enquanto a UI web
o mostra em âmbar, igual a `blocked`.

### 7. Evolução considerada: canal de perguntas por arquivo

A ideia: o harness, ao rodar `relay-spec`, escreve uma pergunta num arquivo local
(opções com checkboxes) e espera; a TUI a exibe, o usuário responde, e a TUI
escreve a resposta noutro arquivo que o harness lê. Harness e TUI nunca falam
entre si; encontram-se em arquivos de escritor único (`ask-<id>.md`, do harness;
`answer-<id>.md`, da TUI), com escrita atômica.

Não faz parte desta entrega. Exigiria:

- **Protocolo primeiro.** A TUI passaria a escrever, e a decisão 5 da ADR-0001
  diz que a aplicação nunca escreve. A saída coerente é um canal **efêmero que
  não é um registro** (sem memória durável, fora do git, apagado após o uso),
  definido no `docs/PROTOCOL.md` antes de qualquer skill. Isso emenda a ADR-0001
  e a spec desta entrega (A-007).
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
superfície que já falhou na UI web) e uma interface de chat sobre os modos
headless dos harnesses (um protocolo por harness, e quebra a neutralidade).
Ficam como possibilidades menores, fora desta entrega: injetar um comando no
painel vizinho por multiplexador (tmux, wezterm) e uma dica de próximo passo
derivada do estado.

## Consequências

### Positivas

- Um download resolve a instalação; nada de Node, repositório ou gerenciador.
- Leveza e partida instantânea sem custo de arquitetura.
- A suite de conformidade passa a testar o protocolo de forma neutra de
  linguagem, o que valeria mesmo sem o Rust: as entradas deixam de morar dentro
  de testes TS.
- Nenhuma superfície de execução volta pela porta dos fundos.

### Negativas e custos assumidos

- O protocolo passa a ter duas implementações; a suite as mantém alinhadas, mas
  cada mudança de gramática custa duas edições.
- Uma segunda toolchain (Cargo) entra em `app/` e uma matriz de CI, com o macOS
  como requisito e Linux e Windows como melhor esforço.
- Binários sem assinatura exigem um passo manual de quem baixa, no macOS e no
  Windows.
- `inconsistent` aparece em cores diferentes na web e no terminal.

## Compliance

1. `app/relay-tui/` não contém chamada de escrita em `.specs/` nem em
   `.orchestration/`, nem criação de processo.
2. `app/conformance/` tem os sete casos de status e um caso por verificação de
   integridade; `relay-core` (TS) e o `core` (Rust) passam todos, e o CI
   falha em divergência.
3. O `core` em Rust não acessa disco; só `workspace` lê arquivos.
4. A tag de release publica os binários de macOS arm64 e x64 e o CI roda
   `cargo test` no macOS: é o requisito. Linux e Windows são melhor esforço e
   uma falha neles não impede o release nem o CI.
5. `.github/workflows/` tem só os dois arquivos do `relay-tui`, com os filtros
   de caminho e de tag da decisão 4.
6. Nenhum arquivo fora de `app/` resolve um caminho para dentro dele, como exige
   a ADR-0004, salvo a exceção dos dois workflows da decisão 4.
7. O canal de perguntas só passa a existir depois de uma emenda do
   `docs/PROTOCOL.md` e do spike do Claude Code.

## Notes

Complementa a ADR-0007 (observador read-only), que decidiu *o que* se entrega, e
a ADR-0003, cujo contrato de estado derivado o `core` em Rust reproduz. A
suite de conformidade é o ponto onde uma futura decisão sobre qual core será a
referência de longo prazo poderia ser tomada; esta ADR não a toma.
