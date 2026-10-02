# Snapshots da view

Cada arquivo tem o texto da tela e, depois de `--- colors ---`, um mapa de cor
com um caractere por célula. `UPDATE_SNAPSHOTS=1 cargo test --test
view_snapshots` regrava; o diff é a revisão.

| Caractere | Cor | Caractere | Cor |
| --- | --- | --- | --- |
| `f` | `fg` | `r` | `red` |
| `m` | `meta` | `i` | `id` |
| `g` | `green` | `d` | `dim` (só borda) |
| `u` | `blue` | `e` | `bar_empty` (só barra) |
| `y` | `yellow` | `#` | fundo do selo `relay` |
| `.` | célula vazia | `?` | cor fora da paleta |

Maiúscula é negrito. O fim de cada linha, vazio, é omitido.
