# Arquitetura

O Ponto Manager usa uma única base Vue 3/TypeScript com Pinia e Vue Router. A fronteira de execução é abstraída por providers: Tauri invoca comandos Rust; Web/PWA usa a API interna autenticada e retorna indisponibilidade controlada quando uma capacidade depende de desktop.

O backend Rust concentra regras, validação de DTOs, serviços, repositórios e transações. `SharedState` resolve o diretório `argws-ponto-manager`, executa a recuperação do banco legado antes das migrations e só publica o estado depois de validar o resultado.

Camadas principais:

1. `src/pages`, `src/components`, `src/stores` e `src/router`: interface e estado;
2. `src/core/invoker`: contratos Tauri/Web;
3. `src-tauri/src/commands`: casos de uso do domínio;
4. `src-tauri/src/internal_api`, `native_webhook` e `native_websocket`: serviços locais autenticados;
5. `src-tauri/src/core/database`: SQLite e providers opcionais MySQL/PostgreSQL;
6. `src-tauri/src/legacy_data.rs` e `migrations.rs`: proteção e evolução dos dados;
7. `src-tauri/src/cli` e `service`: headless, CLI, worker e serviços.

Recursos Tauri usam capability mínima `core:default`; CSP bloqueia objetos, framing e scripts externos. Licenciamento existe apenas como módulo inativo, sem rota, menu, trial, registro ou verificação remota automática.
