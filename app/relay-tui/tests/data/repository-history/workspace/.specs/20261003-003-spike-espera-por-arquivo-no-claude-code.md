# 20261003-003 - Spike: uma skill espera um arquivo de resposta no Claude Code

## Problem

A ADR-0003 (decisão 9) considera um canal de perguntas por arquivo. O
`relay-spec` escreveria a pergunta num arquivo e esperaria; o `relay-tui`
mostraria a pergunta, e a resposta voltaria por outro arquivo. Antes de qualquer
emenda do protocolo, a ADR exige responder uma pergunta: **uma skill consegue
esperar um arquivo de forma confiável?**

Uma skill é instrução para o modelo e só espera por um comando de shell. No
Claude Code, o `Bash` em primeiro plano tem um limite de 10 minutos, menor que o
tempo de quem pensa numa decisão de escopo. Há duas alternativas: o `Bash` em
segundo plano, que acorda a sessão quando termina, e o `Monitor`. Ninguém mediu
como cada um se comporta numa espera longa, numa interrupção ou numa entrevista
com várias perguntas, nem se o modelo espera em vez de seguir com uma resposta
inventada.

## Scope

- Uma skill de teste, ferramenta deste repositório, que escreve uma pergunta num
  arquivo e espera o arquivo de resposta.
- Os três mecanismos do Claude Code medidos: `Bash` em primeiro plano com um laço
  de espera, `Bash` em segundo plano e `Monitor`.
- Medições de espera longa, latência de retomada, custo em tokens, interrupção,
  prazo esgotado e uma entrevista de três perguntas.
- A resposta é gravada à mão (`echo ... > answer`), sem o `relay-tui`.
- O resultado registrado numa ADR, com a decisão de seguir (go) ou não (no-go).

## Non-goals

- **Codex e OpenCode.** A ADR-0003 manda validar o Claude Code primeiro; os
  outros só entram depois de um go.
- **Qualquer mudança no `relay-tui`** (escolha do usuário). Um protótipo ponta a
  ponta só se justifica depois de um go.
- **Emenda do protocolo** e mudança nas skills de `skills/`. O canal continua
  inexistente até a ADR do resultado e uma spec própria.
- O formato e o lugar definitivos do canal. O spike recomenda; não decide.
- Mais de uma sessão ou de um harness esperando ao mesmo tempo.

## Decisions

**Só a viabilidade da espera, com a resposta gravada à mão** (escolha do
usuário). É o que a ADR-0003 põe como condição, e é a parte mais barata de
refutar: se a espera não for confiável, nenhum trabalho no painel se paga.

**Os arquivos ficam fora do repositório**, num diretório temporário por
workspace. Nada que pareça registro é criado, e o git não vê o spike. Onde o
canal real deve morar é uma recomendação da ADR do resultado.

**Os três mecanismos são medidos lado a lado.** O `Bash` em primeiro plano é o
mais simples, mas tem teto de 10 minutos; os outros dois podem não ter teto, mas
mudam a forma como a sessão retoma. Medir só o primeiro deixaria de fora
justamente o caso da espera longa.

**O que conta como confiável.** É go quando ao menos um mecanismo:
- sustenta uma espera de 15 minutos;
- retoma em até 5 segundos depois de a resposta ser gravada;
- tem um custo de espera que não cresce com a duração (nenhum turno do modelo
  por minuto esperado);
- passa em três entrevistas seguidas de três perguntas, sem resposta trocada nem
  pergunta pulada;
- nunca segue com uma resposta inventada quando o prazo esgota ou o usuário
  interrompe.

Os 15 minutos cobrem uma pausa real de quem decide. Abaixo disso, a pergunta
nativa do harness continua melhor.

**Prazo esgotado cai para a pergunta nativa.** A skill de teste, sem resposta no
prazo, faz a mesma pergunta pela interação nativa. É o comportamento que a
decisão 9 prevê quando a TUI não está presente, e é o que impede uma resposta
inventada.

**O resultado é uma ADR, não um relatório solto.** Seguir ou não é uma decisão
de arquitetura, e o `AGENTS.md` põe decisões e o raciocínio delas em
`docs/adr/`. As medições vão nas Notas da ADR nova, e a decisão 9 da ADR-0003
passa a apontar para ela.

**A skill de teste é ferramenta e sai ao fim.** Ela nasce em `.agents/skills/`,
com um symlink por item em `.claude/skills/` (ADR-0001), e é removida quando a
ADR é escrita. Os comandos e o texto dela ficam na ADR, de modo que o spike pode
ser refeito.

## Acceptance criteria

- A-001 - Uma skill de teste em `.agents/skills/` escreve uma pergunta num
  arquivo fora do repositório e espera o arquivo de resposta, e cai para a
  pergunta nativa quando o prazo esgota
- A-002 - Os três mecanismos (`Bash` em primeiro plano com um laço de espera,
  `Bash` em segundo plano e `Monitor`) têm medidos a espera máxima sustentada, a
  latência entre a resposta gravada e a retomada, e os tokens gastos durante a
  espera, em esperas de 30 segundos, 5, 15 e 45 minutos
- A-003 - Uma interrupção do usuário e uma mensagem enviada durante a espera
  foram testadas em cada mecanismo, e nenhuma produziu uma resposta inventada
- A-004 - Três entrevistas seguidas de três perguntas retomaram com a resposta
  certa em cada pergunta, no mecanismo recomendado
- A-005 - Uma ADR nova registra o go ou no-go pelos critérios desta spec, com as
  medições, o mecanismo e o lugar recomendados para o canal (ou o motivo da
  recusa), e a decisão 9 da ADR-0003 aponta para ela
- A-006 - O protocolo, as skills de `skills/`, o `relay-tui` e os registros não
  mudaram por causa do spike, e a skill de teste saiu do repositório

## Backlog candidates

- B-068: Skill de teste do canal por arquivo e roteiro de medição
- B-069: Medições dos três mecanismos, interrupção, prazo e entrevista
  (needs: B-068)
- B-070: ADR com go ou no-go, ADR-0003 atualizada e skill de teste removida
  (needs: B-069)
