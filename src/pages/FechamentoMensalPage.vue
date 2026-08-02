<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import AppPageTitleBar from "../components/base/AppPageTitleBar.vue";
import BaseFilterBar from "../components/base/BaseFilterBar.vue";
import { gerarFechamentoRelatorio, listEmployees, listFechamentos, type ComboOption } from "../services/crud";
import { formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const funcionarioOptions = ref<ComboOption[]>([]);
const rows = ref<Record<string, unknown>[]>([]);
const message = ref("");
const error = ref("");
const loading = ref(false);

const form = reactive({
  funcionarioId: "",
  ano: new Date().getFullYear(),
  mes: new Date().getMonth() + 1
});

async function loadCombos() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
  funcionarioOptions.value = rows.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
  if (!form.funcionarioId && funcionarioOptions.value.length) form.funcionarioId = String(funcionarioOptions.value[0].id);
}

async function load() {
  rows.value = await listFechamentos({
    funcionarioId: form.funcionarioId || null,
    ano: form.ano,
    mes: form.mes
  });
}

async function gerar() {
  message.value = "";
  error.value = "";
  loading.value = true;
  try {
    const result = await gerarFechamentoRelatorio({
      funcionarioId: form.funcionarioId || null,
      ano: form.ano,
      mes: form.mes
    });
    message.value = `Relatório de fechamento gerado em: ${String(result.relatorio_path || '')}`;
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao gerar fechamento.";
  } finally {
    loading.value = false;
  }
}

async function clearFilters() {
  const today = new Date();
  form.funcionarioId = "";
  form.ano = today.getFullYear();
  form.mes = today.getMonth() + 1;
  await load();
}

watch(() => session.activeCompanyId, async () => { await loadCombos(); await load(); });

onMounted(async () => {
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <AppPageTitleBar title="Fechamento mensal" subtitle="Geração do espelho mensal por colaborador, com resumo e campos para assinatura do colaborador e do empregador." icon="calendarCheck" />

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <BaseFilterBar title="Parâmetros do fechamento" description="Selecione o colaborador e a competência que serão gerados." :loading="loading">
        <div class="field filter-field--wide">
          <label>Funcionário</label>
          <select v-model="form.funcionarioId">
            <option value="">Selecione</option>
            <option v-for="item in funcionarioOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>
        <div class="field filter-field--compact">
          <label>Ano</label>
          <input v-model="form.ano" type="number" min="2020" max="2100" />
        </div>
        <div class="field filter-field--compact">
          <label>Mês</label>
          <input v-model="form.mes" type="number" min="1" max="12" />
        </div>
      <template #actions>
        <button class="secondary" type="button" :disabled="loading" @click="clearFilters">Limpar filtros</button>
        <button class="primary" type="button" :disabled="loading" @click="gerar">
          {{ loading ? 'Gerando...' : 'Gerar fechamento' }}
        </button>
      </template>
    </BaseFilterBar>

    <div class="card">
      <h3 style="margin-top: 0;">Fechamentos gerados</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Funcionário</th>
              <th>Período</th>
              <th>Esperado</th>
              <th>Trabalhado</th>
              <th>Saldo</th>
              <th>Atraso</th>
              <th>Extra</th>
              <th>Banco horas</th>
              <th>Arquivo</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="String(row.id)">
              <td>{{ row.funcionario_nome }}</td>
              <td>{{ row.mes }}/{{ row.ano }}</td>
              <td>{{ formatMinutes(Number(row.total_esperado_minutos || 0)) }}</td>
              <td>{{ formatMinutes(Number(row.total_trabalhado_minutos || 0)) }}</td>
              <td>{{ formatMinutes(Number(row.total_saldo_minutos || 0)) }}</td>
              <td>{{ formatMinutes(Number(row.total_atraso_minutos || 0)) }}</td>
              <td>{{ formatMinutes(Number(row.total_extra_minutos || 0)) }}</td>
              <td>{{ formatMinutes(Number(row.total_banco_horas_minutos || 0)) }}</td>
              <td>{{ row.relatorio_path }}</td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="9" class="muted">Nenhum fechamento gerado para o filtro informado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
