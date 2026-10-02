# ADR-0001 — Arquitetura inicial da UI do Relay

## Status

**Accepted** — 2026-09-07. Proposta em 2026-09-06.

As cinco premissas do documento-fonte entram nesta ADR como **restrições
dadas**, não como parte da decisão sob avaliação: a UI vai existir; ajustar a
interface é permitido; ajustar o projeto, inclusive o `PROTOCOL.md`, é
permitido; a distribuição é por repositório, sem loja de nenhum sistema; macOS
primeiro.

## Contexto

O Relay precisa de um ponto de entrada visual. A pergunta que normalmente trava
esse tipo de projeto — "Tauri ou Electron?" — vinha sendo tratada como decisão
de entrada, quando o que de fato restringe o desenho é outra coisa.

O que restringe é a exigência de execução: o app precisa lançar CLIs de agente
**no host**, com o `PATH` do usuário, as credenciais já autenticadas e acesso ao
diretório do projeto. Essa exigência elimina mais opções do que qualquer escolha
de framework, e nenhuma casca a resolve melhor que outra.

A distribuição por clone muda a conta por um motivo mecânico: o atributo
`com.apple.quarantine` do macOS é aplicado por quem *baixa* o arquivo, não por
quem o constrói. Um binário gerado por build local não o recebe e abre sem
passar pelo Gatekeeper. Notarização, certificado no Windows e infraestrutura de
auto-update saem inteiros da conta — e com eles, a maior parte do argumento
histórico a favor de escolher uma casca cedo.

Sobre tudo isso pesa um risco de projeto que domina o desenho. O Relay já tem
**uma** implementação do protocolo: a especificação em Markdown de
`docs/PROTOCOL.md`, interpretada pelas skills. O primeiro protótipo criou uma
segunda, em código — mantinha estado próprio, criava specs e backlog, avançava
TODO e handoff, anexava changelog e derivava `backlog`, `ready`, `in_progress` e
`inconsistent`. Com duas implementações, a pergunta "qual das duas está certa?"
não tem resposta, e o caso que a expõe é o pior possível: o `inconsistent`. Dois
leitores independentes podem divergir, e divergência ali trava o trabalho ou o
deixa passar indevidamente.

Insumo analítico completo: [`../design-system/ui-proposal.md`](../design-system/ui-proposal.md).

## Decisão

### 1. A fronteira é a decisão; a casca não

```text
relay-core   (TypeScript)   parser, derivação de estado, invariantes
                            puro, read-only, testável sem I/O

relay-host   (TypeScript)   watcher, detecção de harness, PTY,
                            servidor local
        ↕ HTTP + WebSocket em 127.0.0.1

relay-ui     (TypeScript)   a interface: HTML, CSS e JS comuns
```

A comunicação host↔UI é **HTTP + WebSocket em `127.0.0.1`**, nunca um IPC
proprietário. Componentes do host, todos independentes de stack:

```text
relay-host
  ├─ RelayCore          parser, derivação, validação de invariantes
  ├─ WorkspaceWatcher   observa .specs/ e .orchestration/
  ├─ HarnessRegistry    um adaptador por CLI de agente
  ├─ PtyRunner          lança e acompanha o processo
  └─ HttpServer         HTTP + WebSocket em loopback
```

**Por quê:** um protocolo de transporte padrão em vez de IPC proprietário é o
que torna a casca uma decisão barata de rever. Do lado do projeto, é o que
permite começar a construir hoje sem gastar a primeira semana empacotando.

### 2. TypeScript ponta a ponta, inclusive no core

> **Revisada pela [ADR-0009](0009-relay-tui-observador-de-terminal-em-rust.md)
> apenas para o `relay-tui`**, que ganha um core em Rust contido por uma suite
> de conformidade e distribuído como binário. `relay-core`, `relay-host` e
> `relay-ui` seguem em TypeScript.

Uma linguagem, um `npm install`, e a UI é TypeScript de qualquer forma.
`node-pty` é a ligação de PTY mais madura disponível e cobre ConPTY quando
Windows entrar.

**Por quê:** Rust no core custaria uma segunda toolchain no `git clone` de todo
contribuidor de um projeto público, em troca de um benefício — binário único —
que a premissa de distribuição por repositório já dispensa. Rust volta à mesa se
essa premissa mudar, e `relay-core` é justamente a peça que se reescreve com
menos risco, por ser pura e ter testes.

### 3. A entrega é servidor local + UI no navegador

O navegador do usuário já está instalado e já é a WebView — só que mantida por
outra pessoa. `xterm.js` roda igual numa aba; a PTY vive no processo Node e
chega por WebSocket, que é exatamente o que o processo main do Electron faria.

| | Servidor local | Tauri | Electron |
| --- | --- | --- | --- |
| `git clone && build` exige | Node | Node + Rust + deps de WebView | Node + ~200 MB de Chromium por clone |
| Empacotar por sistema | não empacota | 3 builds | 3 builds |
| `PATH` e credenciais do host | nativo | nativo | nativo |
| Modo Docker read-only | funciona | não | não |
| Vira aplicação depois? | sim, carregando a mesma UI | — | — |

**Por quê:** a consequência que decide é o ciclo de vida. O host é um processo
separado da janela, então **o servidor sobrevive à janela**: fechar a aba não
encerra a execução, e reabrir reanexa à PTY viva com replay do scrollback que o
host guardou. Em Electron ou Tauri, fechar a janela normalmente encerra tudo, e
essa sobrevivência teria de ser construída à mão.

### 4. A casca de aplicação é decisão adiada e reversível

Não se escolhe Tauri nem Electron agora. Com a fronteira do ponto 1, o Tauri
carrega **essa mesma UI** na WebView quando e se fizer sentido; o trabalho é de
dias, não de reescrita. Uma TUI, se um dia voltar a fazer sentido, consome
`relay-core` direto, sem passar pelo host.

**Por quê:** é a aplicação do último momento responsável. Não há informação hoje
que torne a escolha melhor do que ela será depois de o produto rodar, e adiá-la
não bloqueia nenhum trabalho.

### 5. A aplicação lê, deriva e lança; nunca escreve

A aplicação **nunca** escreve nos cinco registros do protocolo. Toda mutação
passa por uma skill executada em um harness. `relay-core` interpreta e valida;
só as skills mutam.

**Corolário:** se a app precisar de um estado que não consegue derivar, a
correção é no `docs/PROTOCOL.md` — não num write da interface.

**Por quê:** é o que impede a segunda implementação do protocolo descrita no
Contexto. Não é cautela de MVP; é a única regra que mantém uma resposta única
para "qual leitor está certo?" no caso `inconsistent`.

## Consequências

### Positivas

- `git clone && npm install && npm start` roda sem empacotamento nenhum; zero
  builds por sistema operacional.
- Modo Docker read-only viável — `docker run --rm -p 7373:7373 -v "$PWD:/ws:ro"
  relay/ui` —, impossível em Tauri ou Electron. Para um projeto público é
  excelente como demonstração e como "olhar o estado de um repositório sem
  instalar nada".
- `--no-exec` deixa de ser um modo capado inventado para o Docker e passa a ser
  o mesmo produto com a superfície de execução desligada, disponível fora dele.
- A sobrevivência do processo à janela sai de graça, por consequência do
  formato.
- Windows e Linux, quando entrarem, são trabalho localizado no `HarnessAdapter`
  e no `PtyRunner` — sem tocar `relay-core` nem a UI.
- Fechar o terminal não perde estado, porque o estado nunca esteve no terminal;
  e terminal externo passa a ser viável pelo mesmo motivo.
- Uma eventual TUI consome `relay-core` direto, sem passar pelo host.

### Negativas e custos assumidos

- **Um servidor em loopback é alcançável por qualquer página que o usuário
  abrir.** Sem isso resolvido, um site qualquer dispara execução de agente na
  máquina dele. Exige, desde a primeira linha: bind apenas em `127.0.0.1` em
  porta efêmera aleatória; token aleatório por execução, exigido em toda
  requisição e no handshake do WebSocket; rejeição de requisição sem
  `Sec-Fetch-Site: same-origin`; nenhum CORS permissivo.
- Conflito de teclado no terminal embutido: `Cmd+W` fecha a aba antes de chegar
  ao `xterm.js`. Não tem solução completa no navegador. Mitiga-se com aviso na
  primeira execução e com o modo externo, e desaparece se a casca vier.
- Duas superfícies de terminal para manter — embutido e externo — em vez de uma.
- O `PtyRunner` precisa cobrir propagação de resize (SIGWINCH), alt-screen,
  mouse tracking, bracketed paste e um buffer de scrollback por run.
- Ficar em TypeScript adia o binário único: todo usuário precisa de Node.
- A detecção de harness vira componente de primeira classe, não utilitário —
  `PATH` difere entre shell interativo e não-interativo, há shims de npm, brew e
  asdf, a flag de versão varia por CLI, e "instalado mas não autenticado"
  precisa ser um estado distinto de "não instalado".

### Consequência descartada explicitamente

**Lançar harness de dentro de um container não funciona.** Os CLIs não estão
instalados lá, não estão autenticados lá, e o `PATH` é outro. Bind-mount do
projeto resolve os arquivos e não resolve nada do resto. Fica registrado para
que ninguém gaste tempo tentando.

## Conformidade

Critérios objetivos, verificáveis, no espírito de fitness function:

1. **Pureza do core.** `relay-core` não importa `fs`, `net`, `child_process`
   nem qualquer módulo de `relay-host`. Verificável por regra de import — o
   núcleo é puro por construção, não por convenção.
2. **A app não escreve.** Nenhum código em `relay-host` ou `relay-ui` escreve em
   `.specs/` ou `.orchestration/`. Toda escrita nos cinco registros vem de uma
   skill.
3. **Superfície de rede.** Toda rota do `HttpServer` exige token e origem
   same-origin; a rota de lançamento não existe quando `--no-exec` está ativo.
4. **Execução sem shell.** Processos são lançados com `argv[]`, nunca por
   `sh -c`, `bash -c` ou equivalente. Um prompt de agente contém aspas, crase,
   ponto e vírgula e `$`, e concatenar em string de shell é injeção garantida.
   O mesmo vale no modo externo: o host escreve um script wrapper e pede ao
   emulador que o abra — o script é o contrato, não a linha de comando.
5. **Preflight honesto.** O preflight mostra o **argv elemento por elemento**,
   nunca uma string com aspas montada para parecer um comando de shell. Uma
   string exibida é uma *renderização* do argv, e as duas divergem quando o
   prompt contém aspas: o usuário aprova uma coisa e o app executa outra.
   Preflight que mente é pior que preflight nenhum, porque é exatamente ali que
   a confiança é depositada.
6. **Preferência não é autorização.** A escolha de qual harness usar pode ser
   local ao workspace; ela não é consentimento para executar. O preflight
   continua obrigatório em toda execução.
7. **Uma única fonte de verdade.** O estado é derivado apenas dos arquivos em
   disco. O stdout do harness **nunca** é parseado — parseá-lo seria uma
   terceira implementação do protocolo, frágil e específica por harness.

## Notas

**Origem.** Extraída de [`../design-system/ui-proposal.md`](../design-system/ui-proposal.md),
que permanece no repositório como insumo analítico, junto dos dois protótipos.
Esta ADR registra apenas o que foi decidido; a proposta registra também o que
foi observado e descartado.

**Escopo.** As recomendações de UI/UX da proposta — o handoff como objeto
central da tela, as três telas em vez de seis status, densidade e contraste, o
que remover do protótipo — são decisões de design de interface, não de
arquitetura, e não entram aqui.

**Decisão deliberadamente não tomada**, para que não seja confundida com
omissão: a casca de aplicação (ponto 4).

**Correção — semântica de ordem.** A redação original desta ADR listava a
prioridade do backlog e a ordem do `TODO.md` como dívida de contrato em aberto,
e atribuía a `relay-continue` a expressão "maior prioridade". A atribuição
estava errada já na redação: nenhuma skill usa essa expressão, e o protocolo já
havia decidido as duas semânticas em 2026-09-06 (`562a439`), antes desta ADR
existir. A observação veio de
[`../design-system/ui-proposal.md`](../design-system/ui-proposal.md), escrita
contra uma leitura anterior das skills, e foi copiada para cá sem ser
confrontada com o protocolo.

A decisão do protocolo é negativa e definitiva: a ordem textual não codifica
prioridade, fila, dependência, sequência de execução, esforço nem percentual de
progresso (`docs/PROTOCOL.md:90-94` para o backlog, `:114-117` para o TODO). Ela
é apenas a recomendação padrão determinística, e o usuário pode selecionar
qualquer item pendente.

A consequência para a interface não muda: nada de fila numerada, rótulo `FIFO`,
posição, barra de progresso percentual ou pista com dependências. O que muda é o
motivo, e a diferença importa para quem desenhar a tela — não é assunto pendente
à espera de decisão, é assunto decidido no negativo, e a interface nunca deve
afirmá-lo.

**Já resolvido.** A proveniência do handoff — `Harness` e `Updated` em RFC 3339
com fuso explícito — foi acrescentada ao protocolo em 2026-09-06
(`docs/PROTOCOL.md:125-129`) e é validada nas integridades
(`docs/PROTOCOL.md:207-210`). Sem ela, a proveniência existiria apenas dentro da
interface e desapareceria para quem der `cat` no arquivo.
