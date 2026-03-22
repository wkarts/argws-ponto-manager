<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getBootstrap, listSyncQueue, type SyncQueueItem } from "../services/crud";
import { formatMinutes } from "../services/format";

const stats = ref<Record<string, unknown>>({});
const queue = ref<SyncQueueItem[]>([]);

onMounted(async () => {
  stats.value = await getBootstrap();
  queue.value = await listSyncQueue();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Dashboard</h2>
        <div class="muted">Visão geral do sistema local.</div>
      </div>
    </div>

    <div class="kpis">
      <div class="kpi">
        <strong>Empresas</strong>
        <span>{{ stats.empresas || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>Funcionários</strong>
        <span>{{ stats.funcionarios || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>Batidas</strong>
        <span>{{ stats.batidas || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>Jornadas</strong>
        <span>{{ stats.jornadas || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>AFD importados</strong>
        <span>{{ stats.afd_importacoes || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>Banco de horas</strong>
        <span>{{ stats.banco_horas || 0 }}</span>
      </div>
      <div class="kpi">
        <strong>Sync pendente</strong>
        <span>{{ stats.sync_pendente || 0 }}</span>
      </div>
    </div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card">
        <h3 style="margin-top: 0;">Resumo operacional</h3>
        <ul class="summary-list">
          <li><strong>Banco:</strong> {{ stats.db_path }}</li>
          <li><strong>Usuários:</strong> {{ stats.usuarios || 0 }}</li>
          <li><strong>Equipamentos:</strong> {{ stats.equipamentos || 0 }}</li>
          <li><strong>Horários:</strong> {{ stats.horarios || 0 }}</li>
          <li><strong>Última carga diária padrão:</strong> {{ formatMinutes(Number(stats.carga_padrao_minutos || 0)) }}</li>
        </ul>
      </div>

      <div class="card">
        <h3 style="margin-top: 0;">Fila de sincronização</h3>
        <div v-if="queue.length === 0" class="muted">Nenhum item pendente.</div>
        <div v-else class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Entidade</th>
                <th>Ação</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in queue.slice(0, 8)" :key="item.id">
                <td>{{ item.id }}</td>
                <td>{{ item.entity_name }}</td>
                <td>{{ item.action_name }}</td>
                <td>{{ item.status }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
