# Documentação Técnica — Ponto Manager

> Documento restrito para desenvolvimento e manutenção do sistema.

## Objetivo
Concentrar no repositório (fora do menu da aplicação) as orientações técnicas de arquitetura, operação e troubleshooting.

## Escopo
- Arquitetura do frontend (Vue + Pinia + Router)
- Arquitetura do backend desktop (Tauri/Rust)
- Sessão/autenticação/autorização
- Operação e suporte
- Diagnóstico e rollback assistido

## Acesso
Este documento deve ser disponibilizado apenas para equipe técnica e responsáveis autorizados.

## Arquitetura (resumo)
- Frontend SPA em Vue 3
- Bridge Tauri para comandos de domínio
- Persistência local em SQLite
- Controle de acesso por permissões em rotas e componentes

## Operação segura
1. Validar versão/build antes de qualquer intervenção.
2. Reproduzir incidente em contexto isolado quando possível.
3. Coletar logs por módulo e período.
4. Aplicar correção incremental e reversível.
5. Registrar evidências de alteração e validação.

## Checklist de diagnóstico
1. Sessão ativa e permissões corretas.
2. Empresa ativa coerente com o cenário.
3. Base cadastral íntegra (funcionários, jornadas, eventos).
4. Logs sem erro crítico de comando/IO.
5. Teste de regressão mínimo após correção.

## Rollback operacional
- Reverter commit da alteração problemática.
- Restaurar base local a partir de backup consistente (se necessário).
- Reexecutar validações de login, cadastros, apuração e relatórios.
