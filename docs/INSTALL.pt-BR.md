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

O instalador sem clone copia somente os arquivos publicados das skills do Relay
para `.claude/skills/` e preserva as outras skills. As instruções de
desenvolvimento abaixo criam um symlink por skill, e não para o diretório
`skills/` inteiro; assim funcionam mesmo quando o diretório de skills do harness
já existe.

## Claude Code

### Instalar no projeto atual

Na raiz do projeto novo, execute este comando no terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/fabianogoes/ai-relay/main/install.py | python3 -
```

Ele baixa somente os arquivos das skills do Relay do GitHub e os instala em
`.claude/skills/`. Não clona nem baixa o repositório Relay. Também verifica a
release mais recente de `relay-tui` e instala ou atualiza o binário do usuário
quando necessário. Requer Python 3, usado somente pela biblioteca padrão.
Execute novamente para atualizar as skills e a TUI com segurança. Inicie uma
nova sessão do Claude Code e execute `/relay-setup` neste projeto para
inicializar os registros do Relay.

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

Os metadados do pacote estão em `.claude-plugin/plugin.json`.

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

## Configurar outro projeto nesta máquina

Na raiz de um projeto novo, instale as skills uma vez para o harness usado. No
Claude Code, execute o instalador da seção acima. No Codex ou OpenCode, use as
instruções de instalação correspondentes acima. Inicie uma nova sessão e
invoque `relay-setup` no harness:

| Harness | Invocação |
| --- | --- |
| Claude Code | `/relay-setup` |
| Codex ou OpenCode | `Use relay-setup` |

`relay-setup` configura o repositório onde é executado. Cria os arquivos Relay
ausentes e acrescenta a seção gerenciada ao `AGENTS.md` sem substituir
instruções ou registros existentes. Se ainda não houver idioma padrão, pergunta
uma vez e salva a escolha. É seguro executá-lo novamente: preserva o conteúdo
existente e não recria o que já está presente. Execute `relay-status` no mesmo
repositório para conferir o resultado.

O binário `relay-tui` é instalado para a máquina, não copiado para cada projeto.
Execute `relay-tui --version` e compare com a versão mais recente em
[Releases](https://github.com/fabianogoes/ai-relay/releases). Se estiver
atualizado, não precisa fazer nada; se estiver ausente ou desatualizado, siga
[as instruções de atualização da TUI em TUI.pt-BR.md](TUI.pt-BR.md#atualizar).
Depois, invoque `relay-tui-split` na raiz do novo repositório para abrir o painel
nesse workspace. Revise e faça commit da configuração gerada que deve ser
compartilhada com o repositório.

## Atualizar

O Relay mantém um único diretório canônico `skills/`. Para atualizar uma
instalação local ao projeto, execute novamente o comando de instalação acima na
raiz do projeto e inicie uma nova sessão do Claude Code para carregar as skills.
Em instalações por plugin ou checkout, atualize o plugin ou checkout e inicie
uma nova sessão.
