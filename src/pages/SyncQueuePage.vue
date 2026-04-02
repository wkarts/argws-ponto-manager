<script setup lang="ts">
import { onMounted, ref } from "vue";
import { listSyncQueue, markSyncQueueSynced, type SyncQueueItem } from "../services/crud";

const rows = ref<SyncQueueItem[]>([]);
const message = ref("");
const error = ref("");

async function load() {
  rows.value = await listSyncQueue();
}

async function markSynced(id: number) {
  try {
    await markSyncQueueSynced(id);
    await load();
    message.value = `Item ${id} marcado como sincronizado.`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao alterar item.";
  }
}

onMounted(async () => {
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Fila técnica de sincronização</h2>
        <div class="muted">Monitoramento local das alterações enfileiradas para integração posterior.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="info-note">
      <strong>Finalidade desta tela:</strong> ela exibe a fila local criada quando a aplicação grava alterações que podem precisar ser integradas depois com serviço externo, replicação ou rotina técnica futura.
      <ul class="sync-purpose-list">
        <li>o registro principal já foi salvo no banco local antes de entrar na fila;</li>
        <li>a fila não altera cálculos locais de jornada, apuração, banco de horas ou relatórios já gravados;</li>
        <li>o impacto de não sincronizar fica restrito a integrações externas e rastreabilidade técnica;</li>
        <li>esta tela é de uso operacional/técnico, não um passo obrigatório do usuário final no fluxo diário.</li>
      </ul>
    </div>

    <div class="card">
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Entidade</th>
              <th>Ação</th>
              <th>Registro</th>
              <th>Status</th>
              <th>Criado em</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="row.id">
              <td>{{ row.id }}</td>
              <td>{{ row.entity_name }}</td>
              <td>{{ row.action_name }}</td>
              <td>{{ row.record_id || '-' }}</td>
              <td>{{ row.status }}</td>
              <td>{{ row.created_at }}</td>
              <td>
                <button class="secondary" @click="markSynced(row.id)" :disabled="row.status === 'synced'">
                  Marcar como integrado
                </button>
              </td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="7" class="muted">Nenhum item na fila.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
