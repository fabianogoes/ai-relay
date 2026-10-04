# 20261003-002 - Relógio do sistema, documentação enxuta, CI do pacote, idioma e sinais

## Problem

Os itens 5 a 10 da análise de 2026-10-03.

**Horário inventado.** O `relay-session` pede "the current RFC 3339 timestamp"
e o `relay-spec` pede "today's date", mas o modelo não sabe a hora nem o dia. Um
`Updated` inventado passa na verificação de formato e mente sobre quando o
handoff foi escrito.

**Documentação duplicada que já diverge.** `docs/TUI.md` e
`app/relay-tui/README.md` repetem instalação, teclas, telas e limites, contra a
regra de não copiar entre camadas. Já discordam em três pontos: os sinais que
não restauram o terminal, a pasta de instalação (`~/.local/bin` ou
`/usr/local/bin`) e a descrição da visão Agora, que no README não tem os cartões
de spec nem a linha de próximo passo.

**Restos.** `docs/INSTALL.md` promete um comando de marketplace que já está
logo acima. `docs/superpowers/plans/` guarda um plano de outra ferramenta que
diz "Do not create a custom UI". As quatro imagens de `docs/images/` não são
referenciadas desde o commit `ca7e146` e estão cortadas e com erros. Fixtures e
snapshots ainda falam de `relay-ui` e Vue 3. O README dos fixtures aponta um
caminho errado. Há comentários que citam o leitor TypeScript e o Node, e o
`.gitignore` ainda tem entradas de Python e `node_modules`.

**CI incompleto.** O CI não roda `cargo fmt` nem `clippy`. O `open-split.sh` é o
único executável distribuído, e o teste dele (145 casos) não roda no CI. O
`tests/history.rs` lê os registros vivos deste repositório, mas o CI não dispara
quando eles mudam: a limpeza de 2026-10-03 teve de editar o teste à mão, e um
registro inconsistente pode entrar sem que nada acuse.

**Tela só em português.** O pacote se apresenta em inglês, mas a tela do
`relay-tui` é só em pt-BR. O texto das violações está em português dentro do
core, contra a ADR-0003 ("tom e tela ficam fora do estado"), e os fixtures o
comparam letra a letra.

**Nenhum idioma padrão por workspace.** O Relay vai ter dois idiomas, inglês e
pt-BR, mas nada no repositório diz em qual deles um workspace trabalha. As skills
seguem o idioma da conversa, e o painel só conheceria o locale da máquina. Quem
escreve os registros em português num computador com locale `en_US`, um caso
comum, veria a tela em inglês.

**Sinais.** Um `SIGTERM`, um `SIGINT` vindo de fora ou o fechamento da janela
deixam o terminal em modo raw, na tela alternativa e com o mouse capturado.

## Scope

- **Relógio do sistema.** O protocolo diz que datas e horários vêm do relógio
  do sistema na hora da escrita, e as skills que os escrevem dão o comando.
- **Uma fonte de documentação.** `docs/TUI.md` (e `docs/TUI.pt-BR.md`) é a única
  fonte de uso do `relay-tui`; o README do crate fica com o que é de
  desenvolvimento.
- **Remoção dos restos** listados no problema.
- **Fixtures e snapshots** com conteúdo neutro.
- **CI:** `fmt` e `clippy` no job obrigatório do `relay-tui`, um workflow do
  pacote para `.agents/tests/` e `shellcheck`, uma verificação dos registros
  deste repositório e uma ADR para o CI do pacote.
- **Idioma padrão do workspace**, inglês ou pt-BR, escolhido num passo do
  `relay-setup` e seguido pelas skills e pelo painel.
- **Tela em inglês ou pt-BR**, pelo idioma do workspace e pelo locale, com o
  core entregando a violação sem texto.
- **Terminal restaurado** em `SIGTERM`, `SIGINT` e `SIGHUP`.

## Non-goals

- Traduzir o contrato. Títulos de seção (`## Problem`, `## Acceptance
  criteria`), chaves (`- Status:`), status, marcadores e IDs continuam em
  inglês em qualquer idioma, porque o leitor os interpreta.
- Idiomas além de inglês e pt-BR. O conteúdo dos registros aparece como foi
  escrito: só rótulos e mensagens da tela são traduzidos. Os status nos
  registros continuam em inglês.
- Novos diagramas do protocolo. As imagens saem e não são redesenhadas agora,
  porque a spec 20261003-001 muda o ciclo de vida e um diagrama feito hoje
  nasceria velho.
- Assinatura e notarização, auto-update, pacotes (brew, nix) e o Windows.
- Qualquer mudança no protocolo além da origem de datas e horários.
- Um CLI de verificação: o `AGENTS.md` proíbe um Relay CLI.

## Decisions

**Datas do relógio do sistema, com o comando escrito na skill.** O modelo não
sabe a hora, e um horário inventado passa na verificação de formato. `Updated`
usa `date -u +%Y-%m-%dT%H:%M:%SZ`, em UTC, porque o `%z` do `date` gera `-0300`,
sem os dois-pontos que o RFC 3339 exige. A data do registro de changelog usa
`date +%Y-%m-%d` e a do nome da spec usa `date +%Y%m%d`. Alternativa rejeitada:
deixar o modelo inferir.

**`docs/TUI.md` é a autoridade de uso.** O README do crate fica com compilar,
testar, publicar e estrutura, e aponta o guia. As divergências se resolvem pelo
comportamento real. Alternativa rejeitada: manter os dois sincronizados à mão,
que é o que já falhou três vezes.

**Remover os restos em vez de reaproveitá-los.** O plano do superpowers é uma
segunda convenção ao lado do protocolo, e as imagens descrevem um ciclo que a
spec 20261003-001 vai mudar. Dados de teste não descrevem um produto removido:
os fixtures e snapshots passam a ter conteúdo neutro, e os testes continuam
provando a mesma coisa.

**`fmt` e `clippy` no job obrigatório.** Um aviso do `clippy` reprova o CI
(`-D warnings`), e o código é corrigido antes de a regra entrar.

**O pacote ganha um workflow próprio.** `skills/` e `.agents/` não são `app/`,
e pela ADR-0002 `app/` tem de poder ser removido. Os testes de `.agents/tests/`
rodam no macOS, que é o requisito e onde o `osascript` existe; o `shellcheck`
roda no Ubuntu, onde já vem instalado. O workflow dispara com mudanças em
`skills/**`, `.agents/tests/**` e `docs/PROTOCOL.md`. A ADR-0003 deixa de
afirmar que só existem os dois workflows do `relay-tui` e passa a limitar apenas
os dela. Alternativa rejeitada: rodar os testes do pacote no workflow do
`relay-tui`, que só dispara com mudança em `app/` e acoplaria o pacote à
interface.

**Os registros deste repositório viram verificação do CI, e o histórico é
congelado.** Um teste no crate deriva o estado dos registros reais e falha se
houver violação, porque o core é o único leitor. O CI do `relay-tui` também
dispara com `.orchestration/**` e `.specs/**`. O `tests/history.rs` passa a ler
uma cópia congelada dos registros. Assim, "o leitor funciona" e "os registros
estão coerentes" viram dois testes, e editar os registros não exige editar teste.
Alternativa rejeitada: um comando de verificação, que seria um Relay CLI.

**Idioma padrão do workspace, escolhido no setup** (pedido do usuário). O
protocolo define `.orchestration/SETTINGS.md`, com `- Language: en` ou
`- Language: pt-BR`. É configuração, não registro: só o `relay-setup` o escreve,
e ele fica fora das verificações de integridade (um valor desconhecido é
ignorado, e vale o próximo da precedência). Quando o arquivo falta, o
`relay-setup` pergunta o idioma pela interação nativa do harness, recomendando o
idioma da conversa; com o idioma definido, uma nova execução não pergunta de
novo, e trocar de idioma é um pedido explícito ao `relay-setup`. As skills
conduzem a entrevista e escrevem o texto livre de specs e registros nesse
idioma; sem o arquivo, seguem o idioma da conversa, como hoje. Alternativas
rejeitadas: uma linha na seção Relay do `AGENTS.md` (arquivo do usuário, que o
painel teria de interpretar) e só o locale da máquina (não é do repositório e
não chega às skills).

**Tela em inglês ou pt-BR** (escolha do usuário). A precedência é `--lang`,
depois o idioma do workspace, depois o locale e por fim o inglês. O locale segue
o POSIX: `LC_ALL`, depois `LC_MESSAGES`, depois `LANG`, e um valor que começa com
`pt` dá pt-BR. O workspace vem antes do locale porque o idioma em que o trabalho é
registrado é decisão do repositório, não da máquina. `--lang en|pt-BR` torna os
testes determinísticos. A ajuda e as mensagens de erro seguem o mesmo
idioma. O catálogo é um módulo do crate, sem dependência: são poucas dezenas de
textos, e uma biblioteca de tradução não se paga.

**O core entrega a violação sem texto.** `Violation` passa a ter `check`,
parâmetros e registros; a view formata a mensagem no idioma da tela. Era o que a
ADR-0003 já dizia, e é o que desamarra os fixtures do idioma. A ADR-0003 e os
`expected.json` mudam juntos.

**Snapshots nos dois idiomas, sem dobrar tudo.** Os existentes continuam em
pt-BR. O inglês ganha um por status e um por nível do Histórico, em 58 e 40
colunas, onde o comprimento diferente dos textos muda o corte. As variações de
altura ficam só em pt-BR, porque provam o layout, não o idioma.

**Sinais com `signal-hook`.** `SIGTERM`, `SIGINT` e `SIGHUP` restauram o
terminal (modo raw, tela alternativa e captura do mouse) e encerram com 128 mais
o número do sinal, a convenção do shell. A dependência entra na ADR-0003 na
mesma mudança. Alternativa rejeitada: `libc` direto, com `unsafe` próprio para
um problema já resolvido por uma biblioteca.

## Acceptance criteria

- A-001 - O `docs/PROTOCOL.md` diz que `Updated`, a data do registro de
  changelog e a data do nome da spec vêm do relógio do sistema na hora da
  escrita, e as skills que escrevem esses campos dão o comando de cada um
- A-002 - `docs/TUI.md` é o único lugar com instalação, teclas, telas e limites
  do `relay-tui`; o README do crate fica com compilar, testar, publicar e
  estrutura, e aponta o guia
- A-003 - A frase desatualizada do `docs/INSTALL.md` (inglês e pt-BR), o plano
  em `docs/superpowers/`, as imagens de `docs/images/`, as linhas de Python e
  `node_modules` do `.gitignore`, os comentários que citam o leitor TypeScript
  e o Node e o caminho errado no README dos fixtures saem do repositório
- A-004 - Nenhum fixture ou snapshot menciona a interface web (`relay-ui`,
  Vue 3, `ui-primeiro-marco-visual`), e os testes passam
- A-005 - O job obrigatório do CI do `relay-tui` roda `cargo fmt --check` e
  `cargo clippy --all-targets -- -D warnings`, e o código passa nos dois
- A-006 - Um workflow do pacote roda os testes de `.agents/tests/` no macOS e o
  `shellcheck` nos scripts de `skills/` e `.agents/tests/`, dispara com
  mudanças nesses caminhos e no `docs/PROTOCOL.md`, e passa
- A-007 - Um teste falha quando os registros deste repositório derivam
  `inconsistent`, e o CI do `relay-tui` também dispara com mudanças em
  `.orchestration/` e `.specs/`
- A-008 - O `tests/history.rs` lê uma cópia congelada dos registros: editar os
  registros deste repositório não muda o resultado dele
- A-009 - Uma ADR registra o CI do pacote, e a conformidade 5 da ADR-0003 passa
  a limitar só os workflows do `relay-tui`
- A-010 - O idioma da tela segue, nesta ordem, `--lang`, o idioma do
  workspace, o locale (`LC_ALL`, `LC_MESSAGES`, `LANG`) e o inglês; a ajuda e as
  mensagens de erro seguem o mesmo idioma
- A-011 - O core entrega a violação com `check`, parâmetros e registros, sem
  texto; a view formata a mensagem no idioma da tela; a ADR-0003 e os fixtures
  refletem o contrato novo
- A-012 - O `DESIGN.md` traz os rótulos nos dois idiomas antes do código, e há
  snapshots em inglês de cada status e de cada nível do Histórico em 58 e 40
  colunas
- A-013 - `SIGTERM`, `SIGINT` e `SIGHUP` vindos de fora restauram o terminal e
  encerram com 128 mais o número do sinal, como prova o teste em pty; o limite
  sai do `docs/TUI.md`, e a dependência nova entra na ADR-0003
- A-014 - `docs/TUI.md` e `docs/TUI.pt-BR.md` explicam como o idioma da tela é
  escolhido e deixam de dizer que ela é só em português
- A-015 - O `docs/PROTOCOL.md` define `.orchestration/SETTINGS.md` com
  `Language` (`en` ou `pt-BR`), escrito só pelo `relay-setup` e fora das
  verificações de integridade
- A-016 - O `relay-setup` pergunta o idioma padrão quando o workspace não tem um,
  recomendando o idioma da conversa, e uma segunda execução não pergunta de novo
- A-017 - As skills conduzem a entrevista e escrevem o texto livre de specs e
  registros no idioma do workspace, enquanto títulos de seção, chaves, status,
  marcadores e IDs continuam em inglês
- A-018 - Este repositório declara `pt-BR` como idioma padrão

## Backlog candidates

- B-059: Protocolo e skills tiram datas e horários do relógio do sistema
- B-060: `docs/TUI.md` como única fonte de uso, e os restos removidos
- B-061: Fixtures e snapshots sem a interface web
- B-062: `fmt` e `clippy` no CI do relay-tui, workflow do pacote e ADR
- B-063: Registros deste repositório verificados no CI, e histórico congelado
- B-064: Core entrega violações sem texto, com parâmetros
- B-065: Tela em inglês e pt-BR pelo workspace e pelo locale, com
  `DESIGN.md`, snapshots e guia (needs: B-061, B-064, B-067)
- B-066: Terminal restaurado em `SIGTERM`, `SIGINT` e `SIGHUP`
- B-067: Idioma padrão do workspace no protocolo, no passo do `relay-setup` e
  nas skills
