# Guia de release

A release coordenada só parte de CI aprovado em `main` ou de uma retomada manual com tag existente. A versão deve coincidir em `VERSION`, npm, Cargo, Tauri e configuração do frontend.

O workflow faz checkout da tag imutável, usa `npm ci` e `Cargo.lock --locked`, cria um único draft, produz desktop, Web/PWA e CloudPanel, renomeia por produto/versão/SO/arquitetura e gera `SHA256SUMS.txt`, `RELEASE-MANIFEST.json`, `RELEASE-STATUS.json` e `RELEASE-STATUS.md`.

Antes de criar a tag, a preparação sincroniza a versão do pacote workspace no `Cargo.lock` e executa `cargo metadata --locked`. Assim, a tag nunca é criada com `Cargo.toml` e lockfile divergentes.

## Publicação parcial controlada

Os jobs da matriz continuam independentes e `fail-fast` permanece desligado. Se uma arquitetura falhar:

- o job e o workflow continuam marcados como falha;
- o alvo reprovado não é publicado;
- os artefatos aprovados dos demais alvos são coletados, validados por SHA-256 e publicados;
- o diagnóstico e o link da execução são registrados nas notas, no manifesto e no resumo do workflow;
- uma nova execução pode completar os alvos ausentes sem sobrescrever assets divergentes.

O pacote CloudPanel é headless e deve ser compilado com `--no-default-features`. A dependência `tauri` é opcional e ativada exclusivamente pela feature `desktop`; isso é obrigatório porque o próprio Tauri inclui GTK como dependência de plataforma no Linux mesmo sem Wry. GTK, GDK, WebKitGTK, AppIndicator, tray e Wry não podem aparecer no grafo normal do CloudPanel x86/x64.

Antes de compilar, `build-cloudpanel-release.sh` executa `cargo tree` para rejeitar qualquer crate gráfica/Tauri no grafo headless. O CI de Pull Request também gera e armazena por um dia os pacotes CloudPanel x64 e x86, impedindo que a validação fique restrita a uma inspeção textual do script.

A retomada recusa tag movida, release publicada ou asset divergente. Assets idênticos podem ser reutilizados. A PR de migração não publica release oficial.

Passos operacionais:

1. homologar a PR e executar todos os runners;
2. fazer merge sem mover a versão indevidamente;
3. aguardar o CI de `main`;
4. revisar checksums, manifesto e o estado completo/parcial da matriz;
5. homologar os artefatos disponíveis e registrar qualquer alvo ausente;
6. publicar somente após smoke tests e aprovação de rollout/rollback.
