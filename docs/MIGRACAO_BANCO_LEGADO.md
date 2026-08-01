# Migração do banco legado

`legacy_data.rs` executa antes das migrations normais. Ele encontra o banco antigo sem alterar seu conteúdo, inclusive quando a instalação anterior utilizava `data_dir_override` no `bootstrap.json`, faz `PRAGMA integrity_check`, calcula SHA-256 e registra backup com timestamp e versão. O destino só é promovido depois das migrations transacionais e da conferência de schema, índices, contagens e registros críticos.

O marcador `legacy-database-migration-v1.json` armazena versão, origem, checksum e conclusão. Reexecuções com o mesmo estado são no-op. Banco corrompido, erro de cópia, falta de espaço/permissão ou falha de migration abortam antes da promoção; o rollback restaura o destino anterior ou remove o destino incompleto.

O arquivo antigo continua utilizável durante todo o processo. Backups não devem ser apagados antes da homologação e da política de retenção. Migrations destrutivas futuras exigem nova conversão reversa ou manutenção do backup; não se deve alegar reversibilidade apenas porque uma transação existe.

## Compatibilidade de credenciais

Usuários e hashes de senha existentes nas versões `1.23.4` e anteriores são dados operacionais e não são redefinidos durante a atualização. O bootstrap aleatório é criado somente em uma instalação realmente nova, sem usuários preexistentes.

As versões `1.24.0` a `1.24.2` podiam substituir a credencial histórica do administrador durante a primeira migração. Quando o marcador comprova que o banco veio da instalação legada, o administrador ainda está com a senha provisória gerada e não houve login bem-sucedido, a inicialização restaura exclusivamente o hash original a partir do banco legado preservado. O processo cria backup do destino antes do reparo, não altera o arquivo antigo, é idempotente e não sobrescreve uma senha já utilizada ou alterada após a migração.
