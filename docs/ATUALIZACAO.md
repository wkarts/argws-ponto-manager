# Guia de atualização

Antes da atualização, feche a aplicação e faça backup do diretório de dados. Na primeira execução da versão atual, o Ponto Manager procura `%LOCALAPPDATA%/pontos_desktop_tauri/pontos.db`, valida a origem com `PRAGMA integrity_check`, calcula SHA-256 e cria backup versionado antes de executar migrations.

O banco anterior é copiado para o contrato atual `%LOCALAPPDATA%/argws-ponto-manager/ponto-manager.db`, migrado em transação e validado por tabelas, índices, contagens e registros críticos. A origem permanece intacta. Se o destino atual já existir apenas com o bootstrap automático e nunca tiver sido utilizado, ele também recebe backup e é substituído pelo banco legado. Um destino com login ou dados reais nunca é sobrescrito automaticamente. Em falha, o destino anterior é restaurado e a origem continua utilizável.

Usuários e hashes de senha do banco anterior são preservados. Após uma migração, entre com o mesmo login e a mesma senha usados na versão anterior. Somente uma instalação realmente nova gera a credencial aleatória em `%LOCALAPPDATA%/argws-ponto-manager/.bootstrap-admin.local`.

Após atualizar, valide login, empresa ativa, quantidades de usuários/funcionários/batidas, AFD, apuração, banco de horas, fechamento e relatórios. Não remova o diretório antigo até concluir a homologação e a janela de retenção do backup.
