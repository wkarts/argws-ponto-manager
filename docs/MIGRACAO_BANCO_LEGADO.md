# Migração do banco legado

`legacy_data.rs` executa antes das migrations normais. Ele encontra o banco antigo sem alterar seu conteúdo, faz `PRAGMA integrity_check`, calcula SHA-256 e registra backup com timestamp e versão. O destino só é promovido depois das migrations transacionais e da conferência de schema, índices, contagens e registros críticos.

O marcador `legacy-migration.json` armazena versão, origem, checksum e conclusão. Reexecuções com o mesmo estado são no-op. Banco corrompido, erro de cópia, falta de espaço/permissão ou falha de migration abortam antes da promoção; o rollback restaura o destino anterior ou remove o destino incompleto.

O arquivo antigo continua utilizável durante todo o processo. Backups não devem ser apagados antes da homologação e da política de retenção. Migrations destrutivas futuras exigem nova conversão reversa ou manutenção do backup; não se deve alegar reversibilidade apenas porque uma transação existe.
