# Migração do banco legado

`legacy_data.rs` executa antes das migrations normais. O destino atual é `%LOCALAPPDATA%/argws-ponto-manager/ponto-manager.db`. O fluxo encontra `%LOCALAPPDATA%/pontos_desktop_tauri/pontos.db` e variações legadas sem alterar seu conteúdo, respeita `data_dir_override` do `bootstrap.json`, faz `PRAGMA integrity_check`, calcula SHA-256 e registra backup com timestamp e versão. O estado só é publicado depois da cópia, das migrations transacionais e da conferência de schema, índices, contagens e registros críticos.

Se o destino 1.24.x já existir, a origem legada só o substitui quando o destino for comprovadamente o bootstrap automático não utilizado: um único administrador provisório sem login, somente empresa e funcionário demonstrativos e nenhuma movimentação operacional. Os dois bancos são copiados para `backups/` antes da troca. Um destino utilizado tem precedência e não é sobrescrito.

O marcador `legacy-database-migration-v1.json` armazena versão, origem, checksum e conclusão. Reexecuções com o mesmo estado são no-op. Banco corrompido, erro de cópia, falta de espaço/permissão ou falha de migration abortam antes da promoção; o rollback restaura o destino anterior ou remove o destino incompleto.

O arquivo antigo continua utilizável durante todo o processo. Backups não devem ser apagados antes da homologação e da política de retenção. Migrations destrutivas futuras exigem nova conversão reversa ou manutenção do backup; não se deve alegar reversibilidade apenas porque uma transação existe.

## Compatibilidade de credenciais

Usuários e hashes de senha existentes nas versões `1.23.4` e anteriores são dados operacionais e não são redefinidos durante a atualização. O arquivo bootstrap eventualmente criado pelo destino vazio é removido após a migração, evitando indicar uma senha que não pertence ao banco migrado. O bootstrap aleatório é criado somente em uma instalação realmente nova, sem usuários preexistentes.

As versões `1.24.0` a `1.24.2` podiam substituir a credencial histórica do administrador durante a primeira migração. Quando o marcador comprova que o banco veio da instalação legada, o administrador ainda está com a senha provisória gerada e não houve login bem-sucedido, a inicialização restaura exclusivamente o hash original a partir do banco legado preservado. O processo cria backup do destino antes do reparo, não altera o arquivo antigo, é idempotente e não sobrescreve uma senha já utilizada ou alterada após a migração.
