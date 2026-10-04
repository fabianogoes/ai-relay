# ADR-0009 — instalação das skills sem checkout do Relay

## Status

**Accepted** — 2026-10-04.

## Contexto

A instalação anterior exigia clone do Relay, marketplace já instalado no
Claude Code ou symlinks para um checkout local. Um projeto novo aberto no
Claude Code não encontrava as skills sem uma dessas preparações. O binário da
TUI, por outro lado, já é publicado separadamente em GitHub Releases.

## Decisão

1. `install.py`, na raiz do Relay, pode ser executado por pipe no Python 3 a
   partir da raiz do projeto alvo. Ele resolve o commit atual de `main` pela API
   do GitHub, lê a árvore desse commit e baixa individualmente apenas os arquivos
   em `skills/relay-*/`. Confere cada download pelo Git blob SHA da árvore.
2. As skills são instaladas em `.claude/skills/`. Uma nova execução atualiza as
   pastas Relay publicadas, preserva skills alheias, não segue symlinks em
   `.claude/skills/` e prepara os arquivos antes de substituir os existentes.
3. O instalador consulta a release estável mais recente da TUI. Quando não há
   `relay-tui` ou sua versão é anterior, baixa o binário da plataforma, valida o
   checksum publicado e instala em `~/.local/bin/relay-tui`. Não usa `sudo` nem
   rebaixa uma versão igual ou mais nova.
4. O instalador não escreve `AGENTS.md`, `.specs/` nem `.orchestration/`.
   Depois de reiniciar o Claude Code, a pessoa executa `/relay-setup`, a skill
   responsável por criar e atualizar os registros do projeto.
5. O script requer apenas Python 3 e a biblioteca padrão; não requer Git, clone,
   Cargo, gerenciador de pacotes ou build.

## Consequências

- A pessoa instala as skills do projeto sem baixar o checkout ou um arquivo do
  repositório inteiro.
- Arquivos das skills acompanham o projeto e podem ser compartilhados em Git;
  executar o comando novamente atualiza a cópia local para a versão atual de
  `main`.
- O pacote dá suporte ao Claude Code nesta primeira instalação automática. Os
  caminhos para Codex e OpenCode continuam disponíveis no guia de instalação.
- Uma execução pode concluir a atualização das skills e falhar ao instalar a
  TUI; a mensagem deve identificar os dois resultados, e repetir é seguro.
- Python 3 passa a ser requisito do fluxo automatizado de instalação.

## Compliance

1. `install.py` baixa apenas arquivos `skills/relay-*` da árvore Git e assets
   específicos de Release; não clona nem extrai o repositório.
2. Os testes em `.agents/tests/install.test.py` cobrem substituição idempotente,
   preservação de skills alheias, paths inseguros, checksum e versões da TUI.
3. O instalador não altera registros do protocolo; `relay-setup` permanece o
   único caminho de inicialização do workspace.

## Notes

Origem: solicitação do usuário para instalar Relay num projeto novo sem baixar
o repositório do Relay.
