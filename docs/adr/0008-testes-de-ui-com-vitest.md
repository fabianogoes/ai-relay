# ADR-0008 — Testes de UI com Vitest + jsdom

## Status

**Accepted** — 2026-09-16.

## Contexto

A ADR-0005 escolheu Vue 3 para a `relay-ui` e explicitamente deixou a
estratégia de testes de componente em aberto ("decidir isso agora seria
decidir sem informação"). A spec `20260907-013-ui-acessibilidade-e-conformidade.md`
fecha essa lacuna com A-013: uma suíte que reproduza seis regressões reais
encontradas na revisão visual —

1. preview do preflight desatualizado (resposta assíncrona fora de ordem
   substituindo o plano vigente);
2. foco escapando da contenção de um diálogo (Tab saindo do modal);
3. downgrade de consentimento não limpando o valor persistido;
4. harness ausente podendo ser confirmado para execução;
5. queda de conexão não marcando dado como desatualizado;
6. seleção de ID errada ao escolher entre múltiplas tarefas disponíveis.

A mesma spec já condiciona a escolha: "Se isso introduzir Vitest, Playwright,
Testing Library ou equivalente, a escolha e seus custos são registrados em
ADR antes da dependência." Esta ADR é esse registro.

O que a arquitetura já restringe, e que pesa na escolha:

- **Todos os testes hoje são `node --test` puro**, sem framework, em
  `relay-core`, `relay-host` e `relay-ui` (funções e composables). Não existe
  precedente de montagem de componente nem de DOM simulado no repositório.
- **A view não contém lógica** (ADR-0005, decisão 3): o que há para testar em
  componente é majoritariamente interação — clique, foco, evento de input,
  resposta assíncrona — não cálculo de estado.
- **`app/` não pode ganhar passo de build obrigatório para instalar as
  skills** (ADR-0004, estreitada pela ADR-0005 decisão 5); uma dependência de
  teste é `devDependency` do workspace `app/`, nunca do pacote publicado.
- Os seis cenários de A-013 são todos de **componente isolado ou par de
  componentes**, não jornada de usuário ponta a ponta atravessando
  `relay-host` real.

## Decisão

**Vitest + `@vue/test-utils` + `jsdom`**, sem Playwright nem Testing Library.

Considerado e descartado:

- **Playwright** — automação de browser real (Chromium/Firefox/WebKit),
  fidelidade total de foco, Tab e rede. Descartado porque nenhum dos seis
  cenários exige um browser real para ser reproduzido de forma confiável: o
  `focus-trap.ts` dos diálogos é `Element.focus()`/`tabIndex` padrão do DOM, e
  desconexão de WebSocket é simulada no cliente (`relay-client.ts`), não uma
  condição de rede real. O custo permanente — baixar binários de browser,
  suíte mais lenta, dois processos por teste — não se paga contra um ganho de
  fidelidade que os cenários não pedem. **Ressalva encontrada só ao
  implementar**: `jsdom` não calcula layout, então `offsetParent` é sempre
  `null` — e é exatamente o que `focusables()` usa para filtrar elemento
  visível de escondido. Sem correção, o teste de foco veria sempre zero (ou
  um) elemento focável e "passaria" sem provar nada. A suíte de testes
  (`test/vitest-setup.ts`) define um `offsetParent` mínimo (null só sob
  `hidden`/`inert`, o `parentElement` caso contrário) para o filtro operar
  sobre o conjunto real de elementos focáveis do diálogo. Isso é suficiente
  para o cenário de A-013, que testa contenção de Tab, não layout.
- **Testing Library** — não é um runner, é uma camada de queries
  (`getByRole`, `getByLabelText`) sobre Vitest+jsdom. Descartada como
  dependência própria porque `@vue/test-utils` já expõe `find`/`get` por
  seletor CSS suficiente para os seis cenários, e adicionar uma segunda forma
  de consultar o DOM sem necessidade real é peso sem retorno — nada aqui
  impede adotá-la depois, num componente específico, se a legibilidade dos
  testes pedir.
- **`node --test` sem framework**, mantendo o padrão atual — descartado
  porque `node --test` sozinho não monta componente Vue nem tem DOM; exigiria
  reimplementar manualmente o que `@vue/test-utils` já resolve (montagem,
  disparo de evento, espera por atualização reativa).

Vitest entra como `devDependency` só de `app/relay-ui`, executado por
`npm test --workspace relay-ui`, ao lado da suíte `node --test` já existente
naquele workspace — não a substitui, porque as duas continuam válidas para o
que já cobrem (funções puras e composables continuam em `node --test`; só
interação de componente monta em Vitest+jsdom).

## Consequências

### Positivas

- Cobre os seis cenários de A-013 sem introduzir um segundo processo
  (browser) na suíte.
- `jsdom` roda dentro do mesmo processo Node dos testes atuais — suíte
  continua rápida o bastante para rodar a cada mudança.
- `@vue/test-utils` é mantido pelo time do Vue, mesmo ciclo de vida da
  dependência já aceita na ADR-0005.

### Negativas e custos assumidos

- **`jsdom` não é um browser real.** Onde o comportamento depender de layout
  real (`getBoundingClientRect`, scroll, `IntersectionObserver`), o teste não
  serve de evidência — nenhum dos seis cenários de A-013 depende disso, mas
  uma regressão futura desse tipo exigiria revisitar esta ADR.
- Duas suítes de teste convivem em `relay-ui` (`node --test` e Vitest) até
  uma eventual migração completa, que não é escopo desta ADR.
- Mais uma dependência de terceiro (`vitest`, `@vue/test-utils`, `jsdom`) só
  em `app/relay-ui/package.json`, nunca nos demais workspaces.

## Conformidade

1. `vitest`, `@vue/test-utils` e `jsdom` entram só como `devDependency` de
   `app/relay-ui` — nunca de `relay-core`, `relay-host`, nem da raiz de
   `app/`.
2. A suíte `node --test` existente permanece; Vitest cobre só o que ela não
   alcança (montagem de componente e interação de DOM).
3. Cada um dos seis cenários de A-013 tem um teste que falha sem a correção
   correspondente já existente no código (regressão comprovada, não teste
   vazio).
4. Nenhum teste depende de um `relay-host` real rodando nem de rede — os
   seis cenários usam fixtures e stubs, consistente com a suíte atual.

## Notas

**Por que agora e não na ADR-0005.** A ADR-0005 registrou explicitamente que
decidir estratégia de teste naquele ponto seria decidir sem informação —
não havia componente construído, muito menos uma regressão real para servir
de caso de teste. A-013 chegou depois de seis regressões visuais concretas
serem encontradas e corrigidas (specs 010 e 013); esta ADR testa contra elas,
não contra cenários hipotéticos.

**O que não está decidido aqui.** Cobertura de outros componentes além dos
seis cenários citados, teste de regressão visual (screenshot diff) e testes
end-to-end contra um `relay-host` real seguem em aberto — nenhum deles é
exigido por A-013.
