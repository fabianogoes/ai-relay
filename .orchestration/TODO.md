# Active task: B-039

- [x] T-001 - Emendar a spec e a ADR-0009 para "macOS requisito, Linux e Windows melhor esforco" (A-008, escopo, decisao 4, conformidade) e registrar a excecao dos workflows a conformidade 3 da ADR-0004
- [x] T-002 - Perfil release e script de empacotamento do relay-tui (tar.gz com binario, licenca e README, mais sha256), testado localmente nos dois alvos macOS (needs: T-001)
- [x] T-003 - Workflows de CI e de release em `.github/workflows/`: macOS e requisito, Linux e Windows melhor esforco que nunca bloqueia, versao da tag conferida com o Cargo.toml (needs: T-002)
- [x] T-004 - Documentacao de uso: README do relay-tui (download, Gatekeeper, uso, build), `app/README.md`, a excecao na regra de `app/AGENTS.md` e uma frase no README da raiz (needs: T-003)
- [x] T-005 - Verificacao final e fechamento, dizendo o que so um run real do GitHub Actions confirma (needs: T-004)
- [!] T-006 - Run real no GitHub: o CI passa no macOS e uma tag `relay-tui-v<versao>` publica os arquivos de macOS arm64 e x64 no Release, comprovando o A-008 (needs: T-005)
