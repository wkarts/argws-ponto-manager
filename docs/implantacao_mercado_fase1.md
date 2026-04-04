# Implantação mercado — fase 1

## O que esta fase entrega

Esta fase foi aplicada de forma incremental para melhorar três pontos imediatos da implantação:

1. **Relatórios gerados**
   - download com fallback por `file_name` dentro de `exports/generated_reports`
   - atualização automática do `file_path` no banco quando o arquivo for localizado no novo ambiente

2. **Cartão de ponto**
   - filtro por **competência** além de intervalo
   - visão rápida da competência selecionada
   - destaque visual para dias com inconsistência e dias com ocorrência
   - ação para **imprimir todos os cartões da competência**

3. **Base para a próxima fase**
   - manutenção da estrutura atual, sem refatoração abrupta
   - preservação do cálculo diário existente
   - preparação do fluxo operacional para evoluir para motor semanal flexível

## Limites desta fase

Esta fase **não altera** o resolvedor central para reconhecer automaticamente:

- folga móvel semanal
- meia folga móvel
- diarista com variação por semana
- compensação automática por troca de dia de folga
- fechamento inteligente de batidas esquecidas

Esses pontos continuam previstos para a próxima etapa.

## Comportamento atual mantido

- apuração diária segue usando o mecanismo atual da aplicação
- jornada base permanece como referência principal
- cartão de ponto continua usando os mesmos comandos e persistência já existentes
- geração de relatórios continua registrando em `relatorios_gerados`

## Próxima fase recomendada

1. criar camada de **regra semanal flexível**
2. criar **motor de sugestão automática** de tratamento
3. aproximar mais a operação do cartão de ponto ao fluxo do VB6
4. permitir aplicação em lote por jornada, perfil e competência
