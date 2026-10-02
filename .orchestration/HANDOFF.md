# Handoff

- Status: blocked
- Backlog: B-039
- TODO: T-006
- Spec: .specs/20261002-001-relay-tui-observador-em-rust.md
- Harness: claude-code
- Updated: 2026-10-02T15:16:02-03:00

## Objective
Comprovar o criterio A-008 da spec 20261002-001: um run real do GitHub Actions mostra o CI passando no macOS e uma tag publicando no Release os arquivos de macOS arm64 e x64 (Linux e Windows, se saírem, sao bonus).

## Next step
O dono do repositorio faz o commit dos arquivos novos (incluindo app/relay-tui/Cargo.lock), faz push para a main e confere o run do workflow 'relay-tui CI'. Depois sobe a versao em app/relay-tui/Cargo.toml (hoje 0.0.0), commita e envia a tag relay-tui-v<versao> (por exemplo relay-tui-v0.1.0). Com os runs na mao, retomar com a skill relay-session: registrar T-006 com a evidencia (links ou saida dos runs, nomes dos arquivos no Release) nomeando A-008, e so entao fechar o B-039.

## Context
Bloqueio: nenhum workflow foi executado, porque isso exige commit, push e uma tag, que nao foram pedidos. Condicao de retomada: (1) o job 'macOS (required)' do relay-tui CI verde numa execucao real; (2) uma tag relay-tui-v<versao> que passe pelo job 'version' (tag igual a versao do Cargo.toml) e publique no Release os arquivos relay-tui-<versao>-aarch64-apple-darwin.tar.gz e ...-x86_64-apple-darwin.tar.gz com seus .sha256. Tudo que pode ser verificado localmente ja foi (T-005: 88 testes Rust, 53 no relay-core, os dois pacotes macOS gerados e conferidos, YAML e shell validos). Riscos a observar no primeiro run: sintaxe de matrix e needs com always(), continue-on-error dos jobs best-effort, permissoes de contents: write para o gh release create, o runner ubuntu-24.04-arm (so existe para repositorios publicos), e cargo build --locked (Cargo.lock precisa estar commitado). O working tree tem muita coisa nao commitada de antes desta sessao: revisar o que entra no commit. Um teste de quarentena que fiz no macOS pode ter aberto um dialogo do Gatekeeper na tela do usuario: dispensar.
