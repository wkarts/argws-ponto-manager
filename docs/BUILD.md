# Guia de build

Web/PWA:

```bash
npm ci
npm run ci:version
npm run build:web
```

Desktop:

```bash
cargo check --manifest-path src-tauri/Cargo.toml --locked --all-targets --all-features
npm run tauri:build
```

No Windows, `npm run build:windows:short-target` usa um `CARGO_TARGET_DIR` curto para evitar `MAX_PATH` e `LNK1104`. Os targets oficiais são Windows x64/x86, Linux x64/ARM64 e macOS Intel/Apple Silicon. Builds desktop devem ocorrer no sistema correspondente; cross-link não substitui smoke test nativo.

CloudPanel usa `npm run build:linux:cloudpanel:x64` ou `:x86`. Artefatos não são versionados.
