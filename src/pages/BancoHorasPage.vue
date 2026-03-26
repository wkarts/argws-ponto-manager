<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import {
  comboJornadas,
  listEmployees,
  listBancoHoras,
  processBancoHoras,
  saveBancoHorasAjuste,
  type BancoHorasProcessResponse,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const rows = ref<GenericRecord[]>([]);
const employeeOptions = ref<ComboOption[]>([]);
const jornadaOptions = ref<ComboOption[]>([]);
const result = ref<BancoHorasProcessResponse | null>(null);
const error = ref("");
const loading = ref(false);
const processing = ref(false);
const saving = ref(false);

const filters = reactive({
  funcionarioId: "",
  dataInicial: "",
  dataFinal: ""
});

const ajuste = reactive({
  funcionarioId: "",
  jornadaId: "",
  dataReferencia: "",
  minutos: 0,
  observacao: ""
});

async function loadCombos() {
  const [employees, jornadas] = await Promise.all([listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true }), comboJornadas()]);
  employeeOptions.value = employees.map((item) => ({ id: Number(item.id), label: String(item.nome || item.label || item.id) }));
  jornadaOptions.value = jornadas;
  if (!filters.funcionarioId && employeeOptions.value.length) { filters.funcionarioId = String(employeeOptions.value[0].id); }
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listBancoHoras({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: filters.funcionarioId ? Number(filters.funcionarioId) : null,
      dataInicial: filters.dataInicial || null,
      dataFinal: filters.dataFinal || null
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao consultar banco de horas.";
  } finally {
    loading.value = false;
  }
}

async function processar() {
  processing.value = true;
  error.value = "";
  try {
    result.value = await processBancoHoras({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: filters.funcionarioId ? Number(filters.funcionarioId) : null,
      dataInicial: filters.dataInicial,
      dataFinal: filters.dataFinal,
      overwrite: true
    });
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao processar banco de horas.";
  } finally {
    processing.value = false;
  }
}

async function salvarAjuste() {
  saving.value = true;
  error.value = "";
  try {
    await saveBancoHorasAjuste({
      funcionarioId: Number(ajuste.funcionarioId),
      jornadaId: ajuste.jornadaId ? Number(ajuste.jornadaId) : null,
      dataReferencia: ajuste.dataReferencia,
      minutos: Number(ajuste.minutos),
      observacao: ajuste.observacao
    });
    await load();
    ajuste.dataReferencia = "";
    ajuste.minutos = 0;
    ajuste.observacao = "";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar ajuste.";
  } finally {
    saving.value = false;
  }
}

watch(() => session.activeCompanyId, async () => { await loadCombos(); await load(); });

onMounted(async () => {
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Banco de horas</h2>
        <div class="muted-text">Processamento automático da apuração e ajustes manuais controlados.</div>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="section-title">Processar saldo por período</div>
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="filters.funcionarioId">
            <option value="">Todos</option>
            <option v-for="item in employeeOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
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
          <button class="primary" :disabled="processing" @click="processar">{{ processing ? 'Processando...' : 'Processar período' }}</button>
        </div>
      </div>

      <div v-if="result" class="kpis">
        <div class="kpi"><strong>Dias processados</strong><span>{{ result.dias_processados }}</span></div>
        <div class="kpi"><strong>Créditos</strong><span>{{ formatMinutes(result.total_creditos_minutos) }}</span></div>
        <div class="kpi"><strong>Débitos</strong><span>{{ formatMinutes(result.total_debitos_minutos) }}</span></div>
        <div class="kpi"><strong>Saldo líquido</strong><span>{{ formatMinutes(result.saldo_liquido_minutos) }}</span></div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Ajuste manual</div>
      <div class="grid columns-5 mobile-columns-1">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="ajuste.funcionarioId">
            <option value="">Selecione</option>
            <option v-for="item in employeeOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Jornada</label>
          <select v-model="ajuste.jornadaId">
            <option value="">Selecione</option>
            <option v-for="item in jornadaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Data</label>
          <input v-model="ajuste.dataReferencia" type="date" />
        </div>
        <div class="field">
          <label>Minutos (+/-)</label>
          <input v-model="ajuste.minutos" type="number" />
        </div>
        <div class="field">
          <label>Observação</label>
          <input v-model="ajuste.observacao" type="text" placeholder="Motivo do ajuste" />
        </div>
      </div>
      <div class="actions">
        <button class="secondary" :disabled="saving" @click="salvarAjuste">{{ saving ? 'Salvando...' : 'Salvar ajuste' }}</button>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3>Lançamentos</h3>
          <div class="muted-text">Histórico consolidado do banco de horas.</div>
        </div>
        <div class="actions">
          <button class="secondary" :disabled="loading" @click="load">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
        </div>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Data</th>
              <th>Funcionário</th>
              <th>Jornada</th>
              <th>Minutos</th>
              <th>Categoria</th>
              <th>Classificação</th>
              <th>Origem</th>
              <th>Observação</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="Number(row.id)">
              <td>{{ row.data_referencia }}</td>
              <td>{{ row.funcionario_nome }}</td>
              <td>{{ row.jornada_nome || '-' }}</td>
              <td>{{ formatMinutes(Number(row.minutos || 0)) }}</td>
              <td>{{ row.categoria }}</td>
              <td>{{ row.classificacao }}</td>
              <td>{{ row.origem }}</td>
              <td>{{ row.observacao || '-' }}</td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="8" class="empty-cell">Nenhum lançamento encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
