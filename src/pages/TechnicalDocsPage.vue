<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Documentação técnica</h2>
        <div class="muted-text">
          Referência para implantação, manutenção, segurança e diagnóstico do Ponto Manager.
        </div>
      </div>
    </div>

    <div class="alert info">
      Área restrita para perfis técnicos/administrativos. Compartilhe este conteúdo apenas com pessoal autorizado.
    </div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Arquitetura da solução</div>
        <ul class="summary-list">
          <li><strong>Frontend:</strong> Vue 3 + Vite + Pinia + Vue Router.</li>
          <li><strong>Desktop shell:</strong> Tauri (Rust) com comandos invocados via bridge.</li>
          <li><strong>Persistência local:</strong> banco SQLite no diretório configurado do sistema.</li>
          <li><strong>Domínios:</strong> autenticação, cadastros, batidas, apuração, relatórios e licenciamento.</li>
        </ul>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Sessão, autenticação e autorização</div>
        <ul class="summary-list">
          <li>Sessão baseada em token armazenado no <code>sessionStorage</code> do cliente.</li>
          <li>Restauração de sessão no bootstrap de navegação.</li>
          <li>Controle de acesso por permissões em rota e componentes.</li>
          <li>Usuário master com privilégios globais para operações administrativas.</li>
        </ul>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Operação e manutenção</div>
      <ul class="summary-list">
        <li><strong>Sistema e parâmetros:</strong> valida diretórios de dados, versão e build ativa.</li>
        <li><strong>Logs da aplicação:</strong> priorize análise por módulo e timestamp para troubleshooting.</li>
        <li><strong>Licenciamento:</strong> acompanhe validade, status e eventos de ativação.</li>
        <li><strong>Backup:</strong> mantenha política recorrente do diretório de dados e exportações.</li>
      </ul>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Checklist de diagnóstico rápido</div>
      <ol class="summary-list">
        <li>Confirmar versão/build e data/hora do host.</li>
        <li>Validar empresa ativa e permissões do usuário logado.</li>
        <li>Reproduzir cenário e coletar logs da aplicação.</li>
        <li>Revisar dados base: funcionários, jornadas, eventos e vínculos.</li>
        <li>Executar correção guiada e registrar evidências do atendimento.</li>
      </ol>
    </div>
  </div>
</template>
