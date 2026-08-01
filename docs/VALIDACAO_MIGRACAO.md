# Validação da migração

Data: 2026-08-01. Base: `main` em `60d448ee9f846667d03829907d8a2ef70c2dca77`.

## Executado e aprovado

- verificação inicial de identidade e validação estrutural do destino isolado;
- verificação final de identidade e validadores estruturais oficial/local;
- `actionlint 1.7.12` em todos os workflows;
- `npm ci` com cache isolado;
- `npm run ci:version`;
- `npm run typecheck`;
- `npm run build:web` (Web/PWA);
- `cargo generate-lockfile`;
- `cargo check --locked --all-targets --all-features` para `x86_64-pc-windows-gnu`;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` para o mesmo alvo;
- validação do título exato da Pull Request;
- validação sintática dos scripts Bash e Node.js de entrega;
- harness Rust com sete cenários: instalação nova, atualização/repetição/reinício, banco corrompido, falha de cópia/espaço/permissão, falha de migration, rollback de destino existente e lifecycle/permissão do bootstrap.

## Executado e bloqueado pelo ambiente

- check/test nativos Linux: toolchain disponível, porém o runtime não fornece `pkg-config`, GTK e WebKitGTK de desenvolvimento;
- `cargo test --no-run` cruzado: compilou dependências e aplicação, mas o linker Zig não aceita o arquivo `.def` emitido para a DLL Windows.

## Dependente dos runners da PR

- testes Rust nativos com todos os targets/features;
- smoke desktop Windows/Linux/macOS;
- instalação limpa e atualização com amostra real da versão anterior;
- impressão nativa e integrações em rede;
- pacotes CloudPanel x64/x86;
- matriz completa de instaladores e checksums de release.

Nenhum item dependente de runner é declarado aprovado. A PR deve permanecer draft até esses gates e a homologação funcional serem concluídos.
