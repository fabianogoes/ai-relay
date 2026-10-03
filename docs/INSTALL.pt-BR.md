# Instalar o Relay

**[English](INSTALL.md)** · Português

> Tradução de [INSTALL.md](INSTALL.md). Em caso de divergência, vale o original
> em inglês.

A versão um do Relay é um pacote de plugin/skills. Ele não tem um programa de
linha de comando `relay`: as skills leem e escrevem os arquivos do protocolo
diretamente. Outros clientes podem ler, validar, derivar o estado e lançar um
harness com uma skill do Relay, mas não alteram por conta própria os cinco
registros do protocolo.

O repositório público é `https://github.com/fabianogoes/ai-relay`.

Para desenvolvimento local, troque `/absolute/path/to/relay` abaixo pelo
caminho absoluto deste checkout.

Cada instalação abaixo cria um symlink por skill, e não um symlink do diretório
`skills/` inteiro. Assim o Relay convive com as skills que o projeto já tem, e o
comando continua funcionando quando `.claude/skills/`, `.agents/skills/` ou
`.opencode/skills/` já existe — criar o symlink sobre um diretório existente
colocaria o link dentro dele em vez de substituí-lo.

## Claude Code

Os plugins do Claude Code descobrem as skills no diretório `skills/` do plugin.
Teste o checkout diretamente com:

```sh
git clone https://github.com/fabianogoes/ai-relay.git
claude --plugin-dir ./ai-relay
```

Para instalar por marketplace, adicione o repositório do GitHub e instale a
entrada `relay`:

```text
/plugin marketplace add fabianogoes/ai-relay
/plugin install relay@relay
```

Como alternativa, exponha as skills compartilhadas diretamente num projeto de
destino:

```sh
mkdir -p .claude/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .claude/skills/"$skill"
done
```

Os metadados do pacote estão em `.claude-plugin/plugin.json`. Quando o Relay
tiver um repositório público no GitHub e uma entrada de marketplace, esta seção
ganhará o comando equivalente de instalação remota.

## Codex

O manifesto do plugin do Codex é `.codex-plugin/plugin.json`, e ele expõe o
diretório canônico `./skills/`. Instale o checkout local pelo fluxo de
desenvolvimento de plugins do Codex, ou vincule as skills para desenvolvimento
no escopo do repositório:

```sh
mkdir -p .agents/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .agents/skills/"$skill"
done
```

No Codex, abra Plugins, escolha o marketplace `relay` importado do GitHub,
revise as skills listadas e selecione Install. O repositório inclui o catálogo
do Codex em `.agents/plugins/marketplace.json`, para importação no workspace.

## OpenCode

O OpenCode usa a descoberta nativa de Agent Skills. Execute estes comandos para
instalar o Relay do GitHub globalmente para o seu usuário:

```sh
git clone https://github.com/fabianogoes/ai-relay.git ~/.config/opencode/relay
mkdir -p ~/.config/opencode/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s ~/.config/opencode/relay/skills/"$skill" ~/.config/opencode/skills/"$skill"
done
```

Abra uma nova sessão do OpenCode e execute este prompt de teste:

```text
Use relay-status to report the current Relay state.
```

Resultado esperado: o OpenCode encontra `relay-status` e informa o estado sem
alterar arquivos. O OpenCode reconhece skills em `.opencode/skills/`,
`.claude/skills/` e `.agents/skills/`, além do diretório global
`~/.config/opencode/skills/`.

O OpenCode não transforma arquivos `SKILL.md` em comandos `/relay`. As skills
aparecendo em `/skills` confirmam a descoberta; invoque-as com um pedido em
linguagem natural, como `Use relay-status ...`, e o OpenCode carrega a skill
correspondente com a ferramenta nativa `skill`.

Para uma instalação local ao projeto:

```sh
mkdir -p .opencode/skills
for skill in relay-setup relay-spec relay-status relay-continue relay-session relay-tui-split; do
  ln -s /absolute/path/to/relay/skills/"$skill" .opencode/skills/"$skill"
done
```

Veja [.opencode/INSTALL.md](../.opencode/INSTALL.md) para as notas do adaptador.
A instalação no OpenCode continua sendo um link para a descoberta nativa de
Agent Skills; nenhum runtime próprio é instalado.

## Atualizar

O Relay usa um único diretório `skills/` compartilhado. Atualize o checkout ou o
plugin instalado e depois inicie uma nova sessão do harness para que o registro
de skills seja recarregado.
