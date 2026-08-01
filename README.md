# Ponto Manager

Sistema corporativo da ARGWS para gestão, tratamento, apuração e controle de ponto. A versão 1.23.4 preserva o domínio da aplicação existente e adota a arquitetura corporativa Tauri 2 + Rust + Vue 3 + TypeScript.

## Funcionalidades preservadas

- autenticação local, sessão persistente, usuários, empresas, perfis e matriz de permissões;
- departamentos, funções, centros de custo, horários, escalas, equipamentos e eventos;
- funcionários, jornadas, batidas manuais e em lote, férias e feriados;
- importação e reimportação de AFD, tratamento, apuração, banco de horas e fechamento mensal;
- espelho/cartão de ponto, relatórios, exportação CSV, impressão HTML/A4 e arquivos REP;
- fila local de sincronização, auditoria, integrações, configurações e logs.

## Runtimes

- desktop Tauri para Windows, Linux e macOS;
- Web/PWA com provider controlado para recursos nativos;
- servidor Rust headless e API interna autenticada;
- CLI, worker, serviço Windows e serviço Linux;
- pacote CloudPanel x64/x86;
- SQLite padrão, com providers opcionais MySQL e PostgreSQL.

## Segurança do primeiro acesso

Na instalação nova, o usuário `admin` recebe uma credencial exclusiva gerada localmente em `.bootstrap-admin.local`, com permissão restrita. O arquivo é ignorado pelo Git e removido após a troca obrigatória de senha. Não existe senha inicial pública.

## Desenvolvimento

Requisitos: Node.js 24, npm, Rust stable e dependências nativas do Tauri.

```bash
npm ci
npm run ci:version
npm run typecheck
npm run build:web
npm run tauri:dev
```

Validação completa:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo check --manifest-path src-tauri/Cargo.toml --locked --all-targets --all-features
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets --all-features
```

## Modos sem interface

```bash
npm run tauri:server
npm run tauri:cli
npm run tauri:worker
```

A API, webhooks e WebSocket usam loopback por padrão. Bind público exige autorização explícita, token e CORS restrito.

## Documentação

- [Arquitetura](docs/ARQUITETURA.md)
- [Desenvolvimento](docs/DESENVOLVIMENTO.md)
- [Build](docs/BUILD.md) e [release](docs/RELEASE.md)
- [Instalação](docs/INSTALACAO.md), [atualização](docs/ATUALIZACAO.md) e [CloudPanel](docs/CLOUDPANEL.md)
- [Matriz de migração](docs/MATRIZ_MIGRACAO.md) e [compatibilidade](docs/MATRIZ_COMPATIBILIDADE.md)
- [Migração do banco legado](docs/MIGRACAO_BANCO_LEGADO.md) e [backup/rollback](docs/BACKUP_ROLLBACK.md)
- [Variáveis de ambiente](docs/VARIAVEIS_AMBIENTE.md)
- [Segurança e licenciamento desativado](docs/SEGURANCA_LICENCIAMENTO.md)
- [Validação da migração](docs/VALIDACAO_MIGRACAO.md) e [solução de problemas](docs/TROUBLESHOOTING.md)

## Identidade

- produto: Ponto Manager;
- identificador: `br.com.argws.pontomanager`;
- pacote npm: `argws-ponto-manager`;
- crate: `argws_ponto_manager`;
- diretório de dados: `argws-ponto-manager`;
- prefixo de ambiente: `ARGWS_PONTO_MANAGER_`;
- publisher: ARGWS; desenvolvedor: Wallace Kleiton.

O licenciamento está preservado como módulo isolado para uso futuro, mas permanece integralmente desativado e não bloqueia nenhuma função.
