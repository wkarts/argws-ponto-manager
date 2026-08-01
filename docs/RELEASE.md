# Guia de release

A release coordenada só parte de CI aprovado em `main` ou de uma retomada manual com tag existente. A versão deve coincidir em `VERSION`, npm, Cargo, Tauri e configuração do frontend.

O workflow faz checkout da tag imutável, usa `npm ci` e `Cargo.lock --locked`, cria um único draft, produz desktop, Web/PWA e CloudPanel, renomeia por produto/versão/SO/arquitetura e gera `SHA256SUMS.txt` e `RELEASE-MANIFEST.json`.

A retomada recusa tag movida, release publicada ou asset divergente. Assets idênticos podem ser reutilizados. A PR de migração não publica release oficial.

Passos operacionais:

1. homologar a PR e executar todos os runners;
2. fazer merge sem mover a versão indevidamente;
3. aguardar o CI de `main`;
4. revisar o draft e checksums;
5. publicar somente após smoke tests e aprovação de rollout/rollback.
