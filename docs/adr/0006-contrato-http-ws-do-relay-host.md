# ADR-0006 — Contrato HTTP/WS do relay-host

## Status

**Accepted** — 2026-09-07.

## Contexto

O `relay-core` deriva estado; a `relay-ui` desenha. Entre os dois está o
`relay-host`, e a forma como o estado chega à interface — mensagem de
WebSocket, rota HTTP, formato de reconexão e replay — foi adiada de propósito
na ADR-0003 (Notas: "aqui está apenas o que trafega, não como trafega").
A spec 005 é a primeira a precisar dessa forma: servidor local, watcher,
detecção de harness, e as rotas de conteúdo bruto que a segunda visão (spec
007) consome.

A ADR-0001 já fixou as restrições que decidem o desenho. Conformidade 3:
*"Toda rota do HttpServer exige token e origem same-origin; a rota de
lançamento não existe quando `--no-exec` está ativo"*. Contexto: *"Um
servidor em loopback é alcançável por qualquer página que o usuário abrir"* —
sem token e origem, um site qualquer dispara execução de agente na máquina.
Ponto 5: a aplicação nunca escreve nos cinco registros.

A entrega é **servidor local + UI no navegador** (ADR-0001 ponto 3): o
navegador já é a WebView, a PTY vive no processo Node e chega por WebSocket,
e fechar a aba não encerra a execução.

## Decisão

### 1. Servidor HTTP + WebSocket em `127.0.0.1`, porta efêmera aleatória

Um único processo Node serve HTTP e WebSocket (`http.Server` + `ws`
acoplado ao `upgrade`). O bind é **somente** `127.0.0.1`, nunca `0.0.0.0`; a
porta é efêmera (0 → a que o SO atribuir) e é reportada ao chamador para
abrir o navegador.

**Por quê:** loopback restringe o alcance à máquina local; porta aleatória
evita colisão entre instâncias e é a leitura da ADR-0001 ("bind apenas em
`127.0.0.1` em porta efêmera aleatória").

### 2. Token aleatório por execução, entregue no HTML inicial

O host gera um token criptográfico (`crypto.randomBytes`) a cada execução.
Ele viaja no **corpo do HTML inicial** — um `<meta name="relay-token">` — e
nunca em URL nem em query string, para não vazar em histórico do navegador ou
em log de acesso.

- Requisições HTTP de API enviam `X-Relay-Token: <token>` no cabeçalho.
- O handshake do WebSocket não permite cabeçalho customizado do navegador;
  o token vai no **subprotocol** `Sec-WebSocket-Protocol: relay.<token>`.

**Por quê:** token por execução significa que a superfície expira quando o
host sai; nada persistente no disco, nada reutilizável. O subprotocol mantém
o token fora da URL mesmo no handshake WS, honrando a decisão da spec 005.

### 3. `GET /` é o bootstrap; toda rota de API exige token e same-origin

`GET /` entrega o HTML inicial (com o token e o `environment`) e é a única
rota que não exige token — é ela que o entrega. Não há CORS permissivo:
uma página de outro site não consegue ler o corpo (browser bloqueia a
leitura cross-origin), então o token não vaza por aí.

Toda rota `/api/*` e o WebSocket exigem:
- **token** válido (`X-Relay-Token` no HTTP, subprotocol no WS); ausência ou
  valor errado → `403`;
- **origem same-origin**: `Sec-Fetch-Site: same-origin` no HTTP (ausente ou
  diferente → `403`); no WS, o `Origin` do handshake precisa bater com
  `scheme://host:porta` do servidor.

Requisição que passa na autenticação mas não casa com nenhuma rota registrada
→ `404` (a rota não existe), nunca `403`.

**Por quê:** a distinção `403` (autenticado errado) vs `404` (rota inexistente)
é o que torna verificável a Conformidade 3 da ADR-0001 — a rota de lançamento
não existe sob `--no-exec`, então pedir por ela devolve `404`, não `403`.

### 4. Rotas de conteúdo bruto para a segunda visão

Somente-leitura, servindo texto cru — nunca um parser novo do protocolo na
UI:

| Rota | Resposta |
| --- | --- |
| `GET /api/state` | `UiPayload` atual: `RelayState` derivado por `relay-core` + `Environment` |
| `GET /api/specs` | Lista de specs: id (nome do arquivo), título e contagem de tarefas no backlog |
| `GET /api/specs/<id>` | Conteúdo bruto da spec (texto) |
| `GET /api/changelog` | Conteúdo bruto de `CHANGELOG.md` (texto) |
| `GET /api/harnesses` | Detecção de harness: nome, versão, estado (instalado / não autenticado / ausente) |

A coluna do meio da segunda visão (backlog da spec selecionada) **não** tem
rota própria: reusa `RelayState.backlog`, filtrado no cliente pelo campo
`spec` de cada entrada (decisão da spec 007). A contagem por spec da coluna
Specs também sai do estado derivado.

**Por quê:** a UI lê texto e estado; quem interpreta o protocolo é só o
`relay-core`. O host importa `deriveState` e nunca reimplementa a gramática.

### 5. A rota de lançamento é reservada e só existe fora de `--no-exec`

Esta ADR registra a **superfície**, não a implementação: `POST /api/launch`
é a rota de lançamento que a spec 008 implementa. Ela só pode existir quando
`--no-exec` não está ativo; sob `--no-exec`, a rota nem é registrada e a
requisição devolve `404`. Nada nesta ADR autoriza uma rota de lançamento a
escrever registros — execução é só processo, mutação é sempre de skill
(ADR-0001 ponto 5).

### 6. `Environment` sai do host, nunca do core

O `relay-core` é puro e não alcança disco (ADR-0003 decisão 1); `workspace`
(caminho do diretório corrente) e `execEnabled` (`false` sob `--no-exec`)
são fornecidos pelo host e embutidos no `UiPayload`. A lista de harnesses
fica fora do `UiPayload`, num endpoint próprio — o `Environment` da ADR-0003
continua mínimo.

### 7. Watcher de diretório inteiro, não de arquivo

O host observa os diretórios `.orchestration/` e `.specs/` inteiros
(`fs.watch`), não arquivo por arquivo. Qualquer mudança relevante dispara
releitura, re-derivação por `relay-core` e um novo `UiPayload` empurrado a
todo cliente WebSocket conectado — sem recarregar a página.

**Por quê:** os registros são pequenos; o custo de re-derivar tudo a cada
mudança é baixo, e observar diretórios inteiros é mais simples e não perde
spec nova sendo criada (decisão da spec 005).

### 8. Farol de descoberta por workspace para reconexão entre processos

Matar e reiniciar o host muda a porta efêmera (decisão 1); sem mais nada, a
aba já aberta fica presa tentando o endereço morto, e nenhuma navegação
same-origin pode alcançar a origem nova sem que algo aponte para ela — a
reserva desta ADR ("formato de reconexão... é assunto das specs 007/009")
cobria reconectar dentro do mesmo processo, nunca descobrir um processo novo
(spec 012, A-010).

O host abre um segundo listener em `127.0.0.1`, somente leitura, numa porta
**derivada deterministicamente do caminho absoluto do workspace** (hash
estável, não aleatória): mesma workspace, mesma porta de farol, em toda
execução. Ele expõe uma única rota que devolve `{ "port": <porta efêmera
atual> }` — nada além de um inteiro não secreto. Não exige token: não há
segredo para proteger, e exigi-lo seria inútil, já que o próprio propósito é
ser alcançado por uma aba cujo token antigo morreu com o processo anterior.
Aceita `Cross-Origin`: sem isso a aba (presa na origem morta) não conseguiria
ler a resposta vinda de uma porta diferente; o servidor ecoa
`Access-Control-Allow-Origin` apenas quando o `Origin` da requisição casa com
`http://127.0.0.1:<qualquer porta>`, nunca `*`.

A porta do farol viaja para o cliente do mesmo jeito que o token: um `<meta
name="relay-discovery-port">` no HTML inicial. O cliente nunca recalcula o
hash; só lembra o valor que já recebeu. Ao detectar que a origem atual não
responde mais (WebSocket fechado e requisições HTTP falhando de forma
consistente, não uma falha isolada), a aba consulta o farol com backoff até
receber uma porta e navega para `http://127.0.0.1:<porta>/`, o que dispara o
bootstrap normal de `GET /` — token novo, HTML novo, nenhuma mudança nas
decisões 2 e 3.

**Por quê:** preserva as duas garantias que a spec 012 exige intactas — a
porta principal continua efêmera e aleatória, o token continua por execução e
fora de URL/query. O farol não carrega nem token nem conteúdo do protocolo;
o único fato que expõe (qual porta efêmera atende hoje este workspace) exige
já conhecer o workspace, o que só é verdade pra quem já tinha a aba aberta
para ele. Derivar a porta do workspace, em vez de um farol único e global,
evita concentrar num só ponto a descoberta de todos os workspaces ativos da
máquina.

**Consequência assumida:** colisão de hash entre dois workspaces é
astronomicamente rara mas não impossível; quando a porta derivada já está em
uso por outro processo, o host falha ao iniciar com um erro de diagnóstico
claro (nunca escolhe silenciosamente outra porta — isso quebraria a
propriedade "mesma workspace, mesma porta de farol" da qual o mecanismo
inteiro depende).

## Consequências

### Positivas

- A superfície de rede é verificável por integração: bind em loopback, token,
  same-origin, `404` da rota reservada — cada um um teste.
- A UI fica barata de trocar: serviu o `UiPayload` por HTTP/WS padrão, sem
  IPC proprietário (ADR-0001 ponto 1).
- O host nunca interpreta o protocolo: toda derivação vem de `relay-core`, e
  a única fonte de verdade continua sendo os arquivos em disco (ADR-0001
  Conformidade 7).
- `--no-exec` é o mesmo produto com a superfície de execução desligada, não
  um modo capado (ADR-0001 Consequências).

### Negativas e custos assumidos

- Um servidor em loopback continua alcançável por páginas locais; o token e o
  same-origin mitigam, mas a superfície exige cuidado contínuo.
- A detecção de harness é heurística (versão via `--version`, estado de
  autenticação por presença de artefatos de config) e vai querer refinamento
  à medida que os três CLIs mudarem.
- O WebSocket carrega o token em subprotocol, que é menos comum que query
  string — alguns clientes não-Web exigirão adaptação.
- Watcher de diretório re-deriva tudo a cada toque; aceitável pelo tamanho
  dos registros, documentado como escolha.
- O farol de descoberta (decisão 8) é uma segunda superfície de rede sempre
  ativa, alcançável por qualquer página local que já conheça (ou adivinhe)
  a porta derivada de um workspace; ela não expõe token nem conteúdo, só um
  número de porta, mas é uma superfície nova assumida conscientemente.

### Consequência descartada explicitamente

Token em query string do WebSocket ("mais simples") — vazaria em log de
acesso e contraria a decisão da spec 005. Subprotocol fica com o custo.

## Conformidade

1. O servidor liga só em `127.0.0.1`, nunca em `0.0.0.0`.
2. Toda rota `/api/*` e o handshake WS rejeitam requisição sem token ou de
   origem diferente (`403`); `GET /` é o único bootstrap sem token.
3. Sob `--no-exec`, a rota `POST /api/launch` não existe: requisição
   autenticada devolve `404`, não `403`.
4. O host importa `deriveState` de `relay-core` e não reimplementa a
   gramática do protocolo; nenhum código do host escreve em `.specs/` ou
   `.orchestration/`.
5. Mudança em `.orchestration/` ou `.specs/` dispara novo `UiPayload` pela
   WebSocket sem recarregar a página.
6. O token nunca aparece em URL nem em query string; só no HTML inicial, no
   cabeçalho `X-Relay-Token` e no subprotocol do WS.
7. O pacote vive em `app/relay-host/`, seguindo a estrutura da ADR-0004.
8. O farol de descoberta liga só em `127.0.0.1`, numa porta derivada
   deterministicamente do workspace; devolve somente a porta efêmera atual,
   nunca token nem conteúdo do protocolo; falha ao iniciar (não escolhe
   outra porta em silêncio) se a porta derivada já estiver em uso.

## Notas

**Relação com outras ADRs.** A fronteira relay-core/host/ui é a ADR-0001 ponto
1; o formato do que trafega é a ADR-0003; a estrutura de `app/` é a ADR-0004.
Esta ADR registra apenas o transporte.

**Reserva, não implementação.** `POST /api/launch`, PTY e terminal ficam para
as specs 008 e 009; aqui estão registrados como superfície futura para que a
implementação de 008 só acrescente, nunca revise o contrato.

**O que esta ADR não decide.** A forma de servir o HTML da `relay-ui` (dev
server vs arquivo construído) e o formato de reconexão/replay da WebSocket
**dentro do mesmo processo** são assuntos das specs 007/009. Descoberta
**entre processos** — quando o host inteiro reinicia numa porta nova — é a
decisão 8, acrescentada pela spec 012.

**Revisão 2026-09-16 (spec 012).** Decisão 8 é aditiva: nenhuma decisão
anterior foi revogada, a porta principal e o token continuam exatamente como
descritos nas decisões 1 e 2. O status permanece Accepted.