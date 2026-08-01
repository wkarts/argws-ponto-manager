# Guia de instalação

Use o instalador do sistema operacional ou o pacote CloudPanel. Em instalação limpa, a primeira inicialização cria o banco, aplica migrations e gera `.bootstrap-admin.local` no diretório de dados. Consulte o arquivo localmente, entre como `admin` e troque a senha; o arquivo é removido.

Não distribua a credencial bootstrap nem copie o arquivo para outro host. API, webhook e WebSocket ficam em loopback e serviços opcionais ficam desativados até configuração explícita.

Para validar uma instalação: iniciar, trocar a senha, autenticar novamente, abrir dashboard, cadastrar empresa/funcionário, registrar batida, apurar um período, gerar relatório e reiniciar confirmando a sessão/dados.
