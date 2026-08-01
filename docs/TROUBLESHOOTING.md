# Solução de problemas

- **Bootstrap ausente:** confirme o diretório de dados e permissões. Se o usuário já trocou a senha, o arquivo é removido por projeto.
- **Banco antigo não detectado:** configure temporariamente `ARGWS_PONTO_MANAGER_LEGACY_DATABASE_PATH` para o arquivo correto e reinicie; não mova o original.
- **Integridade falhou:** não force a migration. Restaure uma cópia saudável ou use ferramentas SQLite sobre uma cópia.
- **API não inicia:** confira host, porta, token e conflitos. Bind público também exige autorização explícita.
- **PWA sem recurso nativo:** use o comportamento Web documentado ou execute desktop; não há simulação de tray/filesystem.
- **Build Linux falha em GTK/WebKit:** instale as dependências Tauri listadas no CI/guia de desenvolvimento.
- **Build Windows falha por caminho:** use `npm run build:windows:short-target`.
- **CloudPanel não sobe:** execute `check.sh`, valide checksum, permissões, portas, proxy e persistência de `data/`.
