// node-pty distribui `spawn-helper` pré-compilado em `prebuilds/<platform>/`;
// a extração do tarball do npm às vezes não preserva o bit de execução desse
// binário (observado em instalação limpa neste workspace), e sem ele
// `pty.spawn()` falha com "posix_spawnp failed" em runtime, não em install.
// Este script corrige isso depois de cada install, sem builds silenciosos:
// se o arquivo não existir, não faz nada (outra plataforma, ou pacote ausente).
import { chmodSync, existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const prebuildsDir = fileURLToPath(new URL('../../node_modules/node-pty/prebuilds/', import.meta.url))
const platforms = ['darwin-arm64', 'darwin-x64']

for (const platform of platforms) {
  const helper = `${prebuildsDir}${platform}/spawn-helper`
  if (existsSync(helper)) chmodSync(helper, 0o755)
}
