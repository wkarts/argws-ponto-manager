# Guia CloudPanel

O pacote CloudPanel contém o binário Rust headless, Web/PWA, scripts `start`, `stop`, `restart`, `status`, `check`, `cli`, `worker` e modelo público de ambiente.

1. gere o pacote x64/x86 pelo workflow ou script oficial;
2. confira o `.sha256` antes de extrair;
3. configure portas, diretório de dados e tokens fora do pacote;
4. mantenha host da API em loopback quando houver proxy reverso;
5. exponha somente HTTPS no proxy e limite CORS à origem real;
6. execute `check.sh`, depois `start.sh` e `status.sh`.

Persistir `data/` e `logs/` fora de releases. O deploy deve trocar diretório de versão por symlink ou mecanismo equivalente, mantendo a versão anterior disponível para rollback. CloudPanel não recebe credencial bootstrap, token ou certificado em artefato.
