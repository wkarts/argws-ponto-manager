# Guia de desenvolvimento

Use Node.js 24 e Rust stable. Em Linux, instale WebKitGTK 4.1, GTK/AppIndicator, OpenSSL, librsvg, build-essential e patchelf.

```bash
npm ci
npm run dev
```

Para desktop, use `npm run tauri:dev`; para PWA, `npm run build:web && npm run preview`. Os modos `--mode=headless-api`, `--mode=cli` e `--mode=worker` não abrem janela.

Antes de commit:

```bash
npm run ci:version
npm run typecheck
npm run build:web
npm run fmt:rust:check
npm run lint:rust
npm run test:rust
```

Não use `npm install` no CI. Não versione `.env`, `.bootstrap-admin.local`, bancos, logs, tokens, certificados ou diretórios de build. Novos comandos devem validar DTOs na fronteira, parametrizar SQL, usar transação em alterações críticas e devolver erros sem `panic!`.
