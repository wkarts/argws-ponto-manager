# Matriz de migração

O inventário completo do `main` no commit `60d448ee9f846667d03829907d8a2ef70c2dca77` está em `docs/inventario-ponto-manager.json`. A situação “migrado” significa presença no destino e validação estática/compilável; evidências executadas estão em `VALIDACAO_MIGRACAO.md`.

| Origem | Destino | Estratégia | Validação | Situação |
| --- | --- | --- | --- | --- |
| Identidades divergentes | Identidade Ponto Manager única | substituir | verificador de identidade e busca textual | migrado |
| Tauri 2/Vue/TS existentes | Base corporativa Tauri 2/Vue 3/TS/Vite | adaptar | typecheck e build Web | migrado |
| Login e sessão local | Argon2, sessão e primeiro acesso obrigatório | adaptar | compile/check e fluxo frontend | migrado |
| Credencial pública antiga | bootstrap aleatório local 0600 | descartar | harness de segurança; busca de segredo | migrado; descarte obrigatório por segurança |
| Usuários, empresas e perfis | comandos, páginas e permissões | portar | check/clippy e rotas | migrado |
| Departamentos, funções, centros de custo | entidades genéricas tipadas | portar | check/clippy e navegação | migrado |
| Horários, escalas, equipamentos e eventos | entidades/repositórios/páginas | portar | check/clippy e rotas | migrado |
| Funcionários e jornadas | comandos específicos e páginas | portar | typecheck/check/clippy | migrado |
| Batidas manuais/lote | comandos transacionais e telas | portar | check/clippy | migrado |
| AFD e reimportação | importador, histórico e tratamento | preservar/adaptar | check/clippy; homologação de arquivos pendente | migrado |
| Férias | regras de conflito, cancelamento e histórico | adaptar | check/clippy e permissões | migrado |
| Feriados | fonte, importação e CRUD | portar | check/clippy | migrado |
| Apuração/tratamento | serviços e páginas do domínio | preservar/adaptar | check/clippy; golden data pendente | migrado |
| Banco de horas/fechamento | comandos e relatórios | portar | check/clippy | migrado |
| Espelho/cartão/relatórios | HTML/A4, analítico/sintético | adaptar | build Web/check; impressão nativa pendente | migrado |
| CSV/REP | exportadores sanitizados | portar | check/clippy | migrado |
| Fila de sincronização | tabela, auditoria, tela e worker observável | adaptar | check/clippy | migrado |
| SQLite antigo | cópia segura para novo slug | adaptar | 7 cenários executáveis | migrado |
| Provider SQLite | repositórios locais | preservar | harness/check | migrado |
| MySQL/PostgreSQL | providers opcionais por feature | adaptar | check/clippy all-features; runner nativo pendente | migrado |
| API interna | Axum autenticado, loopback por padrão | substituir | check/clippy e configuração | migrado |
| Webhook/WebSocket | serviços opt-in com token | adaptar | check/clippy; rede real pendente | migrado |
| Desktop | provider Tauri e capability mínima | adaptar | check/clippy; smoke nativo pendente | migrado |
| Web/PWA | provider Web/API e fallback controlado | adaptar | typecheck/build | migrado |
| CLI/worker/headless | modos do mesmo binário Rust | adaptar | check/clippy; link nativo pendente | migrado |
| Serviços Windows/Linux | scripts e argumentos headless | adaptar | sintaxe estática; runner nativo pendente | migrado |
| CloudPanel | pacote x64/x86 e scripts operacionais | substituir | validação estática; pacote no runner pendente | migrado |
| CI antigo com `npm install` | Node 24, `npm ci`, locks e gates completos | substituir | actionlint/local e Actions | migrado |
| Release antiga | draft coordenado e idempotente | substituir | actionlint/script checks; dry-run remoto pendente | migrado |
| Licenciamento existente | módulo isolado sem ativação | desativar | verificador e metadado API falso | migrado; desativação exigida pelo produto |
| Demonstrações da base corporativa | somente componentes-base reutilizáveis | descartar | navegação/rotas e busca textual | migrado; não pertencem ao domínio |
| Branding antigo/genérico | assets oficiais do repositório e nova paleta | substituir | auditoria visual/textual | migrado |

Nenhum módulo funcional inventariado foi descartado. Os únicos descartes são a credencial insegura e conteúdo demonstrativo que nunca pertenceu ao Ponto Manager.
