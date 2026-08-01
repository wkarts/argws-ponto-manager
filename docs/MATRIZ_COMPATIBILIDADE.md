# Matriz de compatibilidade

| Capacidade | Desktop | Web/PWA | Headless/API | CLI/worker | Serviço | CloudPanel |
| --- | --- | --- | --- | --- | --- | --- |
| Autenticação/domínio | completa | via API interna | completa | operacional | completa | completa |
| SQLite | local | via backend | local | local | local | local |
| MySQL/PostgreSQL | feature opcional | via backend | feature opcional | feature opcional | feature opcional | feature opcional |
| Importação/exportação | filesystem nativo | upload/download controlado | API autenticada | paths sanitizados | diretório configurado | diretório configurado |
| Impressão | janela nativa HTML/A4 | impressão do navegador | gera HTML/arquivo | gera arquivo | gera arquivo | gera arquivo |
| Tray | completo | indisponível | não aplicável | não aplicável | não aplicável | não aplicável |
| Webhook/WebSocket | opt-in | cliente autenticado | servidor opt-in | controle | opt-in | proxy restrito |
| Sincronização | fila/auditoria | fila via API | fila/auditoria | worker observa fila | worker | worker |

Targets de distribuição: Windows x64/x86, Linux desktop x64/ARM64, macOS Intel/Apple Silicon, Web/PWA e CloudPanel Linux x64/x86.

Limitações comprovadas: Web não acessa filesystem/tray diretamente; impressão usa recursos do navegador; o ambiente desta migração não possui GTK/WebKit para link/smoke Linux nem runners nativos Windows/macOS. Esses gates permanecem obrigatórios na PR draft.
