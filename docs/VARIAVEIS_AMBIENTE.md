# Variáveis de ambiente

O prefixo oficial é `ARGWS_PONTO_MANAGER_`; `.env.example` e `.env.model` são as fontes completas.

Grupos principais:

- `API_*`: host/porta, token obrigatório, header, CORS e exposição pública;
- `WEB_*`: servidor Web/PWA local;
- `WEBHOOK_*` e `WEBSOCKET_*`: serviços opt-in, tokens e bind;
- `DATABASE_*`: provider SQLite/MySQL/PostgreSQL e conexões;
- `LOCAL_DATA_DIR`: diretório de dados;
- `TRAY_*`, `START_WITH_WINDOWS` e `SERVICES_AUTO_START`: runtime desktop/serviço;
- `LEGACY_DATABASE_PATH`: override controlado para migração/homologação.

Segredos devem existir somente no ambiente/secret store. Bind público exige `API_ALLOW_PUBLIC_NETWORK=true`, token não vazio e CORS com origens explícitas. Nunca use curingas de CORS em produção.
