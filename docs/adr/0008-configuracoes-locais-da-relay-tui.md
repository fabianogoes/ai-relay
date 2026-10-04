# ADR-0008 — configurações locais da relay-tui

## Status

**Accepted** — 2026-10-04.

## Contexto

A interface já escolhe inglês ou pt-BR ao iniciar, pela flag, configuração do
workspace ou locale. Para trocar depois era preciso encerrar e iniciar o painel
com outra configuração. A TUI continua somente leitura: a configuração
persistente do workspace é escrita pelo `relay-setup`, não pelo painel.

## Decisão

`c` abre uma terceira tela local, **Configurações**, a partir de Agora e de
qualquer nível de Histórico. Ela oferece `Português (Brasil)` e `English`; as
setas movem a seleção, `Enter` aplica e `Esc` cancela e volta. A visão, o nível e
a seleção anteriores são preservados. `q` e `Ctrl-C` continuam encerrando o
processo.

A língua escolhida passa a ser um override desta execução. Ele prevalece sobre
a flag `--lang` e sobre a resolução inicial do workspace/locale e sobrevive às
releituras do workspace, mas não grava `SETTINGS.md` nem qualquer registro. Ao
iniciar o painel novamente, a língua volta a ser resolvida pelas fontes
persistentes.

Enquanto Configurações está aberta, a captura do mouse fica desligada; ao voltar
a Histórico, ela é reativada. Isso mantém a regra de que o mouse só é capturado
na visão Histórico.

## Consequências

### Positivas

- O idioma pode ser alterado sem reiniciar o painel, mantendo Agora e Histórico
  na língua escolhida.
- O painel continua somente leitura e não mistura uma preferência temporária com
  a configuração persistente do workspace.

### Negativas e custos assumidos

- A escolha não persiste ao fechar o painel; para definir a língua padrão do
  workspace, usa-se `relay-setup`.
- O terceiro destino de navegação acrescenta um estado de tela em `app.rs` e um
  catálogo visual bilíngue.

## Compliance

1. A tela Configurações não lê nem escreve arquivos; `tests/read_only.rs` e o
   teste de aplicação verificam que a escolha não cria `SETTINGS.md`.
2. `c`, setas, `Enter`, `Esc`, `q` e `Ctrl-C` têm comportamento coberto pelos
   testes da aplicação; snapshots cobrem as duas línguas em 58 e 40 colunas.
3. A captura do mouse desliga ao abrir Configurações desde Histórico e volta ao
   retornar; o teste da aplicação verifica os dois estados.
4. O override de idioma continua ativo após uma releitura do workspace e só
   existe na memória do processo.

## Notes

Origem: solicitação do usuário, aprovada em conversa em 2026-10-04.
