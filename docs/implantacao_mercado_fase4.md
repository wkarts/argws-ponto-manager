# Fase 4 — Cartão de ponto operacional estilo VB6

## Entregas principais
- grade diária do cartão com foco operacional
- seleção rápida do dia direto na grade
- ações rápidas de nova batida e nova ocorrência por dia
- motor smart local para sugestões automáticas
- aplicação de sugestões selecionadas ou seguras
- tratamento automático em lote para todos os colaboradores da visão atual
- exclusão assistida de batidas duplicadas ou muito próximas
- reorganização visual dos filtros e painéis para reduzir rolagem vertical

## Observações
- a heurística smart desta fase foi implementada de forma conservadora no frontend, usando o resumo de apuração já calculado
- a exclusão assistida usa as batidas carregadas no filtro atual
- a grade foi reorganizada para se aproximar mais do fluxo operacional do VB6 sem reescrever abruptamente o restante da aplicação
