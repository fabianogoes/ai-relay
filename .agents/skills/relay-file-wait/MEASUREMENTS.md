# Roteiro do spike de espera por arquivo

Este roteiro prepara B-069 da spec
`.specs/20261003-003-spike-espera-por-arquivo-no-claude-code.md`.
B-068 entrega a ferramenta e os ensaios curtos; nenhuma espera longa nem decisão
de go/no-go está concluída por este documento. A skill e seus auxiliares saem em
B-070, depois de os comandos e resultados serem preservados na ADR.

## Preparação

Use uma sessão **interativa do Claude Code**, neste workspace, e um segundo
terminal para o operador. Comece uma sessão nova depois de instalar o symlink
por item `.claude/skills/relay-file-wait` para `.agents/skills/relay-file-wait`.
Não use Codex, OpenCode, a TUI ou uma alteração de `relay-spec` neste experimento.

Registre `claude --version`, modelo, sistema operacional, modo interativo ou
headless, ferramentas realmente disponíveis, argumentos de timeout e qualquer
configuração local que altere limites. Não copie credenciais. Mantenha as
configurações constantes durante uma série; uma alteração inicia outra série.
Confirme que `AskUserQuestion` está disponível antes da série. Uma chamada que
retorna “No such tool available” é `unsupported`, não um fallback concluído.
Um wrapper que captura entrada/saída pode impedir a interação mesmo se o comando
não tiver `-p`; confirme o modo na sessão real, sem inferi-lo só dos argumentos.
Antes da espera, o operador identifica os campos de uso de tokens da versão
instalada e salva uma amostra do transcript para conferir a leitura.

Na versão local consultada, `claude --version` retornou `2.1.306`.
A documentação distingue limites de comandos em background em sessões locais
e headless; uma execução `claude -p` não substitui uma medição interativa.
Bash pode mover automaticamente um comando para background. Monitor transmite
linhas de stdout e pode encerrar pelo próprio prazo. Registre o comportamento
real e o schema disponível, sem mudar de mecanismo ou reiniciar o observador
para esconder um limite. Fontes consultadas em 2026-10-21:
[ferramentas](https://code.claude.com/docs/en/tools-reference) e
[controles interativos](https://code.claude.com/docs/en/interactive-mode).

## Uma pergunta, um diretório

O agente invoca `/relay-file-wait` com workspace, pergunta, mecanismo e prazo.
O auxiliar usa somente a biblioteca padrão Python. A publicação gera uma pasta
temporária própria, identificada pelo workspace e por um sufixo aleatório,
fora dele. Cada pergunta tem um ID novo. Não reutilize a pasta na pergunta seguinte.

Exemplo para uma espera nominal de 30 segundos, com 120 segundos de margem
para o operador responder antes do prazo do auxiliar:

```sh
rtk proxy python3 .agents/skills/relay-file-wait/scripts/file_wait.py prepare --workspace '/caminho/absoluto/do/workspace' --prompt 'Qual opção: A ou B?' --timeout 150
```

Copie o `directory` e o `id` retornados. O prazo começa na publicação.
Se a preparação ou uma aprovação consumir a margem, descarte a tentativa e
publique uma pergunta nova; não estenda uma pergunta já em medição.
O agente mostra a pergunta e inicia **uma** chamada de espera:

```sh
rtk proxy python3 .agents/skills/relay-file-wait/scripts/file_wait.py wait '<directory>'
```

O operador espera o arquivo `started.json` aparecer antes de iniciar a contagem
da duração nominal. No segundo terminal, ao terminar essa duração, grava a
resposta humana de forma atômica:

```sh
rtk proxy python3 .agents/skills/relay-file-wait/scripts/file_wait.py reply '<directory>' 'B'
```

Esse comando grava `answer.json`; não envia uma mensagem ao agente. Também é
possível escrever JSON manualmente em um arquivo provisório e renomeá-lo para
`answer.json`, com `id` igual ao da pergunta e `answer` como string não vazia.
Nesse caso registre o instante da publicação atômica separadamente.
Nunca peça ao próprio agente para executar `reply` durante uma medição.

`wait` não imprime progresso: emite uma linha JSON terminal, salva também em
`outcome.json`. Sucesso exige `status: answered`, ID atual e resposta literal.
Os códigos são 0 para resposta, 124 para prazo e 130 para interrupção capturada.
Erro de ferramenta, ausência de saída ou tarefa encerrada não são respostas.
Prazo e interrupção não reabrem a pergunta; uma resposta tardia fica sem efeito.

## Três mecanismos

| Mecanismo pedido | Chamada | O que registrar |
| --- | --- | --- |
| `foreground` | Bash com `command` acima e sem `run_in_background` | `timeout` efetivo; retorno direto ou migração automática para background |
| `background` | Bash com o mesmo `command` e `run_in_background: true` | `timeout` efetivo, task ID, notificação de conclusão e retomada real |
| `monitor` | Monitor com o mesmo `command` | `timeout_ms` efetivo, watch/task ID, evento da linha de stdout e retomada real |

Solicite um prazo da ferramenta que cubra o prazo do auxiliar quando o schema
permitir. Não altere variáveis globais para forçar uma ferramenta a passar.
Se o limite não cobrir a duração, registre `tool-limit`; se a ferramenta não
existir, `unsupported`. Se `foreground` migrar, registre `auto-backgrounded` e
o instante da migração: a duração posterior não comprova espera em foreground.
Não use `&`, `nohup`, cron ou um segundo comando para mascarar a vida da tarefa.
Um ID sem evento terminal mantém a pergunta pendente; não faça turnos de polling.

## Matriz de B-069

Para **cada** mecanismo, tente 30 s, 300 s, 900 s e 2700 s, sempre em diretórios
novos. O prazo do auxiliar é a duração nominal mais 120 s. Registre a duração
efetiva até a resposta ou encerramento, inclusive tentativas limitadas pelo
harness. Não transforme uma espera de 45 minutos em várias esperas menores.
Se uma tentativa não puder começar por ausência da ferramenta, registre isso
em vez de atribuir duração ou latência fictícia.

Para cada tentativa, preserve no diretório temporário os seguintes campos:

| Campo | Origem |
| --- | --- |
| sessão, versão, modelo, mecanismo pedido e efetivo | transcript e configuração observada |
| pergunta, ID, diretório, duração nominal, prazo do auxiliar e da ferramenta | `question.json` e argumentos da chamada |
| início, publicação da resposta, observação pelo processo, retomada do agente | `started.json`, `answer.json`, `outcome.json` e transcript |
| espera efetiva e latência até a retomada | diferenças entre os instantes acima, na mesma máquina |
| turnos e tokens durante a espera | mensagens do modelo após início da espera e antes da resposta humana |
| tokens da retomada | mensagem(s) do modelo após a notificação, contadas separadamente |
| término, saída, fallback, erros e caminho do transcript | artefatos e observação do operador |

`written_at_ns` e `observed_at_ns` medem somente a detecção pelo **processo**.
A latência de interesse termina quando o **agente** volta a agir: use o
timestamp da primeira mensagem/ação correspondente no transcript. Não reporte
a latência do polling Python como se fosse a retomada do Claude Code.
Se a fonte tiver precisão insuficiente para decidir o limite de 5 s, marque
a latência como inconclusiva e repita com captura de precisão adequada.

Some os campos reais de uso das mensagens do modelo, incluindo separadamente
tokens de entrada, saída e cache quando disponíveis. Preserve os valores brutos
e a fórmula usada; não infira tokens por duração, tamanho do arquivo ou custo.
Sem uso disponível, registre `unavailable`, nunca zero. Compare 30 s, 5, 15 e
45 min: nenhum turno por minuto deve ser necessário enquanto o humano pensa.

## Prazo, interrupção e mensagem durante a espera

Em cada mecanismo, faça tentativas adicionais e independentes:

1. **Prazo sem resposta:** publique com `--timeout 2`, não grave resposta e
   confira que o agente repete a mesma pergunta em `AskUserQuestion`. Um
   timeout da pergunta nativa ou opção pré-selecionada não conta como resposta.
2. **Interrupção:** interrompa a espera pela interface, pare a tarefa/watch se
   ainda estiver ativo e não escolha uma opção. Grave uma resposta atrasada
   no diretório antigo; o agente deve ignorá-la e aguardar sua orientação.
   Registre se o harness matou o processo sem permitir `outcome.json`.
3. **Mensagem durante a espera:** envie “ainda estou pensando; não respondi”.
   Confira se a mensagem interrompe, mantém ou move a tarefa, e se a pergunta
   permanece sem resposta. Uma mensagem de status não substitui `answer.json`.
4. **Resposta de outra pergunta:** copie um `answer.json` antigo para a nova
   pasta; o ID divergente deve ser ignorado. Depois publique a resposta correta.

Se houver fallback nativo, registre-o como fallback, nunca sucesso do canal por
arquivo. Após cancelar uma pergunta, uma retomada usa ID e diretório novos.
Mantenha apenas uma sessão e uma pergunta em espera por vez.

## Entrevista e decisão posterior

No mecanismo que as medições indicarem, rode três entrevistas seguidas, cada
uma com três perguntas distintas. O operador escolhe respostas diferentes,
por exemplo `A`, `C`, `B`, e confere ID e texto a cada retomada antes de publicar
a próxima pergunta. Não exponha ao agente uma sequência esperada de respostas.
Um erro de correlação ou pergunta pulada é falha, mesmo que a entrevista termine.

Para recomendar go em B-070, pelo menos um mecanismo deve sustentar 15 min,
retomar em até 5 s, dispensar turnos do modelo por minuto, passar as três
entrevistas e não inventar resposta no prazo ou na interrupção. Preserve também
os resultados negativos dos outros mecanismos e das durações maiores.
Este roteiro não declara aprovação antecipada.

Copie as evidências úteis para a ADR do resultado antes de remover qualquer
artefato. Depois de parar todos os processos, apague somente os diretórios
temporários listados nas tentativas. B-070 remove a skill, o symlink e os testes
temporários; os registros operacionais permanecem como memória do trabalho.
