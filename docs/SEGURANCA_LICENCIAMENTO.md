# Segurança e licenciamento

A senha pública anterior foi removida. A instalação nova gera credencial bootstrap exclusiva com aleatoriedade criptográfica, salva apenas em `.bootstrap-admin.local` com modo 0600 quando suportado, ignorada pelo Git e removida após a troca obrigatória. Senhas usam Argon2 e política mínima de 12 caracteres com maiúscula, minúscula, número e símbolo.

SQL é parametrizado; operações críticas usam transação; caminhos e nomes de arquivo são normalizados; API, webhook e WebSocket usam loopback por padrão e recusam bind público sem autorização/token. A CSP restringe scripts, objetos, framing, origem base e conexões.

O módulo de licenciamento foi preservado isoladamente, mas:

- flags frontend e Rust estão `false`;
- card, menu e rota são inacessíveis;
- API informa `licensing: false`;
- trial, registro e verificações remotas não iniciam;
- nenhuma função é bloqueada por licença.

Ativação futura exige decisão explícita, revisão de privacidade/segurança e novos gates.
