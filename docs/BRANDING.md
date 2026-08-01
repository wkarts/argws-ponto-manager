# Branding do Ponto Manager

A identidade final usa o logotipo oficial encontrado no repositório de origem, publisher ARGWS e as cores `#2563EB` (primária) e `#14B8A6` (secundária). `src/assets/branding/brand.json` é a referência legível por ferramentas.

Os assets foram regenerados para PWA, favicon, splash, tray, ICO, ICNS e tamanhos Tauri. Nome, identificador, versão e publisher devem permanecer sincronizados em `package.json`, `Cargo.toml`, `tauri.conf.json`, `VERSION`, manifest PWA, frontend, instaladores e release.

Qualquer alteração futura deve atualizar todo o conjunto, executar `verify_identity.py` e fazer conferência visual de transparência, legibilidade em fundo claro/escuro e resolução dos ícones. Não reutilize imagens de produtos derivados ou da base corporativa.
