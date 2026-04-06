<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import { apurarPeriodo, listEmployees, type ApuracaoResumo, type ComboOption } from "../services/crud";
import { formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const funcionarioOptions = ref<ComboOption[]>([]);
const result = ref<ApuracaoResumo | null>(null);
const error = ref("");
const loading = ref(false);

const filters = reactive({
  funcionarioId: "",
  dataInicial: "",
  dataFinal: ""
});

function selectedFuncionarioId(): number | null {
  if (!filters.funcionarioId) return null;
  const parsed = Number(filters.funcionarioId);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

async function loadCombos() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
  funcionarioOptions.value = rows.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
  if (!filters.funcionarioId && funcionarioOptions.value.length) filters.funcionarioId = String(funcionarioOptions.value[0].id);
}

async function processar() {
  error.value = "";
  loading.value = true;
  try {
    result.value = await apurarPeriodo({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: selectedFuncionarioId(),
      dataInicial: filters.dataInicial || null,
      dataFinal: filters.dataFinal || null
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao apurar período.";
  } finally {
    loading.value = false;
  }
}

watch(() => session.activeCompanyId, loadCombos);

onMounted(async () => {
  await loadCombos();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Apuração</h2>
        <div class="muted">Consolidação por período usando jornada semanal, tolerâncias, justificativas, atestados e ajustes manuais.</div>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="filters.funcionarioId">
            <option value="">Todos</option>
            <option v-for="item in funcionarioOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Data inicial</label>
          <input v-model="filters.dataInicial" type="date" />
        </div>
        <div class="field">
          <label>Data final</label>
          <input v-model="filters.dataFinal" type="date" />
        </div>
        <div class="actions align-end">
          <button class="primary" @click="processar" :disabled="loading">
            {{ loading ? "Processando..." : "Apurar" }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="result" class="grid page-gap">
      <div class="kpis">
        <div class="kpi">
          <strong>Funcionários</strong>
          <span>{{ result.total_funcionarios }}</span>
        </div>
        <div class="kpi">
          <strong>Dias</strong>
          <span>{{ result.total_dias }}</span>
        </div>
        <div class="kpi">
          <strong>Obrigatório (base)</strong>
          <span>{{ formatMinutes(result.total_esperado_minutos) }}</span>
          <small v-if="result.total_esperado_diario_minutos != null" class="muted">Previsto (soma diária): {{ formatMinutes(result.total_esperado_diario_minutos || 0) }}</small>
        </div>
        <div class="kpi">
          <strong>Trabalhado</strong>
          <span>{{ formatMinutes(result.total_trabalhado_minutos) }}</span>
        </div>
        <div class="kpi">
          <strong>Saldo</strong>
          <span>{{ formatMinutes(result.total_saldo_minutos) }}</span>
        </div>
        <div class="kpi">
          <strong>Atrasos</strong>
          <span>{{ formatMinutes(result.total_atraso_minutos) }}</span>
        </div>
        <div class="kpi">
          <strong>Extras</strong>
          <span>{{ formatMinutes(result.total_extra_minutos) }}</span>
        </div>
      </div>

      <div class="card">
        <h3 style="margin-top: 0;">Detalhamento</h3>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Funcionário</th>
                <th>Data</th>
                <th>Jornada</th>
                <th>Batidas</th>
                <th>Ocorrências</th>
                <th>Abono</th>
                <th>Esperado</th>
                <th>Computado</th>
                <th>Saldo</th>
                <th>Atraso</th>
                <th>Extra</th>
                <th>Mensagens</th>
                <th>Inconsistente</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in result.rows" :key="`${row.funcionario_id}-${row.data}`">
                <td>{{ row.funcionario_nome }}</td>
                <td>{{ row.data }}</td>
                <td>{{ row.jornada_nome }}</td>
                <td>{{ row.batidas.join(' | ') || '-' }}</td>
                <td>{{ row.ocorrencias.join(' | ') || '-' }}</td>
                <td>
                  <span v-if="row.abonado">Dia abonado</span>
                  <span v-else>{{ row.minutos_abonados > 0 ? formatMinutes(row.minutos_abonados) : '-' }}</span>
                </td>
                <td>{{ formatMinutes(row.horario_esperado_minutos) }}</td>
                <td>{{ formatMinutes(row.trabalhado_minutos) }}</td>
                <td>{{ formatMinutes(row.saldo_minutos) }}</td>
                <td>{{ formatMinutes(row.atraso_minutos) }}</td>
                <td>{{ formatMinutes(row.extra_minutos) }}</td>
                <td>{{ row.mensagens.join(' | ') || '-' }}</td>
                <td>{{ row.inconsistente ? 'Sim' : 'Não' }}</td>
              </tr>
              <tr v-if="result.rows.length === 0">
                <td colspan="13" class="muted">Nenhum registro encontrado para o período.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
