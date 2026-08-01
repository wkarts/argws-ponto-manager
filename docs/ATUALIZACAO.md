# Guia de atualização

Antes da atualização, feche a aplicação e faça backup do diretório de dados. Na primeira execução da versão 1.23.4, o Ponto Manager procura o banco antigo, valida `PRAGMA integrity_check`, calcula SHA-256 e cria backup versionado sem alterar a origem.

O banco é copiado para `argws-ponto-manager/ponto-manager.db`, migrado em transação e validado por tabelas, índices, contagens e registros críticos. Um marcador com versão/checksum torna a operação idempotente. Em falha, o destino é restaurado/removido e o banco antigo permanece utilizável.

Após atualizar, valide login, empresa ativa, quantidades de usuários/funcionários/batidas, AFD, apuração, banco de horas, fechamento e relatórios. Não remova o diretório antigo até concluir a homologação e a janela de retenção do backup.
