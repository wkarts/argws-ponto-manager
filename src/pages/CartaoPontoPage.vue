<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { comboList, deleteBatida, deleteOcorrencia, listBatidas, listEmployees, listOcorrencias, saveBatida, saveOcorrencia, type ComboOption, type GenericRecord } from "../services/crud";
import { logAppError, logAppInfo } from "../services/logger";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const savingBatida = ref(false);
const savingOcorrencia = ref(false);
const error = ref("");
const message = ref("");

const employeeOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const batidas = ref<GenericRecord[]>([]);
const ocorrencias = ref<GenericRecord[]>([]);
const reportHtml = ref("");

const filtros = reactive({
  funcionarioId: "",
  dataInicial: new Date().toISOString().slice(0, 10),
  dataFinal: new Date().toISOString().slice(0, 10),
});

const batidaForm = reactive({
  id: undefined as number | undefined,
  funcionario_id: "",
  data_referencia: new Date().toISOString().slice(0, 10),
  hora: "08:00",
  tipo: "entrada",
  equipamento_id: "",
  justificativa_id: "",
  observacao: "",
  manual_ajuste: true,
  validado: true,
  origem: "cartao_ponto",
  nsr: "",
});

const ocorrenciaForm = reactive({
  id: undefined as number | undefined,
  funcionario_id: "",
  data_referencia: new Date().toISOString().slice(0, 10),
  justificativa_id: "",
  tipo: "ajuste_manual",
  abonar_dia: false,
  minutos_abonados: 0,
  observacao: "",
});

const funcionarioIdNumero = computed<number | null>(() => {
  if (!filtros.funcionarioId) return null;
  const parsed = Number(filtros.funcionarioId);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
});
const funcionarioNomeSelecionado = computed(() => employeeOptions.value.find((item) => String(item.id) === filtros.funcionarioId)?.label || "Todos");

function resetBatida() {
  batidaForm.id = undefined;
  batidaForm.funcionario_id = filtros.funcionarioId || "";
  batidaForm.data_referencia = filtros.dataInicial;
  batidaForm.hora = "08:00";
  batidaForm.tipo = "entrada";
  batidaForm.equipamento_id = "";
  batidaForm.justificativa_id = "";
  batidaForm.observacao = "";
  batidaForm.manual_ajuste = true;
  batidaForm.validado = true;
  batidaForm.origem = "cartao_ponto";
  batidaForm.nsr = "";
}

function resetOcorrencia() {
  ocorrenciaForm.id = undefined;
  ocorrenciaForm.funcionario_id = filtros.funcionarioId || "";
  ocorrenciaForm.data_referencia = filtros.dataInicial;
  ocorrenciaForm.justificativa_id = "";
  ocorrenciaForm.tipo = "ajuste_manual";
  ocorrenciaForm.abonar_dia = false;
  ocorrenciaForm.minutos_abonados = 0;
  ocorrenciaForm.observacao = "";
}

async function carregarBase() {
  error.value = "";
  try {
    const [employees, justificativas] = await Promise.all([
      listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true }),
      comboList("justificativas"),
    ]);
    employeeOptions.value = employees.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
    justificativaOptions.value = justificativas;

    if (!filtros.funcionarioId && employeeOptions.value.length > 0) {
      filtros.funcionarioId = String(employeeOptions.value[0].id);
    }

    if (!batidaForm.funcionario_id) batidaForm.funcionario_id = filtros.funcionarioId;
    if (!ocorrenciaForm.funcionario_id) ocorrenciaForm.funcionario_id = filtros.funcionarioId;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar dados do cartão de ponto.";
    logAppError("cartao_ponto", "Falha ao carregar base de dados da tela.", { error: error.value });
  }
}

async function carregarCartao() {
  loading.value = true;
  error.value = "";
  message.value = "";
  try {
    const [rowsBatida, rowsOcorrencia] = await Promise.all([
      listBatidas({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioIdNumero.value,
        dataInicial: filtros.dataInicial || null,
        dataFinal: filtros.dataFinal || null,
      }),
      listOcorrencias({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioIdNumero.value,
        dataInicial: filtros.dataInicial || null,
        dataFinal: filtros.dataFinal || null,
      }),
    ]);
    batidas.value = rowsBatida;
    ocorrencias.value = rowsOcorrencia;
    reportHtml.value = buildCartaoHtml();
  } catch (err) {
    batidas.value = [];
    ocorrencias.value = [];
    error.value = err instanceof Error ? err.message : "Falha ao carregar o cartão de ponto.";
    logAppError("cartao_ponto", "Falha ao carregar visão operacional do cartão.", {
      error: error.value,
      filtros: { ...filtros, funcionarioId: funcionarioIdNumero.value, empresaId: session.activeCompanyId ?? null },
    });
  } finally {
    loading.value = false;
  }
}

function parseTimeToMinutes(value: string): number | null {
  if (!value || !value.includes(":")) return null;
  const [hh, mm] = value.split(":").map((item) => Number(item));
  if (!Number.isFinite(hh) || !Number.isFinite(mm)) return null;
  return hh * 60 + mm;
}

function minutesToHHMM(value: number): string {
  const safe = Math.max(0, value);
  const hh = Math.floor(safe / 60).toString().padStart(2, "0");
  const mm = Math.floor(safe % 60).toString().padStart(2, "0");
  return `${hh}:${mm}`;
}

function formatDate(value: Date): string {
  return value.toISOString().slice(0, 10);
}

function dayLabel(value: Date): string {
  const map = ["dom", "seg", "ter", "qua", "qui", "sex", "sáb"];
  return map[value.getDay()];
}

function buildCartaoHtml(): string {
  if (!filtros.dataInicial || !filtros.dataFinal) return "";
  const byDate = new Map<string, GenericRecord[]>();
  for (const row of batidas.value) {
    const date = String(row.data_referencia || "");
    if (!date) continue;
    if (!byDate.has(date)) byDate.set(date, []);
    byDate.get(date)!.push(row);
  }

  const initial = new Date(`${filtros.dataInicial}T00:00:00`);
  const final = new Date(`${filtros.dataFinal}T00:00:00`);
  if (Number.isNaN(initial.getTime()) || Number.isNaN(final.getTime()) || initial > final) return "";

  const targetDaily = 8 * 60;
  let totalNormal = 0;
  let totalFalta = 0;
  let totalExtra = 0;

  const rows: string[] = [];
  for (let cursor = new Date(initial); cursor <= final; cursor.setDate(cursor.getDate() + 1)) {
    const day = formatDate(cursor);
    const dayRows = (byDate.get(day) || []).slice().sort((a, b) => String(a.hora || "").localeCompare(String(b.hora || "")));
    const batidasHora = dayRows.map((item) => String(item.hora || "")).filter(Boolean);
    const dailyOcc = ocorrencias.value.filter((item) => String(item.data_referencia || "") === day);
    const isWeekend = cursor.getDay() === 0 || cursor.getDay() === 6;

    let worked = 0;
    for (let i = 0; i < batidasHora.length; i += 2) {
      const start = parseTimeToMinutes(batidasHora[i]);
      const end = parseTimeToMinutes(batidasHora[i + 1] || "");
      if (start != null && end != null && end > start) worked += end - start;
    }

    const hasDayAbono = dailyOcc.some((item) => Number(item.abonar_dia) === 1);
    const normal = Math.min(targetDaily, worked);
    const falta = hasDayAbono || isWeekend ? 0 : Math.max(0, targetDaily - worked);
    const extra = isWeekend ? worked : Math.max(0, worked - targetDaily);
    totalNormal += normal;
    totalFalta += falta;
    totalExtra += extra;

    const col = [...batidasHora, "", "", "", "", "", ""].slice(0, 6);
    rows.push(`
      <tr>
        <td>${day.split("-").reverse().join("/")} - ${dayLabel(cursor)}</td>
        <td>${col[0] || "Folga"}</td>
        <td>${col[1] || "Folga"}</td>
        <td>${col[2] || "Folga"}</td>
        <td>${col[3] || "Folga"}</td>
        <td>${col[4] || "Folga"}</td>
        <td>${col[5] || "Folga"}</td>
        <td>${minutesToHHMM(normal)}</td>
        <td>${minutesToHHMM(falta)}</td>
        <td>${minutesToHHMM(extra)}</td>
        <td>${dailyOcc.map((item) => String(item.tipo || "")).join(" | ") || "-"}</td>
      </tr>
    `);
  }

  return `<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><title>Cartão de ponto</title>
    <style>
      body{font-family:Consolas,monospace;margin:14px;color:#111}
      .head{display:grid;grid-template-columns:1fr auto;gap:8px;align-items:end;border-bottom:2px solid #333;padding-bottom:6px}
      h1{margin:0;font-size:24px}
      .meta{font-size:12px}
      table{width:100%;border-collapse:collapse;font-size:12px;margin-top:10px}
      th,td{border:1px solid #808080;padding:4px 6px;text-align:left}
      thead th{background:#ececec}
      .tot{font-weight:700;background:#f5f5f5}
      .sign{margin-top:32px;display:grid;grid-template-columns:1fr 1fr;gap:24px;text-align:center}
      .line{border-top:1px solid #333;padding-top:4px}
    </style></head>
    <body>
      <div class="head">
        <div>
          <h1>CARTÃO PONTO</h1>
          <div class="meta">Período: ${filtros.dataInicial.split("-").reverse().join("/")} até ${filtros.dataFinal.split("-").reverse().join("/")}</div>
          <div class="meta">Empresa: ${session.activeCompanyName || "-"}</div>
          <div class="meta">Colaborador: ${funcionarioNomeSelecionado.value}</div>
        </div>
        <div class="meta">Emitido em ${new Date().toLocaleDateString("pt-BR")}</div>
      </div>
      <table>
        <thead><tr><th>Dia</th><th>Ent.1</th><th>Saí.1</th><th>Ent.2</th><th>Saí.2</th><th>Ent.3</th><th>Saí.3</th><th>Normais</th><th>Faltas</th><th>Extras</th><th>Observações</th></tr></thead>
        <tbody>${rows.join("")}
          <tr class="tot"><td colspan="7">TOTAIS</td><td>${minutesToHHMM(totalNormal)}</td><td>${minutesToHHMM(totalFalta)}</td><td>${minutesToHHMM(totalExtra)}</td><td>-</td></tr>
        </tbody>
      </table>
      <div class="sign"><div class="line">${funcionarioNomeSelecionado.value}</div><div class="line">Responsável / RH</div></div>
    </body></html>`;
}

async function saveWithDialog(content: string, suggestedName: string, mimeType: string) {
  if (!content) throw new Error("Gere o cartão antes de exportar.");
  const picker = (window as unknown as { showSaveFilePicker?: Function }).showSaveFilePicker;
  if (picker) {
    const handle = await picker({
      suggestedName,
      types: [{ description: "Arquivo", accept: { [mimeType]: [`.${suggestedName.split(".").pop()}`] } }],
    });
    const writable = await handle.createWritable();
    await writable.write(content);
    await writable.close();
    return;
  }
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = suggestedName;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

async function exportarHtml() {
  try {
    reportHtml.value = buildCartaoHtml();
    await saveWithDialog(reportHtml.value, `cartao_ponto_${funcionarioNomeSelecionado.value.replace(/\\s+/g, "_")}_${filtros.dataInicial}_${filtros.dataFinal}.html`, "text/html");
    message.value = "Cartão exportado em HTML.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar HTML.";
  }
}

async function exportarExcel() {
  try {
    reportHtml.value = buildCartaoHtml();
    await saveWithDialog(reportHtml.value, `cartao_ponto_${funcionarioNomeSelecionado.value.replace(/\\s+/g, "_")}_${filtros.dataInicial}_${filtros.dataFinal}.xls`, "application/vnd.ms-excel");
    message.value = "Cartão exportado em Excel.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar Excel.";
  }
}

function imprimirOuSalvarPdf() {
  reportHtml.value = buildCartaoHtml();
  if (!reportHtml.value) {
    error.value = "Gere o cartão antes de imprimir/salvar PDF.";
    return;
  }
  const frame = document.createElement("iframe");
  frame.style.position = "fixed";
  frame.style.right = "0";
  frame.style.bottom = "0";
  frame.style.width = "0";
  frame.style.height = "0";
  frame.style.border = "0";
  document.body.appendChild(frame);

  const doc = frame.contentWindow?.document;
  if (!doc || !frame.contentWindow) {
    frame.remove();
    error.value = "Não foi possível inicializar o modo de impressão.";
    return;
  }

  doc.open();
  doc.write(reportHtml.value);
  doc.close();
  frame.contentWindow.focus();
  setTimeout(() => {
    try {
      frame.contentWindow?.print();
    } finally {
      setTimeout(() => frame.remove(), 1000);
    }
  }, 250);
}

async function salvarBatida() {
  savingBatida.value = true;
  error.value = "";
  message.value = "";
  try {
    if (!batidaForm.funcionario_id) throw new Error("Selecione o funcionário para lançar a batida.");
    await saveBatida({
      ...batidaForm,
      funcionario_id: Number(batidaForm.funcionario_id),
      equipamento_id: batidaForm.equipamento_id ? Number(batidaForm.equipamento_id) : null,
      justificativa_id: batidaForm.justificativa_id ? Number(batidaForm.justificativa_id) : null,
    });
    message.value = "Marcação salva com sucesso.";
    logAppInfo("cartao_ponto", "Marcação salva na área de cartão de ponto.");
    resetBatida();
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar marcação.";
    logAppError("cartao_ponto", "Falha ao salvar marcação manual.", { error: error.value, payload: { ...batidaForm } });
  } finally {
    savingBatida.value = false;
  }
}

async function salvarOcorrencia() {
  savingOcorrencia.value = true;
  error.value = "";
  message.value = "";
  try {
    if (!ocorrenciaForm.funcionario_id) throw new Error("Selecione o funcionário para registrar a ocorrência.");
    await saveOcorrencia({
      ...ocorrenciaForm,
      funcionario_id: Number(ocorrenciaForm.funcionario_id),
      justificativa_id: ocorrenciaForm.justificativa_id ? Number(ocorrenciaForm.justificativa_id) : null,
      minutos_abonados: Number(ocorrenciaForm.minutos_abonados) || 0,
    });
    message.value = "Ocorrência salva com sucesso.";
    logAppInfo("cartao_ponto", "Ocorrência salva na área de cartão de ponto.");
    resetOcorrencia();
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar ocorrência.";
    logAppError("cartao_ponto", "Falha ao salvar ocorrência.", { error: error.value, payload: { ...ocorrenciaForm } });
  } finally {
    savingOcorrencia.value = false;
  }
}

function editarBatida(row: GenericRecord) {
  batidaForm.id = Number(row.id);
  batidaForm.funcionario_id = String(row.funcionario_id || filtros.funcionarioId || "");
  batidaForm.data_referencia = String(row.data_referencia || filtros.dataInicial);
  batidaForm.hora = String(row.hora || "08:00");
  batidaForm.tipo = String(row.tipo || "entrada");
  batidaForm.equipamento_id = row.equipamento_id ? String(row.equipamento_id) : "";
  batidaForm.justificativa_id = row.justificativa_id ? String(row.justificativa_id) : "";
  batidaForm.observacao = String(row.observacao || "");
  batidaForm.manual_ajuste = Number(row.manual_ajuste) === 1 || row.manual_ajuste === true;
  batidaForm.validado = Number(row.validado) === 1 || row.validado === true;
  batidaForm.origem = String(row.origem || "cartao_ponto");
  batidaForm.nsr = String(row.nsr || "");
}

function editarOcorrencia(row: GenericRecord) {
  ocorrenciaForm.id = Number(row.id);
  ocorrenciaForm.funcionario_id = String(row.funcionario_id || filtros.funcionarioId || "");
  ocorrenciaForm.data_referencia = String(row.data_referencia || filtros.dataInicial);
  ocorrenciaForm.justificativa_id = row.justificativa_id ? String(row.justificativa_id) : "";
  ocorrenciaForm.tipo = String(row.tipo || "ajuste_manual");
  ocorrenciaForm.abonar_dia = Number(row.abonar_dia) === 1 || row.abonar_dia === true;
  ocorrenciaForm.minutos_abonados = Number(row.minutos_abonados || 0);
  ocorrenciaForm.observacao = String(row.observacao || "");
}

function addBatidaFromGrid(referenceDate?: string) {
  resetBatida();
  batidaForm.funcionario_id = filtros.funcionarioId || "";
  batidaForm.data_referencia = referenceDate || filtros.dataInicial || new Date().toISOString().slice(0, 10);
}

async function removerBatida(row: GenericRecord) {
  if (!row.id || !confirm("Remover esta batida?")) return;
  try {
    await deleteBatida(Number(row.id));
    message.value = "Batida removida com sucesso.";
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover batida.";
  }
}

async function moverBatida(row: GenericRecord, direction: -1 | 1) {
  const currentMinutes = parseTimeToMinutes(String(row.hora || ""));
  if (currentMinutes == null) return;
  const nextMinutes = Math.min(23 * 60 + 59, Math.max(0, currentMinutes + direction));
  try {
    await saveBatida({
      ...row,
      hora: minutesToHHMM(nextMinutes),
      funcionario_id: Number(row.funcionario_id),
      equipamento_id: row.equipamento_id ? Number(row.equipamento_id) : null,
      justificativa_id: row.justificativa_id ? Number(row.justificativa_id) : null,
    });
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao reorganizar batida.";
  }
}

async function removerOcorrencia(row: GenericRecord) {
  if (!row.id || !confirm("Remover esta ocorrência?")) return;
  try {
    await deleteOcorrencia(Number(row.id));
    message.value = "Ocorrência removida com sucesso.";
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover ocorrência.";
  }
}

watch(() => session.activeCompanyId, async () => {
  await carregarBase();
  await carregarCartao();
});

watch(() => filtros.funcionarioId, () => {
  if (!batidaForm.id) batidaForm.funcionario_id = filtros.funcionarioId;
  if (!ocorrenciaForm.id) ocorrenciaForm.funcionario_id = filtros.funcionarioId;
});

onMounted(async () => {
  await carregarBase();
  await carregarCartao();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Cartão de ponto</h2>
        <div class="muted-text">Visão operacional unificada para consultar, ajustar marcações e tratar ocorrências do colaborador.</div>
      </div>
      <div class="actions">
        <button class="secondary" :disabled="loading" @click="carregarCartao">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
        <button class="secondary" @click="exportarHtml">Exportar HTML</button>
        <button class="secondary" @click="exportarExcel">Exportar Excel</button>
        <button class="primary" @click="imprimirOuSalvarPdf">Imprimir / Salvar PDF</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-if="message" class="alert success">{{ message }}</div>

    <div class="card grid page-gap">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="filtros.funcionarioId">
            <option value="">Todos</option>
            <option v-for="item in employeeOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Data inicial</label>
          <input v-model="filtros.dataInicial" type="date" />
        </div>
        <div class="field">
          <label>Data final</label>
          <input v-model="filtros.dataFinal" type="date" />
        </div>
        <div class="actions align-end">
          <button class="primary" :disabled="loading" @click="carregarCartao">Aplicar filtros</button>
        </div>
      </div>
    </div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Lançamento/edição de marcação</div>
        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Data</label>
            <input v-model="batidaForm.data_referencia" type="date" />
          </div>
          <div class="field">
            <label>Hora</label>
            <input v-model="batidaForm.hora" type="time" />
          </div>
          <div class="field">
            <label>Tipo</label>
            <select v-model="batidaForm.tipo">
              <option value="entrada">Entrada</option>
              <option value="saida">Saída</option>
              <option value="intervalo_saida">Intervalo saída</option>
              <option value="intervalo_retorno">Intervalo retorno</option>
            </select>
          </div>
          <div class="field">
            <label>Justificativa</label>
            <select v-model="batidaForm.justificativa_id">
              <option value="">Sem justificativa</option>
              <option v-for="item in justificativaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
          <div class="field span-2">
            <label>Observação</label>
            <textarea v-model="batidaForm.observacao" rows="3" placeholder="Detalhes da marcação manual"></textarea>
          </div>
          <div class="field checkbox-line"><input v-model="batidaForm.manual_ajuste" class="checkbox-input" type="checkbox" /><label>Ajuste manual</label></div>
          <div class="field checkbox-line"><input v-model="batidaForm.validado" class="checkbox-input" type="checkbox" /><label>Validado</label></div>
        </div>
        <div class="actions">
          <button class="primary" :disabled="savingBatida" @click="salvarBatida">{{ savingBatida ? 'Salvando...' : batidaForm.id ? 'Atualizar marcação' : 'Salvar marcação' }}</button>
          <button class="secondary" @click="resetBatida">Limpar</button>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Justificativas e ocorrências</div>
        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Data</label>
            <input v-model="ocorrenciaForm.data_referencia" type="date" />
          </div>
          <div class="field">
            <label>Tipo de ocorrência</label>
            <select v-model="ocorrenciaForm.tipo">
              <option value="ajuste_manual">Ajuste manual</option>
              <option value="atestado">Atestado</option>
              <option value="falta_justificada">Falta justificada</option>
              <option value="abono">Abono</option>
            </select>
          </div>
          <div class="field">
            <label>Justificativa</label>
            <select v-model="ocorrenciaForm.justificativa_id">
              <option value="">Sem justificativa</option>
              <option v-for="item in justificativaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Minutos abonados</label>
            <input v-model.number="ocorrenciaForm.minutos_abonados" min="0" type="number" />
          </div>
          <div class="field span-2">
            <label>Observação</label>
            <textarea v-model="ocorrenciaForm.observacao" rows="3" placeholder="Detalhes da ocorrência"></textarea>
          </div>
          <div class="field checkbox-line"><input v-model="ocorrenciaForm.abonar_dia" class="checkbox-input" type="checkbox" /><label>Abonar dia</label></div>
        </div>
        <div class="actions">
          <button class="primary" :disabled="savingOcorrencia" @click="salvarOcorrencia">{{ savingOcorrencia ? 'Salvando...' : ocorrenciaForm.id ? 'Atualizar ocorrência' : 'Salvar ocorrência' }}</button>
          <button class="secondary" @click="resetOcorrencia">Limpar</button>
        </div>
      </div>
    </div>

    <div class="card table-wrap">
      <h3 style="margin-top: 0;">Marcações do cartão</h3>
      <table>
        <thead>
          <tr>
            <th>Data</th>
            <th>Hora</th>
            <th>Tipo</th>
            <th>Origem</th>
            <th>Justificativa</th>
            <th>Ação</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in batidas" :key="String(row.id)">
            <td>{{ row.data_referencia }}</td>
            <td>{{ row.hora }}</td>
            <td>{{ row.tipo }}</td>
            <td>{{ row.origem || '-' }}</td>
            <td>{{ row.justificativa_nome || '-' }}</td>
            <td>
              <div class="actions compact-actions">
                <button class="secondary" @click="editarBatida(row)">Editar</button>
                <button class="secondary" @click="addBatidaFromGrid(String(row.data_referencia || ''))">+ Batida</button>
                <button class="secondary" @click="moverBatida(row, -1)">↑</button>
                <button class="secondary" @click="moverBatida(row, 1)">↓</button>
                <button class="danger" @click="removerBatida(row)">Remover</button>
              </div>
            </td>
          </tr>
          <tr v-if="!batidas.length">
            <td colspan="6" class="empty-cell">Nenhuma marcação encontrada para o período.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="card table-wrap">
      <h3 style="margin-top: 0;">Ocorrências do cartão</h3>
      <table>
        <thead>
          <tr>
            <th>Data</th>
            <th>Tipo</th>
            <th>Justificativa</th>
            <th>Abono</th>
            <th>Observação</th>
            <th>Ação</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in ocorrencias" :key="String(row.id)">
            <td>{{ row.data_referencia }}</td>
            <td>{{ row.tipo }}</td>
            <td>{{ row.justificativa_nome || '-' }}</td>
            <td>{{ Number(row.minutos_abonados || 0) > 0 ? row.minutos_abonados : (Number(row.abonar_dia) === 1 ? 'Dia abonado' : '-') }}</td>
            <td>{{ row.observacao || '-' }}</td>
            <td>
              <div class="actions compact-actions">
                <button class="secondary" @click="editarOcorrencia(row)">Editar</button>
                <button class="danger" @click="removerOcorrencia(row)">Remover</button>
              </div>
            </td>
          </tr>
          <tr v-if="!ocorrencias.length">
            <td colspan="6" class="empty-cell">Nenhuma ocorrência encontrada para o período.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <iframe v-if="reportHtml" class="report-frame" :src="`data:text/html;charset=utf-8,${encodeURIComponent(reportHtml)}`" title="Prévia cartão de ponto" />
  </div>
</template>
