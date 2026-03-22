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
        <h2 style="margin: 0;">Fila de sincronização</h2>
        <div class="muted">Controle local das alterações pendentes.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

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
                  Marcar sincronizado
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
