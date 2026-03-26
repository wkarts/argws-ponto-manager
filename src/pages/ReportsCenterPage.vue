<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { apurarPeriodo, gerarFechamentoRelatorio, listEmployees, type ApuracaoResumo } from "../services/crud";
import { formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const employees = ref<{ id: number; nome: string }[]>([]);
const reportType = ref("apuracao");
const funcionarioId = ref<number | null>(null);
const dataInicial = ref(new Date().toISOString().slice(0, 10));
const dataFinal = ref(new Date().toISOString().slice(0, 10));
const ano = ref(new Date().getFullYear());
const mes = ref(new Date().getMonth() + 1);
const previewHtml = ref("");
const message = ref("");
const error = ref("");

const previewSrc = computed(() => (previewHtml.value ? `data:text/html;charset=utf-8,${encodeURIComponent(previewHtml.value)}` : ""));

async function loadEmployeesForCompany() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
  employees.value = rows.map((item) => ({ id: Number(item.id), nome: String(item.nome || item.id) }));
  if (!funcionarioId.value && employees.value.length) funcionarioId.value = employees.value[0].id;
}

function buildApuracaoHtml(result: ApuracaoResumo) {
  const rows = result.rows.map((row) => `
    <tr>
      <td>${row.funcionario_nome}</td>
      <td>${row.data}</td>
      <td>${row.jornada_nome}</td>
      <td>${(row.batidas || []).join(' | ') || '-'}</td>
      <td>${formatMinutes(row.horario_esperado_minutos)}</td>
      <td>${formatMinutes(row.trabalhado_minutos)}</td>
      <td>${formatMinutes(row.saldo_minutos)}</td>
      <td>${(row.mensagens || []).join(' | ') || '-'}</td>
    </tr>`).join('');
  return `<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><title>Apuração</title><style>body{font-family:Arial,sans-serif;margin:18px}table{width:100%;border-collapse:collapse;font-size:12px}th,td{border:1px solid #d1d5db;padding:6px 8px}th{background:#f3f4f6}.kpis{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin:16px 0}.box{border:1px solid #d1d5db;border-radius:8px;padding:10px}</style></head><body><h1>Apuração do período</h1><div class="kpis"><div class="box"><strong>Funcionários</strong><br>${result.total_funcionarios}</div><div class="box"><strong>Dias</strong><br>${result.total_dias}</div><div class="box"><strong>Saldo</strong><br>${formatMinutes(result.total_saldo_minutos)}</div><div class="box"><strong>Extras</strong><br>${formatMinutes(result.total_extra_minutos)}</div></div><table><thead><tr><th>Funcionário</th><th>Data</th><th>Jornada</th><th>Batidas</th><th>Previsto</th><th>Trabalhado</th><th>Saldo</th><th>Mensagens</th></tr></thead><tbody>${rows}</tbody></table></body></html>`;
}

function downloadCurrent() {
  if (!previewHtml.value) return;
  const blob = new Blob([previewHtml.value], { type: "text/html;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  const base = reportType.value === "apuracao" ? `apuracao_${dataInicial.value}_${dataFinal.value}` : `fechamento_${ano.value}_${mes.value}`;
  a.href = url;
  a.download = `${base}.html`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

function printCurrent() {
  if (!previewHtml.value) return;
  const win = window.open("", "_blank");
  if (!win) return;
  win.document.write(previewHtml.value);
  win.document.close();
  win.focus();
  setTimeout(() => win.print(), 300);
}

async function generate() {
  message.value = "";
  error.value = "";
  try {
    if (reportType.value === "apuracao") {
      const result = await apurarPeriodo({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioId.value,
        dataInicial: dataInicial.value,
        dataFinal: dataFinal.value,
      });
      previewHtml.value = buildApuracaoHtml(result);
      message.value = "Prévia de apuração gerada. Use Imprimir para selecionar impressora ou salvar como PDF.";
    } else {
      const result = await gerarFechamentoRelatorio({
        funcionarioId: funcionarioId.value,
        ano: ano.value,
        mes: mes.value,
      });
      const path = String(result.relatorio_path || "");
      previewHtml.value = `<!DOCTYPE html><html lang="pt-BR"><body style="font-family:Arial;padding:24px"><h1>Fechamento mensal gerado</h1><p>Arquivo salvo em:</p><code>${path}</code><p>Use o botão abaixo para abrir o arquivo no sistema.</p></body></html>`;
      if (path) window.open(`file://${path}`, "_blank");
      message.value = "Relatório de fechamento gerado. O arquivo HTML foi aberto e pode ser impresso ou salvo como PDF pelo diálogo do sistema.";
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao gerar relatório.";
  }
}

watch(() => session.activeCompanyId, loadEmployeesForCompany);

onMounted(loadEmployeesForCompany);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Central de relatórios</h2>
        <div class="muted-text">Prévia HTML unificada, com impressão direta, seleção de impressora pelo diálogo do sistema e possibilidade de salvar/exportar o relatório.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="downloadCurrent">Salvar / exportar</button>
        <button class="primary" @click="printCurrent">Imprimir / PDF</button>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Relatório</label>
          <select v-model="reportType">
            <option value="apuracao">Apuração do período</option>
            <option value="fechamento">Fechamento mensal</option>
          </select>
        </div>
        <div class="field">
          <label>Funcionário</label>
          <select v-model="funcionarioId">
            <option v-for="item in employees" :key="String(item.id)" :value="Number(item.id)">{{ item.nome }}</option>
          </select>
        </div>
        <template v-if="reportType === 'apuracao'">
          <div class="field"><label>Data inicial</label><input v-model="dataInicial" type="date" /></div>
          <div class="field"><label>Data final</label><input v-model="dataFinal" type="date" /></div>
        </template>
        <template v-else>
          <div class="field"><label>Ano</label><input v-model="ano" type="number" min="2020" max="2100" /></div>
          <div class="field"><label>Mês</label><input v-model="mes" type="number" min="1" max="12" /></div>
        </template>
      </div>
      <div class="actions">
        <button class="primary" @click="generate">Gerar prévia</button>
      </div>
    </div>

    <iframe v-if="previewSrc" class="report-frame" :src="previewSrc" title="Prévia do relatório" />
  </div>
</template>
