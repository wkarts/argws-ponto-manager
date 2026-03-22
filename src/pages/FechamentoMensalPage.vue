<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { comboList, gerarFechamentoRelatorio, listFechamentos, type ComboOption } from "../services/crud";
import { formatMinutes } from "../services/format";

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
  funcionarioOptions.value = await comboList("funcionarios");
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

onMounted(async () => {
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Fechamento mensal / espelho para assinatura</h2>
        <div class="muted">Gera o relatório mensal por colaborador, com resumo do período e campos de assinatura do colaborador e do empregador.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="form.funcionarioId">
            <option value="">Selecione</option>
            <option v-for="item in funcionarioOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Ano</label>
          <input v-model="form.ano" type="number" min="2020" max="2100" />
        </div>
        <div class="field">
          <label>Mês</label>
          <input v-model="form.mes" type="number" min="1" max="12" />
        </div>
        <div class="actions align-end">
          <button class="primary" @click="gerar" :disabled="loading">
            {{ loading ? 'Gerando...' : 'Gerar fechamento' }}
          </button>
        </div>
      </div>
    </div>

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
