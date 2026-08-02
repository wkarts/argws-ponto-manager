# Cartão de ponto, dashboard e integridade AFD

## Objetivo

Esta evolução mantém a arquitetura `1.24.x`, recupera a ergonomia operacional da série `1.23.x` e torna a origem oficial das marcações auditável de ponta a ponta.

## Regras de integridade

- Marcações cuja origem contenha `afd`, `rep` ou `conector` são oficiais e imutáveis.
- Uma marcação oficial não pode ser editada nem excluída fisicamente.
- A primeira marcação oficial de um funcionário na mesma data e hora é a principal.
- Uma repetição oficial exata é descartada durante a importação/coleta e permanece registrada no histórico AFD ou no log do Connector.
- Se uma marcação manual existir no mesmo instante, a oficial passa a ser a principal e a manual recebe o estado `duplicidade`.
- Tratar duplicidade altera `ativo` para `0`; a linha continua na tabela `batidas`, vinculada por `duplicada_de_id`.
- Reativar restaura `ativo = 1`, volta a incluir a batida nos cálculos e grava auditoria/sincronização.
- Relatórios, apuração, banco de horas e cartão consideram somente batidas ativas.
- A proximidade de até um minuto é apresentada como candidata à conferência, não ocultada automaticamente.

## Migração de banco

A migration acrescenta em `batidas`:

- `ativo`
- `status`
- `duplicada_de_id`
- `inativada_em`
- `inativada_motivo`
- `reativada_em`

Duplicidades oficiais históricas com funcionário, data e hora idênticos são consolidadas preservando o menor ID oficial como principal. Não há remoção de dados.

## Interface

- A sidebar volta às áreas Início, Cadastros, Operação, Relatórios e integração, Documentação e Sistema.
- Cada recurso possui ícone semântico exclusivo, validado no CI.
- A dashboard exibe dados reais de funcionários, batidas, AFD, Connector, inconsistências e sincronização.
- O cartão recupera a grade horizontal, edição inline, painel lateral com Marcações/Ocorrências/Smart/Exclusão e prévia acima da operação.
- A prévia e a impressão usam a mesma função geradora de HTML.
- A guia Exclusão mostra candidatas, registros ocultados e a ação de reativação.

## Connector

O contrato HTTP, autenticação, timeout, NSR incremental e logs do Ponto Manager Connector não foram alterados. A coleta passou a compartilhar a mesma regra de idempotência e prioridade oficial da importação AFD.

## Rollback

A reversão do código não exige apagar colunas nem restaurar banco. As novas colunas podem permanecer sem afetar versões anteriores. Antes de qualquer rollback operacional, preserve o SQLite e o diretório `backups` da instalação.
