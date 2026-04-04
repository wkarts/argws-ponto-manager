# Implantação mercado — Fase 3

## Entregas desta fase

- painel smart no cartão de ponto
- sugestões automáticas para:
  - esquecimento de batida
  - falta sem marcação
  - troca de folga detectada
  - meia folga / jornada parcial
- aplicação em lote das sugestões automáticas do tipo seguro
- exclusão assistida de batidas duplicadas
- refinamento visual do cartão com blocos operacionais mais compactos
- ajustes globais de grid de filtros para reduzir colunas excessivamente verticais

## Escopo técnico

### Backend
- `smart_suggestion_list`
- `smart_apply_suggestions`
- `batida_duplicate_candidates`
- `batida_delete_batch`

### Frontend
- `CartaoPontoPage.vue` reorganizada com dois blocos operacionais laterais:
  - motor smart
  - exclusão assistida

## Observações

- a heurística de meia folga e troca de folga continua evolutiva; nesta fase ela prioriza segurança e explicabilidade
- a aplicação automática em lote foi restrita aos cenários mais seguros para não provocar regressão em produção
- a revisão visual foi feita sem reescrever a estrutura principal da tela
