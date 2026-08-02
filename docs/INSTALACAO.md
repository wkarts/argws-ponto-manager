# Guia de instalação

Use o instalador do sistema operacional ou o pacote CloudPanel. Em instalação limpa no Windows, a primeira inicialização cria `%LOCALAPPDATA%\argws-ponto-manager\ponto-manager.db`, aplica migrations e gera `%LOCALAPPDATA%\argws-ponto-manager\.bootstrap-admin.local`. Consulte o arquivo localmente, entre como `admin` com a senha exclusiva nele registrada e troque a senha; o arquivo é removido.

Não distribua a credencial bootstrap nem copie o arquivo para outro host. API, webhook e WebSocket ficam em loopback e serviços opcionais ficam desativados até configuração explícita.

Para validar uma instalação: iniciar, trocar a senha, autenticar novamente, abrir dashboard, cadastrar empresa/funcionário, registrar batida, apurar um período, gerar relatório e reiniciar confirmando a sessão/dados.
