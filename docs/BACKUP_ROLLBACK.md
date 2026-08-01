# Backup e rollback

Antes do deploy, copie o diretório de dados e registre versão/checksum. Durante a atualização legada, o próprio aplicativo cria backup imutável antes de escrever no destino.

Rollback de aplicação:

1. pare desktop, serviço e worker;
2. preserve logs e o marcador de migração;
3. restaure o binário/pacote anterior;
4. aponte a versão anterior ao banco antigo intacto;
5. se o novo banco já recebeu dados após o corte, não o sobrescreva: exporte/reconcilie os dados antes de voltar;
6. valide integridade, login, contagens e apuração.

Rollback CloudPanel troca o symlink/diretório da aplicação para a versão anterior e mantém `data/` fora do pacote. Se uma migration futura remover ou transformar dados sem conversão reversa, o rollback permitido é restaurar o backup correspondente, aceitando a perda apenas das operações posteriores ao snapshot e após aprovação explícita.
