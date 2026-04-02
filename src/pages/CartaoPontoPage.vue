<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import AppSwitch from "../components/AppSwitch.vue";
import {
  apurarPeriodo,
  comboList,
  deleteBatida,
  deleteOcorrencia,
  listBatidas,
  listCompanies,
  listEmployees,
  listOcorrencias,
  registerGeneratedReport,
  saveBatida,
  saveOcorrencia,
  type ApuracaoDia,
  type ApuracaoResumo,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { logAppError, logAppInfo } from "../services/logger";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const savingBatida = ref(false);
const savingOcorrencia = ref(false);
const error = ref("");
const message = ref("");
const batidaModalOpen = ref(false);
const ocorrenciaModalOpen = ref(false);

const employeeOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const batidas = ref<GenericRecord[]>([]);
const ocorrencias = ref<GenericRecord[]>([]);
const apuracaoResumo = ref<ApuracaoResumo | null>(null);
const reportHtml = ref("");
const empresaResponsavel = ref("Responsável / RH");

const filtros = reactive({
  funcionarioId: "",
  dataInicial: new Date().toISOString().slice(0, 10),
  dataFinal: new Date().toISOString().slice(0, 10),
  modeloRelatorio: "cartao_ponto",
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

function closeBatidaModal() {
  batidaModalOpen.value = false;
}

function openNovaBatida(referenceDate?: string) {
  resetBatida();
  batidaForm.funcionario_id = filtros.funcionarioId || batidaForm.funcionario_id || "";
  batidaForm.data_referencia = referenceDate || filtros.dataInicial || new Date().toISOString().slice(0, 10);
  batidaModalOpen.value = true;
}

function closeOcorrenciaModal() {
  ocorrenciaModalOpen.value = false;
}

function openNovaOcorrencia(referenceDate?: string) {
  resetOcorrencia();
  ocorrenciaForm.funcionario_id = filtros.funcionarioId || ocorrenciaForm.funcionario_id || "";
  ocorrenciaForm.data_referencia = referenceDate || filtros.dataInicial || new Date().toISOString().slice(0, 10);
  ocorrenciaModalOpen.value = true;
}

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
    const [employees, justificativas, empresas] = await Promise.all([
      listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true }),
      comboList("justificativas"),
      listCompanies({ onlyActive: true }),
    ]);
    employeeOptions.value = employees.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
    justificativaOptions.value = justificativas;

    if (!filtros.funcionarioId && employeeOptions.value.length > 0) {
      filtros.funcionarioId = String(employeeOptions.value[0].id);
    }

    if (!batidaForm.funcionario_id) batidaForm.funcionario_id = filtros.funcionarioId;
    if (!ocorrenciaForm.funcionario_id) ocorrenciaForm.funcionario_id = filtros.funcionarioId;
    const activeCompany = empresas.find((item) => Number(item.id) === Number(session.activeCompanyId));
    empresaResponsavel.value = String(activeCompany?.responsavel_nome || "Responsável / RH");
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
    const [rowsBatida, rowsOcorrencia, apuracao] = await Promise.all([
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
      apurarPeriodo({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioIdNumero.value,
        dataInicial: filtros.dataInicial || null,
        dataFinal: filtros.dataFinal || null,
      }),
    ]);
    batidas.value = rowsBatida;
    ocorrencias.value = rowsOcorrencia;
    apuracaoResumo.value = apuracao;
    reportHtml.value = buildCartaoHtml();
  } catch (err) {
    batidas.value = [];
    ocorrencias.value = [];
    apuracaoResumo.value = null;
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
  const safe = Math.max(0, Number(value || 0));
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

function toBase64Utf8(content: string) {
  return btoa(unescape(encodeURIComponent(content)));
}

interface DailyReportRow {
  day: string;
  dayLabel: string;
  ent1: string;
  sai1: string;
  ent2: string;
  sai2: string;
  ent3: string;
  sai3: string;
  previsto: string;
  realizado: string;
  interJornada: string;
  intraJornada: string;
  hDiurnas: string;
  hNoturnas: string;
  hTrabalhadas: string;
  hTotais: string;
  heDiurnas: string;
  heNoturnas: string;
  heTotal: string;
  atraso: string;
  normal: string;
  falta: string;
  extra: string;
  ocorrencias: string;
}

function calcInterJornada(previousEndMinutes: number | null): string {
  if (previousEndMinutes == null) return "24h+";
  const currentStart = parseTimeToMinutes("00:00");
  if (currentStart == null) return "24h+";
  const total = (24 * 60 - previousEndMinutes) + currentStart;
  if (total >= 24 * 60) return "24h+";
  return minutesToHHMM(total);
}

function splitNightMinutes(start: number, end: number): { day: number; night: number } {
  if (end <= start) return { day: 0, night: 0 };
  const nightStart = 22 * 60;
  const nightEnd = 5 * 60;
  let night = 0;
  let day = 0;
  for (let m = start; m < end; m += 1) {
    const minuteOfDay = m % (24 * 60);
    const isNight = minuteOfDay >= nightStart || minuteOfDay < nightEnd;
    if (isNight) night += 1;
    else day += 1;
  }
  return { day, night };
}

function calcIntraFromBatidas(batidasDia: string[]): number {
  if (batidasDia.length < 3) return 0;
  const outInterval = parseTimeToMinutes(batidasDia[1]);
  const inInterval = parseTimeToMinutes(batidasDia[2]);
  if (outInterval == null || inInterval == null || inInterval <= outInterval) return 0;
  return inInterval - outInterval;
}

function buildDailyRows(initial: Date, final: Date): {
  rows: DailyReportRow[];
  totals: Record<string, number>;
} {
  const apuracaoByDate = new Map<string, ApuracaoDia>();
  for (const row of apuracaoResumo.value?.rows || []) {
    apuracaoByDate.set(row.data, row);
  }

  const rows: DailyReportRow[] = [];
  let previousLastPunch: number | null = null;
  const totals = {
    normal: 0,
    falta: 0,
    extra: 0,
    noturno: 0,
    atraso: 0,
  };
  for (let cursor = new Date(initial); cursor <= final; cursor.setDate(cursor.getDate() + 1)) {
    const day = formatDate(cursor);
    const apuracaoDia = apuracaoByDate.get(day);
    const punches = apuracaoDia?.batidas || [];
    const occLabel = apuracaoDia?.ocorrencias?.join(" | ") || (punches.length ? "Verificada" : "Falta");
    const esperado = Number(apuracaoDia?.horario_esperado_minutos || 0);
    const trabalhado = Number(apuracaoDia?.trabalhado_minutos || 0);
    const extra = Math.max(0, Number(apuracaoDia?.extra_minutos || 0));
    const atraso = Math.max(0, Number(apuracaoDia?.atraso_minutos || 0));
    const saldo = Number(apuracaoDia?.saldo_minutos || 0);
    const falta = apuracaoDia?.abonado ? 0 : Math.max(0, -saldo);
    const previsto = apuracaoDia ? `${apuracaoDia.jornada_nome} (${minutesToHHMM(esperado)})` : "Sem jornada";
    const realizado = punches.length ? punches.join(" | ") : "Folga";
    const cols = [...punches, "", "", "", "", "", ""].slice(0, 6);

    const intra = calcIntraFromBatidas(punches);
    let diurno = 0;
    for (let i = 0; i + 1 < punches.length; i += 2) {
      const start = parseTimeToMinutes(punches[i]);
      const end = parseTimeToMinutes(punches[i + 1] || "");
      if (start != null && end != null && end > start) {
        const split = splitNightMinutes(start, end);
        diurno += split.day;
      }
    }
    const noturno = Math.max(0, trabalhado - diurno);
    const normal = Math.min(esperado, trabalhado);

    totals.normal += normal;
    totals.falta += falta;
    totals.extra += extra;
    totals.noturno += noturno;
    totals.atraso += atraso;

    const currentLastPunch = parseTimeToMinutes(punches[punches.length - 1] || "");
    rows.push({
      day: day.split("-").reverse().join("/"),
      dayLabel: dayLabel(cursor),
      ent1: cols[0] || "Folga",
      sai1: cols[1] || "Folga",
      ent2: cols[2] || "Folga",
      sai2: cols[3] || "Folga",
      ent3: cols[4] || "Folga",
      sai3: cols[5] || "Folga",
      previsto,
      realizado,
      interJornada: calcInterJornada(previousLastPunch),
      intraJornada: minutesToHHMM(intra),
      hDiurnas: minutesToHHMM(diurno),
      hNoturnas: minutesToHHMM(noturno),
      hTrabalhadas: minutesToHHMM(trabalhado),
      hTotais: minutesToHHMM(trabalhado),
      heDiurnas: minutesToHHMM(Math.max(0, extra - noturno)),
      heNoturnas: minutesToHHMM(Math.min(extra, noturno)),
      heTotal: minutesToHHMM(extra),
      atraso: minutesToHHMM(atraso),
      normal: minutesToHHMM(normal),
      falta: minutesToHHMM(falta),
      extra: minutesToHHMM(extra),
      ocorrencias: occLabel,
    });
    previousLastPunch = currentLastPunch;
  }

  return { rows, totals };
}

function buildCartaoHtml(): string {
  if (!filtros.dataInicial || !filtros.dataFinal) return "";
  const initial = new Date(`${filtros.dataInicial}T00:00:00`);
  const final = new Date(`${filtros.dataFinal}T00:00:00`);
  if (Number.isNaN(initial.getTime()) || Number.isNaN(final.getTime()) || initial > final) return "";

  const { rows: dailyRows, totals } = buildDailyRows(initial, final);

  const logoSvg = `<svg xmlns='http://www.w3.org/2000/svg' width='180' height='44' viewBox='0 0 420 100'><rect width='100' height='100' rx='18' fill='#1d4ed8'/><path d='M50 24v28l18-14' stroke='#fff' stroke-width='8' stroke-linecap='round'/><circle cx='50' cy='50' r='32' fill='none' stroke='rgba(255,255,255,.35)' stroke-width='8'/><text x='122' y='45' font-family='Segoe UI, Arial' font-size='28' font-weight='700' fill='#1f2937'>Ponto Manager</text><text x='122' y='74' font-family='Segoe UI, Arial' font-size='14' fill='#64748b'>jornada • rep • banco de horas</text></svg>`;
  const tableByModel: Record<string, string> = {
    cartao_ponto: `
      <thead><tr><th>Dia</th><th>Ent.1</th><th>Saí.1</th><th>Ent.2</th><th>Saí.2</th><th>Ent.3</th><th>Saí.3</th><th>Normais</th><th>Faltas</th><th>Extras</th><th>Observações</th></tr></thead>
      <tbody>
      ${dailyRows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.ent1}</td><td>${r.sai1}</td><td>${r.ent2}</td><td>${r.sai2}</td><td>${r.ent3}</td><td>${r.sai3}</td><td>${r.normal}</td><td>${r.falta}</td><td>${r.extra}</td><td>${r.ocorrencias}</td></tr>`).join("")}
      <tr class="tot"><td colspan="7">TOTAIS</td><td>${minutesToHHMM(totals.normal)}</td><td>${minutesToHHMM(totals.falta)}</td><td>${minutesToHHMM(totals.extra)}</td><td>-</td></tr>
      </tbody>
    `,
    folha_resumida: `
      <thead><tr><th>Data</th><th>Previsto</th><th>Realizado</th><th>H. trab.</th></tr></thead>
      <tbody>
      ${dailyRows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.realizado}</td><td>${r.hTrabalhadas}</td></tr>`).join("")}
      <tr class="tot"><td colspan="3">TOTAIS</td><td>${minutesToHHMM(totals.normal + totals.extra)}</td></tr>
      </tbody>
    `,
    folha_interjornada: `
      <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. trab.</th></tr></thead>
      <tbody>
      ${dailyRows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTrabalhadas}</td></tr>`).join("")}
      <tr class="tot"><td colspan="7">TOTAIS</td><td>${minutesToHHMM(totals.normal + totals.extra)}</td></tr>
      </tbody>
    `,
    folha_com_he: `
      <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. totais</th><th>HE diurnas</th><th>HE noturnas</th><th>HE total</th><th>H. trab.</th><th>Atraso</th></tr></thead>
      <tbody>
      ${dailyRows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTotais}</td><td>${r.heDiurnas}</td><td>${r.heNoturnas}</td><td>${r.heTotal}</td><td>${r.hTrabalhadas}</td><td>${r.atraso}</td></tr>`).join("")}
      <tr class="tot"><td colspan="11">TOTAIS</td><td>${minutesToHHMM(totals.normal + totals.extra)}</td><td>${minutesToHHMM(totals.atraso)}</td></tr>
      </tbody>
    `,
    folha_completa: `
      <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. totais</th><th>HE diurnas</th><th>HE noturnas</th><th>HE total</th><th>H. trab.</th><th>Atraso</th></tr></thead>
      <tbody>
      ${dailyRows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTotais}</td><td>${r.heDiurnas}</td><td>${r.heNoturnas}</td><td>${r.heTotal}</td><td>${r.hTrabalhadas}</td><td>${r.atraso}</td></tr>`).join("")}
      <tr class="tot"><td colspan="11">TOTAIS</td><td>${minutesToHHMM(totals.normal + totals.extra)}</td><td>${minutesToHHMM(totals.atraso)}</td></tr>
      </tbody>
    `,
  };

  const summaryByModel = filtros.modeloRelatorio === "folha_completa" ? `
    <div class="summary-grid">
      <div class="summary-box"><strong>Total atrasos</strong><div>${minutesToHHMM(totals.atraso)}</div></div>
      <div class="summary-box"><strong>Total horas noturnas</strong><div>${minutesToHHMM(totals.noturno)}</div></div>
      <div class="summary-box"><strong>Total H.E. acumuladas</strong><div>${minutesToHHMM(totals.extra)}</div></div>
      <div class="summary-box"><strong>Total banco de horas</strong><div>${minutesToHHMM(totals.extra - totals.falta)}</div></div>
    </div>
  ` : "";

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
      .summary-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;margin-top:12px}
      .summary-box{border:1px solid #666;padding:6px;text-align:center}
    </style></head>
    <body>
      <div class="head">
        <div>
          <div>${logoSvg}</div>
          <h1>CARTÃO PONTO — ${filtros.modeloRelatorio.replace(/_/g, " " ).toUpperCase()}</h1>
          <div class="meta">Período: ${filtros.dataInicial.split("-").reverse().join("/")} até ${filtros.dataFinal.split("-").reverse().join("/")}</div>
          <div class="meta">Empresa: ${session.activeCompanyName || "-"}</div>
          <div class="meta">Colaborador: ${funcionarioNomeSelecionado.value}</div>
        </div>
        <div class="meta">Emitido em ${new Date().toLocaleDateString("pt-BR")}</div>
      </div>
      <table>
        ${tableByModel[filtros.modeloRelatorio] || tableByModel.folha_resumida}
      </table>
      ${summaryByModel}
      <div class="sign"><div class="line">${funcionarioNomeSelecionado.value}</div><div class="line">${empresaResponsavel.value}</div></div>
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
    const fileName = `cartao_ponto_${funcionarioNomeSelecionado.value.replace(/\\s+/g, "_")}_${filtros.dataInicial}_${filtros.dataFinal}.html`;
    await saveWithDialog(reportHtml.value, fileName, "text/html");
    await registerGeneratedReport({
      descricao: "Cartão de ponto",
      tipoRelatorio: "cartao_ponto",
      origemRotina: "cartao_ponto",
      formato: "HTML",
      fileName,
      mimeType: "text/html",
      competencia: `${filtros.dataInicial}..${filtros.dataFinal}`,
      funcionarioId: funcionarioIdNumero.value,
      funcionarioNome: funcionarioNomeSelecionado.value,
      usuarioLogin: session.user?.login || null,
      detalhado: true,
      status: "GERADO",
      contentBase64: toBase64Utf8(reportHtml.value),
    });
    message.value = "Cartão exportado em HTML e registrado em Relatórios Gerados.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar HTML.";
  }
}

async function exportarExcel() {
  try {
    reportHtml.value = buildCartaoHtml();
    const fileName = `cartao_ponto_${funcionarioNomeSelecionado.value.replace(/\\s+/g, "_")}_${filtros.dataInicial}_${filtros.dataFinal}.xls`;
    await saveWithDialog(reportHtml.value, fileName, "application/vnd.ms-excel");
    await registerGeneratedReport({
      descricao: "Cartão de ponto",
      tipoRelatorio: "cartao_ponto",
      origemRotina: "cartao_ponto",
      formato: "EXCEL",
      fileName,
      mimeType: "application/vnd.ms-excel",
      competencia: `${filtros.dataInicial}..${filtros.dataFinal}`,
      funcionarioId: funcionarioIdNumero.value,
      funcionarioNome: funcionarioNomeSelecionado.value,
      usuarioLogin: session.user?.login || null,
      detalhado: true,
      status: "GERADO",
      contentBase64: toBase64Utf8(reportHtml.value),
    });
    message.value = "Cartão exportado em Excel e registrado em Relatórios Gerados.";
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
      registerGeneratedReport({
        descricao: "Cartão de ponto (impressão/PDF)",
        tipoRelatorio: "cartao_ponto",
        origemRotina: "cartao_ponto",
        formato: "PDF",
        fileName: `cartao_ponto_${funcionarioNomeSelecionado.value.replace(/\\s+/g, "_")}_${filtros.dataInicial}_${filtros.dataFinal}.pdf`,
        mimeType: "application/pdf",
        competencia: `${filtros.dataInicial}..${filtros.dataFinal}`,
        funcionarioId: funcionarioIdNumero.value,
        funcionarioNome: funcionarioNomeSelecionado.value,
        usuarioLogin: session.user?.login || null,
        detalhado: true,
        status: "GERADO",
        contentBase64: toBase64Utf8(reportHtml.value),
      }).catch(() => {});
      message.value = "Impressão iniciada. O relatório também foi registrado em Relatórios Gerados.";
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
    closeBatidaModal();
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
    closeOcorrenciaModal();
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar ocorrência.";
    logAppError("cartao_ponto", "Falha ao salvar ocorrência.", { error: error.value, payload: { ...ocorrenciaForm } });
  } finally {
    savingOcorrencia.value = false;
  }
}

function editarBatida(row: GenericRecord) {
  batidaModalOpen.value = true;
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
  ocorrenciaModalOpen.value = true;
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
  openNovaBatida(referenceDate);
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
        <div class="field">
          <label>Modelo do relatório</label>
          <select v-model="filtros.modeloRelatorio">
            <option value="cartao_ponto">0) Cartão de ponto (padrão)</option>
            <option value="folha_resumida">1) Folha resumida</option>
            <option value="folha_interjornada">2) Folha com inter/intra jornada</option>
            <option value="folha_com_he">3) Folha com HE e atrasos</option>
            <option value="folha_completa">4) Folha completa com resumos</option>
          </select>
        </div>
        <div class="actions align-end">
          <button class="primary" :disabled="loading" @click="carregarCartao">Aplicar filtros</button>
        </div>
      </div>
    </div>

    <div class="actions">
      <button class="secondary" @click="openNovaBatida()">Nova marcação</button>
      <button class="secondary" @click="openNovaOcorrencia()">Nova ocorrência</button>
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

    <AppModal
      :open="batidaModalOpen"
      :title="batidaForm.id ? 'Editar marcação' : 'Nova marcação'"
      subtitle="Fluxo de inclusão e edição convertido para modal, mantendo a visão operacional do cartão."
      width="lg"
      @close="closeBatidaModal"
    >
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
        <AppSwitch v-model="batidaForm.manual_ajuste" label="Ajuste manual" />
        <AppSwitch v-model="batidaForm.validado" label="Validado" />
      </div>
      <div class="actions">
        <button class="primary" :disabled="savingBatida" @click="salvarBatida">{{ savingBatida ? 'Salvando...' : batidaForm.id ? 'Atualizar marcação' : 'Salvar marcação' }}</button>
        <button class="secondary" @click="resetBatida">Limpar</button>
      </div>
    </AppModal>

    <AppModal
      :open="ocorrenciaModalOpen"
      :title="ocorrenciaForm.id ? 'Editar ocorrência' : 'Nova ocorrência'"
      subtitle="Fluxo de inclusão e edição convertido para modal, mantendo a listagem operacional atual."
      width="lg"
      @close="closeOcorrenciaModal"
    >
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
        <AppSwitch v-model="ocorrenciaForm.abonar_dia" label="Abonar dia" />
      </div>
      <div class="actions">
        <button class="primary" :disabled="savingOcorrencia" @click="salvarOcorrencia">{{ savingOcorrencia ? 'Salvando...' : ocorrenciaForm.id ? 'Atualizar ocorrência' : 'Salvar ocorrência' }}</button>
        <button class="secondary" @click="resetOcorrencia">Limpar</button>
      </div>
    </AppModal>

    <iframe v-if="reportHtml" class="report-frame" :src="`data:text/html;charset=utf-8,${encodeURIComponent(reportHtml)}`" title="Prévia cartão de ponto" />
  </div>
</template>
