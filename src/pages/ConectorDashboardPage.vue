<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { formatDateTimeLocal } from "../services/format";
import {
  baixarAfdConector,
  coletarBatidasConector,
  dashboardConector,
  testarConector,
} from "../services/conectorService";

type DashboardPayload = {
  totais?: Record<string, unknown>;
  equipamentos?: any[];
  logs?: any[];
};

const loading = ref(false);
const actionLoading = ref<number | null>(null);
const error = ref("");
const message = ref("");
const dashboard = ref<DashboardPayload>({ totais: {}, equipamentos: [], logs: [] });
const filtros = ref({ somenteConector: true });

const totais = computed(() => dashboard.value.totais || {});
const equipamentos = computed(() => {
  const rows = dashboard.value.equipamentos || [];
  return filtros.value.somenteConector ? rows.filter((item) => item.usar_conector) : rows;
});
const logs = computed(() => dashboard.value.logs || []);

function formatDate(value?: string | null) {
  return formatDateTimeLocal(value || "");
}

function asNumber(value: unknown) {
  return Number(value || 0).toLocaleString("pt-BR");
}

async function carregar() {
  loading.value = true;
  error.value = "";
  try {
    dashboard.value = await dashboardConector() as DashboardPayload;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

async function testar(equipamentoId: number) {
  actionLoading.value = equipamentoId;
  error.value = "";
  message.value = "";
  try {
    const status = await testarConector(equipamentoId);
    message.value = `Conector do REP ${equipamentoId} respondeu com status ${status}.`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    actionLoading.value = null;
  }
}

async function coletarIncremental(equipamentoId: number) {
  actionLoading.value = equipamentoId;
  error.value = "";
  message.value = "";
  try {
    const result: any = await coletarBatidasConector({ equipamento_id: equipamentoId });
    message.value = `Coleta concluída. Importadas: ${result.total_importadas || 0}. Duplicadas: ${result.total_duplicadas || 0}. Último NSR: ${result.nsr_fim || "-"}.`;
    await carregar();
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    actionLoading.value = null;
  }
}

async function coletarCompleta(equipamentoId: number) {
  actionLoading.value = equipamentoId;
  error.value = "";
  message.value = "";
  try {
    const result: any = await coletarBatidasConector({ equipamento_id: equipamentoId, completo: true });
    message.value = `Coleta completa concluída. Importadas: ${result.total_importadas || 0}. Duplicadas: ${result.total_duplicadas || 0}.`;
    await carregar();
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    actionLoading.value = null;
  }
}

async function baixarAfd(equipamentoId: number) {
  actionLoading.value = equipamentoId;
  error.value = "";
  message.value = "";
  try {
    const result: any = await baixarAfdConector(equipamentoId);
    message.value = `AFD baixado e salvo em: ${result.arquivo_path || "arquivo local"}.`;
    await carregar();
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    actionLoading.value = null;
  }
}

onMounted(carregar);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Dashboard de coleta do Conector</h2>
        <div class="muted-text">Acompanhamento da integração Ponto Manager x Ponto Manager Conector, com NSR incremental, AFD e logs.</div>
      </div>
      <div class="actions">
        <button class="primary" :disabled="loading" @click="carregar">Atualizar</button>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-4 mobile-columns-1">
      <div class="card metric-card"><span>Equipamentos</span><strong>{{ asNumber(totais.equipamentos) }}</strong></div>
      <div class="card metric-card"><span>Usando conector</span><strong>{{ asNumber(totais.equipamentos_conector) }}</strong></div>
      <div class="card metric-card"><span>Batidas importadas</span><strong>{{ asNumber(totais.batidas_importadas) }}</strong></div>
      <div class="card metric-card"><span>AFDs baixados</span><strong>{{ asNumber(totais.afd_baixados) }}</strong></div>
    </div>

    <div class="grid columns-3 mobile-columns-1">
      <div class="card metric-card"><span>Duplicadas ignoradas</span><strong>{{ asNumber(totais.batidas_duplicadas) }}</strong></div>
      <div class="card metric-card"><span>Erros</span><strong>{{ asNumber(totais.erros) }}</strong></div>
      <div class="card metric-card"><span>Última coleta</span><strong>{{ formatDate(String(totais.ultima_coleta || "")) }}</strong></div>
    </div>

    <div class="card grid page-gap">
      <div class="toolbar compact-toolbar">
        <div>
          <h3>Equipamentos</h3>
          <div class="muted-text">Use a coleta incremental para continuar a partir do último NSR persistido no cadastro do REP.</div>
        </div>
        <label class="inline-check">
          <input v-model="filtros.somenteConector" type="checkbox" />
          Somente configurados no conector
        </label>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Descrição</th>
              <th>Modelo/IP</th>
              <th>Device ID</th>
              <th>API</th>
              <th>Últ. NSR</th>
              <th>Últ. coleta</th>
              <th>Import.</th>
              <th>Dup.</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in equipamentos" :key="item.id">
              <td>{{ item.id }}</td>
              <td>{{ item.descricao }}</td>
              <td>{{ item.modelo || '-' }}<br /><span class="muted-text">{{ item.ip || '-' }}:{{ item.porta || '-' }}</span></td>
              <td>{{ item.conector_device_id || '-' }}</td>
              <td>
                <span>{{ item.conector_base_url || '-' }}</span><br />
                <span class="muted-text">Token: {{ item.conector_token_configurado ? 'configurado' : 'não informado' }}</span>
              </td>
              <td>{{ item.conector_ultimo_nsr || 0 }}</td>
              <td>{{ formatDate(item.conector_ultima_coleta_em || item.ultima_execucao) }}</td>
              <td>{{ asNumber(item.total_importadas) }}</td>
              <td>{{ asNumber(item.total_duplicadas) }}</td>
              <td class="actions nowrap">
                <button class="secondary" :disabled="actionLoading === item.id || !item.usar_conector" @click="testar(item.id)">Testar</button>
                <button class="secondary" :disabled="actionLoading === item.id || !item.usar_conector" @click="coletarIncremental(item.id)">Incremental</button>
                <button class="secondary" :disabled="actionLoading === item.id || !item.usar_conector" @click="coletarCompleta(item.id)">Completa</button>
                <button class="secondary" :disabled="actionLoading === item.id || !item.usar_conector" @click="baixarAfd(item.id)">AFD</button>
              </td>
            </tr>
            <tr v-if="!equipamentos.length">
              <td colspan="10" class="muted-text text-center">Nenhum equipamento encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="card grid page-gap">
      <div>
        <h3>Últimas coletas</h3>
        <div class="muted-text">Histórico técnico das últimas 50 operações via conector.</div>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Data</th>
              <th>Equipamento</th>
              <th>Tipo</th>
              <th>Status</th>
              <th>Receb.</th>
              <th>Import.</th>
              <th>Dup.</th>
              <th>NSR</th>
              <th>Arquivo</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in logs" :key="log.id">
              <td>{{ formatDate(log.created_at) }}</td>
              <td>{{ log.equipamento }}</td>
              <td>{{ log.tipo }}</td>
              <td>{{ log.status }}</td>
              <td>{{ asNumber(log.total_recebidas) }}</td>
              <td>{{ asNumber(log.total_importadas) }}</td>
              <td>{{ asNumber(log.total_duplicadas) }}</td>
              <td>{{ log.nsr_inicio || '-' }} → {{ log.nsr_fim || '-' }}</td>
              <td class="small-path">{{ log.arquivo_path || '-' }}</td>
            </tr>
            <tr v-if="!logs.length">
              <td colspan="10" class="muted-text text-center">Nenhum log de coleta encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
