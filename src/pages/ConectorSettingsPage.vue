<script setup lang="ts">
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Parâmetros do Ponto Manager Conector</h2>
        <div class="muted-text">
          A configuração do conector é individual por REP/equipamento. Cada relógio pode apontar para uma API diferente e usar um token próprio.
        </div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div>
        <h3>Onde configurar</h3>
        <div class="muted-text">
          Acesse <strong>Cadastros &gt; Equipamentos</strong>, edite o REP desejado e preencha os campos abaixo.
        </div>
      </div>

      <div class="grid columns-2 mobile-columns-1">
        <div class="info-item">
          <strong>Usar Ponto Manager Conector</strong>
          <span>Ativa a comunicação via API para este relógio.</span>
        </div>
        <div class="info-item">
          <strong>ID do dispositivo no conector</strong>
          <span>ID/UUID do relógio cadastrado no Ponto Manager Conector.</span>
        </div>
        <div class="info-item">
          <strong>URL da API deste conector</strong>
          <span>Ex.: http://127.0.0.1:3000 ou URL remota específica deste REP.</span>
        </div>
        <div class="info-item">
          <strong>Token da API deste REP</strong>
          <span>Token/API Key individual usado somente por este relógio.</span>
        </div>
        <div class="info-item">
          <strong>Timeout do conector</strong>
          <span>Tempo máximo de espera da API para este REP.</span>
        </div>
        <div class="info-item">
          <strong>Último NSR importado</strong>
          <span>Usado na importação incremental. Se estiver zerado, o sistema solicita AFD completo.</span>
        </div>
      </div>
    </div>
  </div>
</template>
