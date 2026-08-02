<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import AppSwitch from "../components/AppSwitch.vue";
import AppPageTitleBar from "../components/base/AppPageTitleBar.vue";
import BaseFilterBar from "../components/base/BaseFilterBar.vue";
import {
  apurarPeriodo,
  comboList,
  deleteBatida,
  deleteOcorrencia,
  listBatidas,
  listCompanies,
  listEmployees,
  listOcorrencias,
  markBatidaDuplicate,
  reactivateBatida,
  registerGeneratedReport,
  saveBatida,
  saveOcorrencia,
  type ApuracaoDia,
  type ApuracaoResumo,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { logAppError, logAppInfo } from "../services/logger";
import { printHtmlExternally } from "../services/print";
import { showSplashError, showSplashInfo, showSplashSuccess } from "../services/splash";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const savingBatida = ref(false);
const savingOcorrencia = ref(false);
const printingAllCompetencia = ref(false);
const error = ref("");
const message = ref("");
const batidaModalOpen = ref(false);
const ocorrenciaModalOpen = ref(false);

const employeeOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const batidas = ref<GenericRecord[]>([]);
const batidasInativas = ref<GenericRecord[]>([]);
const ocorrencias = ref<GenericRecord[]>([]);
const apuracaoResumo = ref<ApuracaoResumo | null>(null);
const reportHtml = ref("");
const empresaResponsavel = ref("Responsável / RH");

const selectedDate = ref("");
const smartBusy = ref(false);
const duplicateBusy = ref(false);
const smartFaltaTipo = ref("falta");
const smartJustificativaId = ref("");
const smartSuggestionSelection = reactive<Record<string, boolean>>({});
const duplicateSelection = reactive<Record<string, boolean>>({});

type SmartSuggestionType = "esquecimento" | "falta" | "troca_folga" | "meia_folga" | "falta_continua" | "atestado_provavel";

interface SmartSuggestionItem {
  key: string;
  date: string;
  funcionarioId: number;
  funcionarioNome: string;
  tipo: SmartSuggestionType;
  titulo: string;
  observacao: string;
  seguro: boolean;
  esperadoMinutos: number;
  trabalhadoMinutos: number;
  batidas: string[];
}

interface DuplicatePunchCandidate {
  key: string;
  date: string;
  funcionarioNome: string;
  horarioBase: string;
  principalId: number;
  duplicateIds: number[];
  principalOrigem: string;
  repeticoes: number;
  diferencaSegundos: number;
}

function isProtectedPunch(row: GenericRecord | null | undefined): boolean {
  if (!row) return false;
  if (Number(row.origem_protegida || 0) === 1 || row.origem_protegida === true) return true;
  const origem = String(row.origem || '').toLowerCase();
  return origem.includes('afd') || origem.includes('conector') || origem.includes('rep');
}

const smartSuggestions = ref<SmartSuggestionItem[]>([]);
const duplicateCandidates = ref<DuplicatePunchCandidate[]>([]);
const TOLERANCIA_SALDO_CONSOLIDADO_MINUTOS = 5;

function saldoCredorConsolidado(saldoMinutos: number): number {
  return saldoMinutos > TOLERANCIA_SALDO_CONSOLIDADO_MINUTOS ? saldoMinutos : 0;
}

function saldoDevedorConsolidado(saldoMinutos: number): number {
  return saldoMinutos < -TOLERANCIA_SALDO_CONSOLIDADO_MINUTOS ? Math.abs(saldoMinutos) : 0;
}

const gridEditor = reactive<Record<string, string>>({});
const gridSaving = reactive<Record<string, boolean>>({});
const gridCellRefs = ref<Record<string, HTMLInputElement | null>>({});
const activeView = ref<"edicao" | "previsualizacao">("edicao");
const activeSideTab = ref<"marcacoes" | "ocorrencias" | "smart" | "exclusao">("marcacoes");
const sidePanelCollapsed = ref(false);
const gridStatus = ref('Pronto para edição inline. Use Enter, setas e Del para operar a grade.');

const hoje = new Date();
const competenciaAtual = getCompetenciaRange(hoje.getFullYear(), hoje.getMonth() + 1);
const filtros = reactive({
  funcionarioId: "",
  modoPeriodo: "competencia" as "intervalo" | "competencia",
  competenciaMes: hoje.getMonth() + 1,
  competenciaAno: hoje.getFullYear(),
  dataInicial: competenciaAtual.dataInicial,
  dataFinal: competenciaAtual.dataFinal,
  modeloRelatorio: "cartao_ponto",
});

async function clearCartaoFilters() {
  filtros.funcionarioId = employeeOptions.value.length ? String(employeeOptions.value[0].id) : "";
  filtros.modoPeriodo = "competencia";
  filtros.competenciaMes = hoje.getMonth() + 1;
  filtros.competenciaAno = hoje.getFullYear();
  filtros.dataInicial = competenciaAtual.dataInicial;
  filtros.dataFinal = competenciaAtual.dataFinal;
  filtros.modeloRelatorio = "cartao_ponto";
  await carregarCartao();
}

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
const inconsistenciasNoPeriodo = computed(() => (apuracaoResumo.value?.rows || []).filter((row) => row.inconsistente).length);
const diasComOcorrenciaNoPeriodo = computed(() => (apuracaoResumo.value?.rows || []).filter((row) => (row.ocorrencias || []).length > 0).length);
const periodoLabel = computed(() => {
  if (filtros.modoPeriodo === "competencia") {
    return `${String(filtros.competenciaMes).padStart(2, "0")}/${filtros.competenciaAno}`;
  }
  return `${filtros.dataInicial}..${filtros.dataFinal}`;
});
const previewOrientation = computed(() => isCartaoModeloPaisagem(filtros.modeloRelatorio) ? "landscape" : "portrait");

function getCompetenciaRange(ano: number, mes: number) {
  const inicio = new Date(ano, mes - 1, 1);
  const fim = new Date(ano, mes, 0);
  return {
    dataInicial: formatDate(inicio),
    dataFinal: formatDate(fim),
  };
}

function syncPeriodFilters() {
  if (filtros.modoPeriodo !== "competencia") return;
  const range = getCompetenciaRange(Number(filtros.competenciaAno), Number(filtros.competenciaMes));
  filtros.dataInicial = range.dataInicial;
  filtros.dataFinal = range.dataFinal;
}

function periodoAtual() {
  if (filtros.modoPeriodo === "competencia") {
    return getCompetenciaRange(Number(filtros.competenciaAno), Number(filtros.competenciaMes));
  }
  return {
    dataInicial: filtros.dataInicial,
    dataFinal: filtros.dataFinal,
  };
}


interface DailyGridRow extends DailyReportRow {
  isoDate: string;
  inconsistente: boolean;
  ocorrenciasCount: number;
  mensagens: string[];
  batidasRaw: string[];
  workedMinutes: number;
  expectedMinutes: number;
  saldoMinutes: number;
}

interface GridBatidaSlot {
  key: string;
  date: string;
  slotIndex: number;
  value: string;
  record: GenericRecord | null;
  tipo: string;
}

function normalizeHourInput(value: string): string {
  const raw = String(value || '').trim();
  if (!raw) return '';
  const only = raw.replace(/[^\d]/g, '');
  if (only.length === 3) {
    return `${only.slice(0, 1).padStart(2, '0')}:${only.slice(1, 3)}`;
  }
  if (only.length >= 4) {
    return `${only.slice(0, 2)}:${only.slice(2, 4)}`;
  }
  return raw;
}

function isValidHourInput(value: string): boolean {
  return /^([01]\d|2[0-3]):([0-5]\d)$/.test(value);
}

const dailyGridRows = computed<DailyGridRow[]>(() => {
  const periodo = periodoAtual();
  if (!periodo.dataInicial || !periodo.dataFinal) return [];
  const initial = new Date(`${periodo.dataInicial}T00:00:00`);
  const final = new Date(`${periodo.dataFinal}T00:00:00`);
  if (Number.isNaN(initial.getTime()) || Number.isNaN(final.getTime()) || initial > final) return [];

  const { rows } = buildDailyRows(apuracaoResumo.value, initial, final);
  return rows.map((row, index) => {
    const cursor = new Date(initial);
    cursor.setDate(initial.getDate() + index);
    const isoDate = formatDate(cursor);
    const resumo = apuracaoResumo.value?.rows.find((item) => item.data === isoDate);
    return {
      ...row,
      isoDate,
      inconsistente: Boolean(resumo?.inconsistente),
      ocorrenciasCount: (resumo?.ocorrencias || []).length,
      mensagens: resumo?.mensagens || [],
      batidasRaw: resumo?.batidas || [],
      workedMinutes: Number(resumo?.trabalhado_minutos || 0),
      expectedMinutes: Number(resumo?.horario_esperado_minutos || 0),
      saldoMinutes: Number(resumo?.saldo_minutos || 0),
    };
  });
});

const gridSlotsByDate = computed<Record<string, GridBatidaSlot[]>>(() => {
  const employeeId = funcionarioIdNumero.value;
  const map: Record<string, GridBatidaSlot[]> = {};
  for (const row of dailyGridRows.value) {
    const dayBatidas = batidas.value
      .filter((item) => String(item.data_referencia || '') === row.isoDate && (!employeeId || Number(item.funcionario_id) === employeeId))
      .sort((a, b) => String(a.hora || '').localeCompare(String(b.hora || '')) || Number(a.id || 0) - Number(b.id || 0));
    const slots: GridBatidaSlot[] = [];
    for (let i = 0; i < 6; i += 1) {
      const record = dayBatidas[i] || null;
      const key = `${row.isoDate}:${i}`;
      const defaultValue = record ? String(record.hora || '').slice(0, 5) : '';
      if (gridEditor[key] == null) {
        gridEditor[key] = defaultValue;
      }
      slots.push({
        key,
        date: row.isoDate,
        slotIndex: i,
        value: gridEditor[key] ?? defaultValue,
        record,
        tipo: i % 2 === 0 ? 'entrada' : 'saida',
      });
    }
    map[row.isoDate] = slots;
  }
  return map;
});

function syncGridEditorFromData() {
  for (const [date, slots] of Object.entries(gridSlotsByDate.value)) {
    for (const slot of slots) {
      const key = `${date}:${slot.slotIndex}`;
      const value = slot.record ? String(slot.record.hora || "").slice(0, 5) : "";
      gridEditor[key] = value;
    }
  }
}

const selectedDaySummary = computed(() => dailyGridRows.value.find((item) => item.isoDate === selectedDate.value) || null);
const batidasSelecionadas = computed(() => selectedDate.value ? batidas.value.filter((item) => String(item.data_referencia || '') === selectedDate.value) : batidas.value);
const ocorrenciasSelecionadas = computed(() => selectedDate.value ? ocorrencias.value.filter((item) => String(item.data_referencia || '') === selectedDate.value) : ocorrencias.value);
const selectedDayLabel = computed(() => selectedDaySummary.value ? `${selectedDaySummary.value.day} • ${selectedDaySummary.value.dayLabel}` : 'Nenhum dia selecionado');
const smartResumo = computed(() => ({
  esquecimentos: smartSuggestions.value.filter((item) => item.tipo === 'esquecimento').length,
  faltas: smartSuggestions.value.filter((item) => item.tipo === 'falta' || item.tipo === 'falta_continua').length,
  trocasFolga: smartSuggestions.value.filter((item) => item.tipo === 'troca_folga').length,
  meiasFolga: smartSuggestions.value.filter((item) => item.tipo === 'meia_folga').length,
  atestados: smartSuggestions.value.filter((item) => item.tipo === 'atestado_provavel').length,
}));

function suggestionBadgeClass(tipo: SmartSuggestionType) {
  if (tipo === 'falta' || tipo === 'falta_continua') return 'badge-danger';
  if (tipo === 'troca_folga' || tipo === 'atestado_provavel') return 'badge-info';
  return 'badge-warning';
}

function dailyRowClass(row: DailyGridRow) {
  if (row.isoDate === selectedDate.value) return 'vb6-selected-row';
  if (row.inconsistente) return 'row-highlight-warning';
  if (row.ocorrenciasCount > 0) return 'row-highlight-info';
  return '';
}

function selectDay(date: string) {
  selectedDate.value = date;
}

function toggleSidebar() {
  sidePanelCollapsed.value = !sidePanelCollapsed.value;
}

async function selectView(view: "edicao" | "previsualizacao") {
  activeView.value = view;
  if (view === "previsualizacao") {
    await carregarCartao();
  }
}

function resetSelectionMap(target: Record<string, boolean>) {
  Object.keys(target).forEach((key) => delete target[key]);
}

function generateSmartSuggestionsFromSummary(summary: ApuracaoResumo | null, employeeId: number | null, employeeName: string): SmartSuggestionItem[] {
  const items: SmartSuggestionItem[] = [];
  if (!summary || !employeeId) return items;

  const rows = [...(summary.rows || [])].sort((a, b) => String(a.data).localeCompare(String(b.data)));
  let consecutiveAbsences = 0;
  let currentAbsenceDates: string[] = [];

  for (const row of rows) {
    const batidas = row.batidas || [];
    const mensagens = row.mensagens || [];
    const worked = Number(row.trabalhado_minutos || 0);
    const expected = Number(row.horario_esperado_minutos || 0);
    const saldo = Number(row.saldo_minutos || 0);
    const hasOccurrence = (row.ocorrencias || []).length > 0;
    const date = String(row.data || '');

    if (batidas.length > 0 && batidas.length % 2 === 1) {
      items.push({
        key: `${date}:esquecimento`, date, funcionarioId: employeeId, funcionarioNome: employeeName,
        tipo: 'esquecimento', titulo: 'Possível esquecimento de batida',
        observacao: mensagens.join(' | ') || 'Quantidade ímpar de marcações no dia.',
        seguro: false, esperadoMinutos: expected, trabalhadoMinutos: worked, batidas,
      });
    }

    if (expected > 0 && worked === 0 && !hasOccurrence) {
      consecutiveAbsences += 1;
      currentAbsenceDates.push(date);
      items.push({
        key: `${date}:falta`, date, funcionarioId: employeeId, funcionarioNome: employeeName,
        tipo: 'falta', titulo: 'Falta sem marcação',
        observacao: mensagens.join(' | ') || 'Jornada esperada sem batidas e sem ocorrência.',
        seguro: true, esperadoMinutos: expected, trabalhadoMinutos: worked, batidas,
      });
    } else {
      if (consecutiveAbsences >= 2 && currentAbsenceDates.length) {
        const first = currentAbsenceDates[0];
        const last = currentAbsenceDates[currentAbsenceDates.length - 1];
        items.push({
          key: `${first}:falta_continua`, date: first, funcionarioId: employeeId, funcionarioNome: employeeName,
          tipo: consecutiveAbsences >= 3 ? 'atestado_provavel' : 'falta_continua',
          titulo: consecutiveAbsences >= 3 ? 'Atestado provável / ausência contínua' : 'Falta contínua provável',
          observacao: `Ausência contínua detectada entre ${first} e ${last}.`,
          seguro: false, esperadoMinutos: 0, trabalhadoMinutos: 0, batidas: [],
        });
      }
      consecutiveAbsences = 0;
      currentAbsenceDates = [];
    }

    if (expected === 0 && worked > 0) {
      items.push({
        key: `${date}:troca_folga`, date, funcionarioId: employeeId, funcionarioNome: employeeName,
        tipo: 'troca_folga', titulo: 'Possível troca de folga',
        observacao: mensagens.join(' | ') || 'Dia tratado como folga, mas houve marcação de trabalho.',
        seguro: true, esperadoMinutos: expected, trabalhadoMinutos: worked, batidas,
      });
    }

    if (expected > 0 && worked > 0 && worked < Math.ceil(expected * 0.65) && saldo < 0 && !hasOccurrence) {
      items.push({
        key: `${date}:meia_folga`, date, funcionarioId: employeeId, funcionarioNome: employeeName,
        tipo: 'meia_folga', titulo: 'Jornada parcial / meia folga provável',
        observacao: mensagens.join(' | ') || 'Cumprimento parcial relevante da jornada sem ocorrência registrada.',
        seguro: true, esperadoMinutos: expected, trabalhadoMinutos: worked, batidas,
      });
    }
  }

  if (consecutiveAbsences >= 2 && currentAbsenceDates.length) {
    const first = currentAbsenceDates[0];
    const last = currentAbsenceDates[currentAbsenceDates.length - 1];
    items.push({
      key: `${first}:falta_continua`, date: first, funcionarioId: employeeId, funcionarioNome: employeeName,
      tipo: consecutiveAbsences >= 3 ? 'atestado_provavel' : 'falta_continua',
      titulo: consecutiveAbsences >= 3 ? 'Atestado provável / ausência contínua' : 'Falta contínua provável',
      observacao: `Ausência contínua detectada entre ${first} e ${last}.`,
      seguro: false, esperadoMinutos: 0, trabalhadoMinutos: 0, batidas: [],
    });
  }

  return items;
}

function analisarSugestoes() {
  smartSuggestions.value = generateSmartSuggestionsFromSummary(apuracaoResumo.value, funcionarioIdNumero.value, funcionarioNomeSelecionado.value);
  resetSelectionMap(smartSuggestionSelection);
  for (const item of smartSuggestions.value) {
    smartSuggestionSelection[item.key] = item.seguro;
  }
  if (smartSuggestions.value.length) {
    message.value = `${smartSuggestions.value.length} sugestão(ões) smart analisadas para o período.`;
  } else {
    message.value = 'Nenhuma sugestão smart gerada para o período atual.';
  }
}

async function aplicarSugestoesSelecionadas(apenasSeguras = false) {
  const selecionadas = smartSuggestions.value.filter((item) => smartSuggestionSelection[item.key] && (!apenasSeguras || item.seguro));
  if (!selecionadas.length) {
    message.value = 'Nenhuma sugestão selecionada para aplicação.';
    return;
  }

  smartBusy.value = true;
  error.value = '';
  message.value = '';
  try {
    for (const item of selecionadas) {
      let tipo = 'ajuste_manual';
      let observacao = `[SMART] ${item.titulo}. ${item.observacao}`;
      let abonarDia = false;
      let minutosAbonados = 0;

      if (item.tipo === 'falta' || item.tipo === 'falta_continua') {
        tipo = smartFaltaTipo.value || 'falta';
      } else if (item.tipo === 'atestado_provavel') {
        tipo = 'atestado';
        abonarDia = true;
      } else if (item.tipo === 'troca_folga') {
        tipo = 'troca_folga';
      } else if (item.tipo === 'meia_folga') {
        tipo = 'meia_folga';
        minutosAbonados = Math.max(0, item.esperadoMinutos - item.trabalhadoMinutos);
      }

      await saveOcorrencia({
        funcionario_id: item.funcionarioId,
        data_referencia: item.date,
        justificativa_id: smartJustificativaId.value ? Number(smartJustificativaId.value) : null,
        tipo,
        abonar_dia: abonarDia,
        minutos_abonados: minutosAbonados,
        observacao,
      });
    }

    message.value = `${selecionadas.length} sugestão(ões) aplicadas com sucesso.`;
    await carregarCartao();
    analisarSugestoes();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao aplicar sugestões smart.';
  } finally {
    smartBusy.value = false;
  }
}

async function tratarTodosAutomaticos() {
  if (!employeeOptions.value.length) return;
  smartBusy.value = true;
  error.value = '';
  message.value = '';
  let totalAplicado = 0;

  try {
    const periodo = periodoAtual();
    for (const employee of employeeOptions.value) {
      const summary = await apurarPeriodo({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: Number(employee.id),
        competenciaAno: filtros.modoPeriodo === 'competencia' ? Number(filtros.competenciaAno) : null,
        competenciaMes: filtros.modoPeriodo === 'competencia' ? Number(filtros.competenciaMes) : null,
        dataInicial: filtros.modoPeriodo === 'competencia' ? null : periodo.dataInicial,
        dataFinal: filtros.modoPeriodo === 'competencia' ? null : periodo.dataFinal,
      });
      const suggestions = generateSmartSuggestionsFromSummary(summary, Number(employee.id), employee.label).filter((item) => item.seguro);
      for (const item of suggestions) {
        await saveOcorrencia({
          funcionario_id: item.funcionarioId,
          data_referencia: item.date,
          justificativa_id: smartJustificativaId.value ? Number(smartJustificativaId.value) : null,
          tipo: (item.tipo === 'falta' || item.tipo === 'falta_continua') ? (smartFaltaTipo.value || 'falta') : (item.tipo === 'atestado_provavel' ? 'atestado' : item.tipo),
          abonar_dia: item.tipo === 'atestado_provavel',
          minutos_abonados: item.tipo === 'meia_folga' ? Math.max(0, item.esperadoMinutos - item.trabalhadoMinutos) : 0,
          observacao: `[SMART LOTE] ${item.titulo}. ${item.observacao}`,
        });
        totalAplicado += 1;
      }
    }

    message.value = totalAplicado > 0
      ? `${totalAplicado} sugestão(ões) automáticas aplicadas no lote da visão atual.`
      : 'Nenhuma sugestão automática segura encontrada para aplicação em lote.';
    await carregarCartao();
    analisarSugestoes();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao tratar automaticamente todos os colaboradores.';
  } finally {
    smartBusy.value = false;
  }
}

function localizarDuplicidades() {
  const grouped: DuplicatePunchCandidate[] = [];
  const rows = [...batidas.value]
    .map((item) => ({
      id: Number(item.id || 0),
      funcionarioNome: String(item.funcionario_nome || '-'),
      date: String(item.data_referencia || ''),
      hora: String(item.hora || ''),
      origem: String(item.origem || 'manual'),
      oficial: isProtectedPunch(item),
    }))
    .filter((item) => item.id > 0 && item.date && item.hora)
    .sort((a, b) => `${a.date} ${a.hora}`.localeCompare(`${b.date} ${b.hora}`));

  const byDay: Record<string, typeof rows> = {};
  for (const row of rows) {
    const key = `${row.funcionarioNome}::${row.date}`;
    (byDay[key] ||= []).push(row);
  }

  const addCandidate = (groupKey: string, entries: typeof rows) => {
    if (entries.length < 2) return;
    const [funcionarioNome, date] = groupKey.split('::');
    const prioritized = [...entries].sort((a, b) => {
      if (a.oficial !== b.oficial) return a.oficial ? -1 : 1;
      return a.hora.localeCompare(b.hora) || a.id - b.id;
    });
    const principal = prioritized[0];
    grouped.push({
      key: `${groupKey}:${entries[0].hora}`,
      date,
      funcionarioNome,
      horarioBase: entries[0].hora,
      principalId: principal.id,
      principalOrigem: principal.origem,
      duplicateIds: prioritized.slice(1).map((entry) => entry.id),
      repeticoes: entries.length,
      diferencaSegundos: Math.max(
        0,
        (parseTimeToMinutes(entries[entries.length - 1].hora) || 0)
          - (parseTimeToMinutes(entries[0].hora) || 0),
      ) * 60,
    });
  };

  for (const [groupKey, items] of Object.entries(byDay)) {
    let current: typeof items = [];
    for (const item of items) {
      const currMinutes = parseTimeToMinutes(item.hora);
      const lastMinutes = current.length ? parseTimeToMinutes(current[current.length - 1].hora) : null;
      if (current.length === 0 || currMinutes == null || lastMinutes == null || (currMinutes - lastMinutes) > 1) {
        addCandidate(groupKey, current);
        current = [item];
      } else {
        current.push(item);
      }
    }
    addCandidate(groupKey, current);
  }

  duplicateCandidates.value = grouped;
  resetSelectionMap(duplicateSelection);
  for (const item of grouped) {
    duplicateSelection[item.key] = true;
  }
  message.value = grouped.length ? `${grouped.length} agrupamento(s) de batidas muito próximas localizado(s).` : 'Nenhuma batida duplicada ou muito próxima foi localizada no filtro atual.';
}

async function excluirDuplicidadesSelecionadas() {
  const candidates = duplicateCandidates.value.filter((item) => duplicateSelection[item.key]);
  const total = candidates.reduce((sum, item) => sum + item.duplicateIds.length, 0);
  if (!total) {
    message.value = 'Nenhuma duplicidade selecionada para tratamento.';
    showSplashInfo(message.value);
    return;
  }
  if (!confirm(`Marcar ${total} batida(s) como duplicidade? Os registros serão ocultados, mas permanecerão auditáveis e poderão ser reativados.`)) return;

  duplicateBusy.value = true;
  error.value = '';
  message.value = '';
  try {
    for (const candidate of candidates) {
      for (const id of candidate.duplicateIds) {
        await markBatidaDuplicate(
          id,
          candidate.principalId,
          `Duplicidade confirmada no cartão de ponto; principal ${candidate.principalId}.`,
        );
      }
    }
    message.value = `${total} batida(s) marcadas como duplicidade, sem exclusão física.`;
    showSplashSuccess(message.value);
    await carregarCartao();
    localizarDuplicidades();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao tratar batidas duplicadas.';
    showSplashError(error.value);
  } finally {
    duplicateBusy.value = false;
  }
}

async function reativarBatidaDuplicada(row: GenericRecord) {
  if (!row.id || !confirm('Reativar esta batida? Ela voltará aos cálculos e relatórios do cartão.')) return;
  duplicateBusy.value = true;
  error.value = '';
  try {
    await reactivateBatida(Number(row.id), 'Reativação solicitada na guia Exclusão do cartão de ponto.');
    message.value = 'Batida reativada e reincluída nos cálculos.';
    showSplashSuccess(message.value);
    await carregarCartao();
    localizarDuplicidades();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao reativar a batida.';
    showSplashError(error.value);
  } finally {
    duplicateBusy.value = false;
  }
}

function batidasDia(date: string) {
  return batidas.value.filter((item) => String(item.data_referencia || '') === date);
}

function ocorrenciasDia(date: string) {
  return ocorrencias.value.filter((item) => String(item.data_referencia || '') === date);
}

function getGridSlot(date: string, slotIndex: number): GridBatidaSlot | undefined {
  return gridSlotsByDate.value[date]?.[slotIndex];
}

function setGridCellRef(key: string, el: unknown) {
  gridCellRefs.value[key] = (el as HTMLInputElement | null) || null;
}

function focusGridCell(date: string, slotIndex: number) {
  const target = gridCellRefs.value[`${date}:${slotIndex}`];
  if (target) {
    target.focus();
    target.select();
  }
}

async function commitGridCell(date: string, slotIndex: number) {
  const slot = getGridSlot(date, slotIndex);
  if (!slot) return;
  const key = slot.key;
  const input = normalizeHourInput(gridEditor[key] || '');
  const previous = slot.record ? String(slot.record.hora || '').slice(0, 5) : '';

  if (slot.record && isProtectedPunch(slot.record) && input !== previous) {
    error.value = 'Marcações oficiais AFD/REP/Connector não podem ser alteradas. Use a guia Exclusão para tratar uma duplicidade sem apagar o registro.';
    gridEditor[key] = previous;
    return;
  }

  if (!input) {
    if (slot.record?.id) {
      await deleteBatida(Number(slot.record.id));
      gridStatus.value = `Batida removida em ${date} (${slot.tipo}).`;
      await carregarCartao();
    }
    gridEditor[key] = '';
    return;
  }

  if (!isValidHourInput(input)) {
    error.value = 'Informe a hora no formato HH:MM.';
    gridEditor[key] = previous;
    return;
  }

  if (input === previous) {
    gridEditor[key] = input;
    return;
  }

  if (!funcionarioIdNumero.value) {
    error.value = 'Selecione um funcionário para editar a grade inline.';
    gridEditor[key] = previous;
    return;
  }

  gridSaving[key] = true;
  try {
    await saveBatida({
      id: slot.record?.id,
      funcionario_id: Number(slot.record?.funcionario_id || funcionarioIdNumero.value),
      data_referencia: date,
      hora: input,
      tipo: slot.record?.tipo || slot.tipo,
      equipamento_id: slot.record?.equipamento_id ? Number(slot.record.equipamento_id) : null,
      justificativa_id: slot.record?.justificativa_id ? Number(slot.record.justificativa_id) : null,
      observacao: slot.record?.observacao || 'Edição inline no cartão de ponto',
      manual_ajuste: slot.record ? (Number(slot.record.manual_ajuste) === 1 || slot.record.manual_ajuste === true) : true,
      validado: slot.record ? (Number(slot.record.validado) === 1 || slot.record.validado === true) : true,
      origem: slot.record?.origem || 'cartao_inline',
      nsr: slot.record?.nsr || '',
    });
    gridEditor[key] = input;
    gridStatus.value = `Batida ${slot.record?.id ? 'atualizada' : 'incluída'} em ${date} (${slot.tipo}) às ${input}.`;
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao salvar célula da grade.';
    gridEditor[key] = previous;
  } finally {
    gridSaving[key] = false;
  }
}

async function clearGridCell(date: string, slotIndex: number) {
  const slot = getGridSlot(date, slotIndex);
  if (!slot) return;
  if (slot.record && isProtectedPunch(slot.record)) {
    error.value = 'Esta marcação é oficial e imutável. Use a guia Exclusão para classificá-la como duplicidade.';
    gridEditor[slot.key] = String(slot.record.hora || '').slice(0, 5);
    return;
  }
  if (slot.record?.id) {
    await deleteBatida(Number(slot.record.id));
    gridStatus.value = `Batida removida em ${date} (${slot.tipo}).`;
    await carregarCartao();
  } else {
    gridEditor[slot.key] = '';
  }
}

async function onGridCellKeydown(event: KeyboardEvent, date: string, slotIndex: number) {
  const rows = dailyGridRows.value;
  const rowIndex = rows.findIndex((item) => item.isoDate === date);
  if (event.key === 'Enter') {
    event.preventDefault();
    await commitGridCell(date, slotIndex);
    const nextSlot = slotIndex < 5 ? slotIndex + 1 : 0;
    const nextDate = slotIndex < 5 ? date : (rows[rowIndex + 1]?.isoDate || date);
    focusGridCell(nextDate, nextSlot);
    return;
  }
  if (event.key === 'Delete') {
    event.preventDefault();
    await clearGridCell(date, slotIndex);
    focusGridCell(date, slotIndex);
    return;
  }
  if (event.key === 'ArrowRight') {
    event.preventDefault();
    focusGridCell(date, Math.min(5, slotIndex + 1));
    return;
  }
  if (event.key === 'ArrowLeft') {
    event.preventDefault();
    focusGridCell(date, Math.max(0, slotIndex - 1));
    return;
  }
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    focusGridCell(rows[Math.min(rows.length - 1, rowIndex + 1)]?.isoDate || date, slotIndex);
    return;
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault();
    focusGridCell(rows[Math.max(0, rowIndex - 1)]?.isoDate || date, slotIndex);
  }
}

function rowBadgeClass(row: GenericRecord) {
  const date = String(row.data_referencia || "");
  const resumo = apuracaoResumo.value?.rows.find((item) => item.data === date);
  return resumo?.inconsistente ? "row-highlight-warning" : ((resumo?.ocorrencias || []).length ? "row-highlight-info" : "");
}

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
  syncPeriodFilters();
  loading.value = true;
  error.value = "";
  message.value = "";
  try {
    const [rowsBatida, rowsBatidaComInativas, rowsOcorrencia, apuracao] = await Promise.all([
      listBatidas({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioIdNumero.value,
        dataInicial: filtros.dataInicial || null,
        dataFinal: filtros.dataFinal || null,
      }),
      listBatidas({
        empresaId: session.activeCompanyId ?? null,
        funcionarioId: funcionarioIdNumero.value,
        dataInicial: filtros.dataInicial || null,
        dataFinal: filtros.dataFinal || null,
        incluirInativas: true,
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
        competenciaAno: filtros.modoPeriodo === "competencia" ? Number(filtros.competenciaAno) : null,
        competenciaMes: filtros.modoPeriodo === "competencia" ? Number(filtros.competenciaMes) : null,
        dataInicial: filtros.modoPeriodo === "competencia" ? null : (filtros.dataInicial || null),
        dataFinal: filtros.modoPeriodo === "competencia" ? null : (filtros.dataFinal || null),
      }),
    ]);
    batidas.value = rowsBatida;
    batidasInativas.value = rowsBatidaComInativas.filter((row) => Number(row.ativo ?? 1) === 0);
    ocorrencias.value = rowsOcorrencia;
    syncGridEditorFromData();
    apuracaoResumo.value = apuracao;
    reportHtml.value = buildCartaoHtml();
    const availableDates = apuracao.rows.map((item) => item.data);
    if (!selectedDate.value || !availableDates.includes(selectedDate.value)) {
      selectedDate.value = availableDates[0] || filtros.dataInicial;
    }
  } catch (err) {
    batidas.value = [];
    batidasInativas.value = [];
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
  const parts = value.split(":").map((item) => Number(item));
  const hh = parts[0];
  const mm = parts[1];
  const ss = parts[2] || 0;
  if (!Number.isFinite(hh) || !Number.isFinite(mm) || !Number.isFinite(ss)) return null;
  return hh * 60 + mm + (ss / 60);
}

function minutesToHHMM(value: number): string {
  const safe = Math.max(0, Number(value || 0));
  const hh = Math.floor(safe / 60).toString().padStart(2, "0");
  const mm = Math.floor(safe % 60).toString().padStart(2, "0");
  return `${hh}:${mm}`;
}

function minutesToSignedHHMM(value: number): string {
  const numeric = Number(value || 0);
  const sign = numeric < 0 ? "-" : "";
  const absolute = Math.abs(numeric);
  const hh = Math.floor(absolute / 60).toString().padStart(2, "0");
  const mm = Math.floor(absolute % 60).toString().padStart(2, "0");
  return `${sign}${hh}:${mm}`;
}

function hhmmToMinutes(value: string): number {
  const clean = String(value || "00:00").trim();
  const sign = clean.startsWith("-") ? -1 : 1;
  const normalized = clean.replace(/^[+-]/, "");
  const [h, m] = normalized.split(":").map((part) => Number(part || 0));
  return sign * ((Number.isFinite(h) ? h : 0) * 60 + (Number.isFinite(m) ? m : 0));
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

function buildDailyRows(summary: ApuracaoResumo | null, initial: Date, final: Date): {
  rows: DailyReportRow[];
  totals: Record<string, number>;
} {
  const apuracaoByDate = new Map<string, ApuracaoDia>();
  for (const row of summary?.rows || []) {
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
    trabalhado: 0,
    esperado: 0,
    saldo: 0,
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
    totals.trabalhado += trabalhado;
    totals.esperado += esperado;
    totals.saldo += saldo;

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

  totals.extra = saldoCredorConsolidado(totals.saldo);
  totals.falta = saldoDevedorConsolidado(totals.saldo);

  return { rows, totals };
}


function isCartaoModeloPaisagem(modelo = filtros.modeloRelatorio): boolean {
  return ["folha_interjornada", "folha_com_he", "folha_completa"].includes(modelo);
}

function cartaoPrintCss(modelo = filtros.modeloRelatorio): string {
  const isLandscape = isCartaoModeloPaisagem(modelo);
  const margin = "6mm";
  const orientation = isLandscape ? "landscape" : "portrait";
  const pageWidth = isLandscape ? "297mm" : "210mm";
  const pageHeight = isLandscape ? "210mm" : "297mm";
  const bodyFontSize = isLandscape ? "8.5px" : "9px";
  const tableFontSize = isLandscape ? "7.4px" : "8.2px";
  const cellPadding = isLandscape ? "1.6px 2.4px" : "2px 3px";
  const titleSize = isLandscape ? "14px" : "15px";
  const signatureMargin = isLandscape ? "10px" : "12px";

  return `
      @page{size:A4 ${orientation};margin:${margin}}
      *,*::before,*::after{box-sizing:border-box}
      html,body{min-height:100%}
      body{font-family:Consolas,monospace;margin:0;color:#111;font-size:${bodyFontSize}}
      .report-page{position:relative;background:#fff}
      .head{display:grid;grid-template-columns:1fr auto;gap:6px;align-items:end;border-bottom:1px solid #333;padding-bottom:3px}
      h1{margin:0;font-size:${titleSize};line-height:1.1}
      .meta{font-size:${isLandscape ? "8px" : "8.5px"};line-height:1.15}
      .page-number{align-self:start;text-align:right;white-space:nowrap}
      table{width:100%;border-collapse:collapse;font-size:${tableFontSize};margin-top:4px;table-layout:fixed}
      th,td{border:1px solid #808080;padding:${cellPadding};text-align:left;vertical-align:top;word-break:break-word;line-height:1.12}
      thead th{background:#ececec}
      tr{break-inside:avoid;page-break-inside:avoid}
      .tot{font-weight:700;background:#f5f5f5}
      .sign{margin-top:${signatureMargin};display:grid;grid-template-columns:1fr 1fr;gap:18px;text-align:center}
      .line{border-top:1px solid #333;padding-top:3px}
      .summary-grid{display:grid;grid-template-columns:repeat(5,1fr);gap:4px;margin-top:5px}
      .summary-box{border:1px solid #666;padding:3px;text-align:center}
      .legend{font-size:${isLandscape ? "7px" : "7.5px"};margin-top:4px}
      svg{max-width:${isLandscape ? "112px" : "120px"};height:auto}
      @media screen{
        html{background:#e9eef5}
        body{display:flex;flex-direction:column;align-items:center;gap:24px;padding:24px;background:#e9eef5;overflow:auto}
        .report-page{width:${pageWidth};min-height:${pageHeight};padding:${margin};flex:0 0 auto;box-shadow:0 18px 48px rgba(15,23,42,.18);border:1px solid #d8dee8}
      }
      @media print{
        html,body{min-height:0;background:#fff}
        body{display:block;padding:0;overflow:visible;-webkit-print-color-adjust:exact;print-color-adjust:exact}
        .report-page{width:auto;min-height:0;margin:0;padding:0;border:0;box-shadow:none;break-after:page;page-break-after:always}
        .report-page:last-child{break-after:auto;page-break-after:auto}
      }
      ${isLandscape ? `
        @media screen and (max-width:1200px){.report-page{zoom:.82}}
        @media screen and (max-width:900px){.report-page{zoom:.72}}
        @media screen and (max-width:720px){.report-page{zoom:.55}}
        @media screen and (max-width:560px){.report-page{zoom:.42}}
        @media screen and (max-width:420px){.report-page{zoom:.31}}
      ` : `
        @media screen and (max-width:840px){.report-page{zoom:.90}}
        @media screen and (max-width:720px){.report-page{zoom:.80}}
        @media screen and (max-width:560px){.report-page{zoom:.62}}
        @media screen and (max-width:420px){.report-page{zoom:.45}}
      `}
    `;
}

function buildCartaoHtmlFromSummary(summary: ApuracaoResumo | null, employeeName: string, dataInicial: string, dataFinal: string): string {
  if (!dataInicial || !dataFinal) return "";
  const initial = new Date(`${dataInicial}T00:00:00`);
  const final = new Date(`${dataFinal}T00:00:00`);
  if (Number.isNaN(initial.getTime()) || Number.isNaN(final.getTime()) || initial > final) return "";

  const { rows: dailyRows, totals } = buildDailyRows(summary, initial, final);

  const logoSvg = `<svg xmlns='http://www.w3.org/2000/svg' width='180' height='44' viewBox='0 0 420 100'><rect width='100' height='100' rx='18' fill='#1d4ed8'/><path d='M50 24v28l18-14' stroke='#fff' stroke-width='8' stroke-linecap='round'/><circle cx='50' cy='50' r='32' fill='none' stroke='rgba(255,255,255,.35)' stroke-width='8'/><text x='122' y='45' font-family='Segoe UI, Arial' font-size='28' font-weight='700' fill='#1f2937'>Ponto Manager</text><text x='122' y='74' font-family='Segoe UI, Arial' font-size='14' fill='#64748b'>jornada • rep • banco de horas</text></svg>`;
  function buildTableForModel(rows: DailyReportRow[], includeTotals: boolean): string {
    const totalsRowByModel: Record<string, string> = {
      cartao_ponto: `<tr class="tot"><td colspan="3">TOTAIS</td><td>${minutesToHHMM(totals.trabalhado)}</td><td>${minutesToHHMM(totals.esperado)}</td><td>${minutesToSignedHHMM(totals.saldo)}</td><td>-</td><td>-</td></tr>`,
      folha_resumida: `<tr class="tot"><td colspan="3">TOTAIS</td><td>${minutesToHHMM(totals.trabalhado)}</td></tr>`,
      folha_interjornada: `<tr class="tot"><td colspan="7">TOTAIS</td><td>${minutesToHHMM(totals.trabalhado)}</td></tr>`,
      folha_com_he: `<tr class="tot"><td colspan="11">TOTAIS</td><td>${minutesToHHMM(totals.trabalhado)}</td><td>${minutesToHHMM(totals.atraso)}</td></tr>`,
      folha_completa: `<tr class="tot"><td colspan="11">TOTAIS</td><td>${minutesToHHMM(totals.trabalhado)}</td><td>${minutesToHHMM(totals.atraso)}</td></tr>`,
    };
    const tableByModel: Record<string, string> = {
      cartao_ponto: `
        <thead><tr><th>Data</th><th>Dia semana</th><th>Marcações do dia</th><th>Total trabalhado</th><th>Jornada esperada</th><th>Saldo do dia</th><th>Ocorrência</th><th>Observação</th></tr></thead>
        <tbody>${rows.map((r) => `<tr><td>${r.day}</td><td>${r.dayLabel}</td><td>${[r.ent1, r.sai1, r.ent2, r.sai2, r.ent3, r.sai3].filter((p) => p && p !== "Folga").join(" | ") || "-"}</td><td>${r.hTrabalhadas}</td><td>${r.previsto}</td><td>${minutesToSignedHHMM(hhmmToMinutes(r.extra) - hhmmToMinutes(r.falta))}</td><td>${r.ocorrencias || "Normal"}</td><td>-</td></tr>`).join("")}${includeTotals ? totalsRowByModel.cartao_ponto : ""}</tbody>`,
      folha_resumida: `
        <thead><tr><th>Data</th><th>Previsto</th><th>Realizado</th><th>H. trab.</th></tr></thead>
        <tbody>${rows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.realizado}</td><td>${r.hTrabalhadas}</td></tr>`).join("")}${includeTotals ? totalsRowByModel.folha_resumida : ""}</tbody>`,
      folha_interjornada: `
        <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. trab.</th></tr></thead>
        <tbody>${rows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTrabalhadas}</td></tr>`).join("")}${includeTotals ? totalsRowByModel.folha_interjornada : ""}</tbody>`,
      folha_com_he: `
        <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. totais</th><th>HE diurnas</th><th>HE noturnas</th><th>HE total</th><th>H. trab.</th><th>Atraso</th></tr></thead>
        <tbody>${rows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTotais}</td><td>${r.heDiurnas}</td><td>${r.heNoturnas}</td><td>${r.heTotal}</td><td>${r.hTrabalhadas}</td><td>${r.atraso}</td></tr>`).join("")}${includeTotals ? totalsRowByModel.folha_com_he : ""}</tbody>`,
      folha_completa: `
        <thead><tr><th>Data</th><th>Previsto</th><th>Inter-jornada</th><th>Realizado</th><th>Intra-jornada</th><th>H. diurnas</th><th>H. noturnas</th><th>H. totais</th><th>HE diurnas</th><th>HE noturnas</th><th>HE total</th><th>H. trab.</th><th>Atraso</th></tr></thead>
        <tbody>${rows.map((r) => `<tr><td>${r.day} - ${r.dayLabel}</td><td>${r.previsto}</td><td>${r.interJornada}</td><td>${r.realizado}</td><td>${r.intraJornada}</td><td>${r.hDiurnas}</td><td>${r.hNoturnas}</td><td>${r.hTotais}</td><td>${r.heDiurnas}</td><td>${r.heNoturnas}</td><td>${r.heTotal}</td><td>${r.hTrabalhadas}</td><td>${r.atraso}</td></tr>`).join("")}${includeTotals ? totalsRowByModel.folha_completa : ""}</tbody>`,
    };
    return tableByModel[filtros.modeloRelatorio] || tableByModel.folha_resumida;
  }

  const summaryByModel = filtros.modeloRelatorio === "folha_completa" ? `
    <div class="summary-grid">
      <div class="summary-box"><strong>Total atrasos</strong><div>${minutesToHHMM(totals.atraso)}</div></div>
      <div class="summary-box"><strong>Total horas noturnas</strong><div>${minutesToHHMM(totals.noturno)}</div></div>
      <div class="summary-box"><strong>Total H.E. acumuladas</strong><div>${minutesToHHMM(totals.extra)}</div></div>
      <div class="summary-box"><strong>Total horas faltantes</strong><div>${minutesToHHMM(totals.falta)}</div></div>
      <div class="summary-box"><strong>Total banco de horas</strong><div>${minutesToSignedHHMM(totals.saldo)}</div></div>
    </div>
  ` : "";

  const rowsPerPageByModel: Record<string, number> = {
    cartao_ponto: 31,
    folha_resumida: 31,
    folha_interjornada: 30,
    folha_com_he: 27,
    folha_completa: 25,
  };
  const rowsPerPage = rowsPerPageByModel[filtros.modeloRelatorio] || 31;
  const pageRows = dailyRows.length
    ? Array.from({ length: Math.ceil(dailyRows.length / rowsPerPage) }, (_, index) => dailyRows.slice(index * rowsPerPage, (index + 1) * rowsPerPage))
    : [[] as DailyReportRow[]];
  const modelLabel = filtros.modeloRelatorio.replace(/_/g, " ").toUpperCase();
  const emittedAt = new Date().toLocaleDateString("pt-BR");
  const pagesHtml = pageRows.map((rows, pageIndex) => {
    const isLastPage = pageIndex === pageRows.length - 1;
    return `
      <main class="report-page report-page--${isCartaoModeloPaisagem(filtros.modeloRelatorio) ? "landscape" : "portrait"}" data-page="${pageIndex + 1}" data-page-count="${pageRows.length}">
        <div class="head">
          <div>
            <div>${logoSvg}</div>
            <h1>CARTÃO PONTO — ${modelLabel}</h1>
            <div class="meta">Período: ${dataInicial.split("-").reverse().join("/")} até ${dataFinal.split("-").reverse().join("/")}</div>
            <div class="meta">Competência/visão: ${periodoLabel.value}</div>
            <div class="meta">Empresa: ${session.activeCompanyName || "-"}</div>
            <div class="meta">Colaborador: ${employeeName}</div>
          </div>
          <div class="meta page-number">Emitido em ${emittedAt}<br>Página ${pageIndex + 1} de ${pageRows.length}</div>
        </div>
        <table>${buildTableForModel(rows, isLastPage)}</table>
        ${isLastPage ? summaryByModel : ""}
        ${isLastPage
          ? `<p class="legend"><strong>Legenda:</strong> Total H.E. acumuladas e total horas faltantes são demonstrados pelo saldo líquido consolidado do período, sem exibir crédito e débito simultaneamente para o mesmo colaborador.</p><div class="sign"><div class="line">${employeeName}</div><div class="line">${empresaResponsavel.value}</div></div>`
          : `<p class="legend page-continuation">Continua na página seguinte.</p>`}
      </main>`;
  }).join("");

  return `<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><title>Cartão de ponto</title>
    <style>${cartaoPrintCss(filtros.modeloRelatorio)}</style></head>
    <body>${pagesHtml}</body></html>`;
}

function buildCartaoHtml(): string {
  const periodo = periodoAtual();
  return buildCartaoHtmlFromSummary(apuracaoResumo.value, funcionarioNomeSelecionado.value, periodo.dataInicial, periodo.dataFinal);
}

function sanitizeFilePart(value: string) {
  return value
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[^a-zA-Z0-9_-]+/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_|_$/g, "") || "relatorio";
}

function extractPrintableBody(html: string): string {
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
  return bodyMatch?.[1] || html;
}

function buildAllCardsHtml(cards: { employeeName: string; html: string }[]) {
  const content = cards
    .map((card) => `<section class="card-document" data-employee="${card.employeeName}">${extractPrintableBody(card.html)}</section>`)
    .join("");

  return `<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><title>Cartões da competência</title>
    <style>
      ${cartaoPrintCss(filtros.modeloRelatorio)}
      .card-document{display:contents}
      .card-document:not(:last-child) .report-page:last-child{break-after:page;page-break-after:always}
    </style></head><body>${content}</body></html>`;
}

async function generateCompetenciaCardsHtml() {
  syncPeriodFilters();
  const periodo = periodoAtual();
  const targetEmployees = employeeOptions.value.filter((item) => Number(item.id) > 0);
  const cards: { employeeName: string; html: string }[] = [];

  for (const employee of targetEmployees) {
    const summary = await apurarPeriodo({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: Number(employee.id),
      competenciaAno: Number(filtros.competenciaAno),
      competenciaMes: Number(filtros.competenciaMes),
      dataInicial: null,
      dataFinal: null,
    });
    cards.push({
      employeeName: employee.label,
      html: buildCartaoHtmlFromSummary(summary, employee.label, periodo.dataInicial, periodo.dataFinal),
    });
  }

  return buildAllCardsHtml(cards);
}

async function openPrintFrame(html: string) {
  const periodo = periodoAtual();
  await printHtmlExternally(html, {
    fileName: `cartao_ponto_${sanitizeFilePart(funcionarioNomeSelecionado.value || "competencia")}_${periodo.dataInicial}_${periodo.dataFinal}.html`,
  });
}

async function imprimirTodosCompetencia() {
  if (filtros.modoPeriodo !== "competencia") {
    error.value = "Selecione o modo Competência para imprimir todos os cartões do mês.";
    return;
  }
  printingAllCompetencia.value = true;
  error.value = "";
  message.value = "";
  try {
    const html = await generateCompetenciaCardsHtml();
    await openPrintFrame(html);
    const fileName = `cartoes_ponto_competencia_${String(filtros.competenciaMes).padStart(2, "0")}_${filtros.competenciaAno}.pdf`;
    await registerGeneratedReport({
      descricao: "Cartões de ponto da competência",
      tipoRelatorio: "cartao_ponto_lote",
      origemRotina: "cartao_ponto",
      formato: "PDF",
      fileName,
      mimeType: "application/pdf",
      competencia: periodoLabel.value,
      funcionarioId: null,
      funcionarioNome: "Todos os colaboradores",
      usuarioLogin: session.user?.login || null,
      detalhado: true,
      status: "GERADO",
      contentBase64: toBase64Utf8(html),
    });
    message.value = "Impressão de todos os cartões da competência iniciada e registrada em Relatórios Gerados.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao imprimir todos os cartões da competência.";
  } finally {
    printingAllCompetencia.value = false;
  }
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
    const periodo = periodoAtual();
    const fileName = `cartao_ponto_${sanitizeFilePart(funcionarioNomeSelecionado.value)}_${periodo.dataInicial}_${periodo.dataFinal}.html`;
    await saveWithDialog(reportHtml.value, fileName, "text/html");
    await registerGeneratedReport({
      descricao: "Cartão de ponto",
      tipoRelatorio: "cartao_ponto",
      origemRotina: "cartao_ponto",
      formato: "HTML",
      fileName,
      mimeType: "text/html",
      competencia: periodoLabel.value,
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
    const periodo = periodoAtual();
    const fileName = `cartao_ponto_${sanitizeFilePart(funcionarioNomeSelecionado.value)}_${periodo.dataInicial}_${periodo.dataFinal}.xls`;
    await saveWithDialog(reportHtml.value, fileName, "application/vnd.ms-excel");
    await registerGeneratedReport({
      descricao: "Cartão de ponto",
      tipoRelatorio: "cartao_ponto",
      origemRotina: "cartao_ponto",
      formato: "EXCEL",
      fileName,
      mimeType: "application/vnd.ms-excel",
      competencia: periodoLabel.value,
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

async function imprimirOuSalvarPdf() {
  reportHtml.value = buildCartaoHtml();
  if (!reportHtml.value) {
    error.value = "Gere o cartão antes de imprimir/salvar PDF.";
    return;
  }

  try {
    await openPrintFrame(reportHtml.value);
    const periodo = periodoAtual();
    await registerGeneratedReport({
      descricao: "Cartão de ponto (impressão/PDF)",
      tipoRelatorio: "cartao_ponto",
      origemRotina: "cartao_ponto",
      formato: "PDF",
      fileName: `cartao_ponto_${sanitizeFilePart(funcionarioNomeSelecionado.value)}_${periodo.dataInicial}_${periodo.dataFinal}.pdf`,
      mimeType: "application/pdf",
      competencia: periodoLabel.value,
      funcionarioId: funcionarioIdNumero.value,
      funcionarioNome: funcionarioNomeSelecionado.value,
      usuarioLogin: session.user?.login || null,
      detalhado: true,
      status: "GERADO",
      contentBase64: toBase64Utf8(reportHtml.value),
    });
    message.value = "Impressão iniciada. O relatório também foi registrado em Relatórios Gerados.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao imprimir ou salvar PDF.";
  }
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
  if (isProtectedPunch(row)) {
    message.value = 'Marcação oficial protegida: os dados AFD/REP/Connector são somente leitura.';
    showSplashInfo(message.value);
    return;
  }
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
  if (isProtectedPunch(row)) {
    error.value = 'Marcações oficiais não podem ser excluídas. Classifique a repetição como duplicidade na guia Exclusão.';
    showSplashError(error.value);
    return;
  }
  if (!row.id || !confirm("Remover esta batida?")) return;
  try {
    await deleteBatida(Number(row.id));
    message.value = "Batida removida com sucesso.";
    showSplashSuccess(message.value);
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover batida.";
    showSplashError(error.value);
  }
}

async function moverBatida(row: GenericRecord, direction: -1 | 1) {
  if (isProtectedPunch(row)) {
    error.value = 'Marcações oficiais não podem ter o horário ajustado.';
    showSplashError(error.value);
    return;
  }
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

watch(() => [filtros.modoPeriodo, filtros.competenciaMes, filtros.competenciaAno], () => {
  if (filtros.modoPeriodo === "competencia") {
    syncPeriodFilters();
  }
});

watch(() => filtros.modeloRelatorio, () => {
  if (apuracaoResumo.value) {
    reportHtml.value = buildCartaoHtml();
  }
});

watch(dailyGridRows, (rows) => {
  if (!rows.length) {
    selectedDate.value = filtros.dataInicial;
    return;
  }
  if (!rows.some((item) => item.isoDate === selectedDate.value)) {
    selectedDate.value = rows[0].isoDate;
  }
  const validKeys = new Set(rows.flatMap((row) => Array.from({ length: 6 }, (_, idx) => `${row.isoDate}:${idx}`)));
  Object.keys(gridEditor).forEach((key) => {
    if (!validKeys.has(key)) delete gridEditor[key];
  });
}, { immediate: true });

onMounted(async () => {
  syncPeriodFilters();
  await carregarBase();
  await carregarCartao();
});
</script>

<template>
  <div class="grid page-gap cartao-vb6-page">
    <AppPageTitleBar class="cartao-page-titlebar" title="Cartão de ponto" subtitle="Edição operacional e pré-visualização fiel do documento impresso em modos independentes." icon="timeCard">
      <template #actions>
        <div class="cartao-titlebar-actions">
          <nav class="cartao-view-tabs" role="tablist" aria-label="Modo de visualização do cartão de ponto">
            <button
              id="cartao-tab-edicao"
              type="button"
              role="tab"
              title="Abrir a edição operacional do cartão"
              :aria-selected="activeView === 'edicao'"
              aria-controls="cartao-panel-edicao"
              :class="{ active: activeView === 'edicao' }"
              @click="selectView('edicao')"
            >
              Editar cartão
            </button>
            <button
              id="cartao-tab-preview"
              type="button"
              role="tab"
              title="Visualizar o documento exatamente como será impresso"
              :aria-selected="activeView === 'previsualizacao'"
              aria-controls="cartao-panel-preview"
              :class="{ active: activeView === 'previsualizacao' }"
              @click="selectView('previsualizacao')"
            >
              Pré-visualizar
            </button>
          </nav>
          <div class="cartao-document-actions">
            <button class="secondary titlebar-action" :disabled="loading" @click="carregarCartao">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
            <button class="secondary titlebar-action" @click="exportarHtml">Exportar HTML</button>
            <button class="secondary titlebar-action" @click="exportarExcel">Exportar Excel</button>
            <button class="secondary titlebar-action" :disabled="printingAllCompetencia || filtros.modoPeriodo !== 'competencia'" @click="imprimirTodosCompetencia">{{ printingAllCompetencia ? 'Preparando lote...' : 'Imprimir competência' }}</button>
            <button class="primary titlebar-action" @click="imprimirOuSalvarPdf">Imprimir / Salvar PDF</button>
          </div>
        </div>
      </template>
    </AppPageTitleBar>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-if="message" class="alert success">{{ message }}</div>

    <BaseFilterBar class="cartao-filter-card" title="Filtros do cartão" description="A mesma seleção é preservada nos modos de edição, pré-visualização e impressão." density="compact" :loading="loading">
        <div class="field filter-field--wide">
          <label>Funcionário</label>
          <select v-model="filtros.funcionarioId">
            <option value="">Todos</option>
            <option v-for="item in employeeOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field filter-field--status">
          <label>Período</label>
          <select v-model="filtros.modoPeriodo">
            <option value="competencia">Competência</option>
            <option value="intervalo">Intervalo de datas</option>
          </select>
        </div>
        <div v-if="filtros.modoPeriodo === 'competencia'" class="field filter-field--compact">
          <label>Competência</label>
          <div class="inline-grid compact-inline-grid">
            <input v-model.number="filtros.competenciaMes" type="number" min="1" max="12" />
            <input v-model.number="filtros.competenciaAno" type="number" min="2020" max="2100" />
          </div>
        </div>
        <template v-else>
          <div class="field filter-field--date">
            <label>Data inicial</label>
            <input v-model="filtros.dataInicial" type="date" />
          </div>
          <div class="field filter-field--date">
            <label>Data final</label>
            <input v-model="filtros.dataFinal" type="date" />
          </div>
        </template>
        <div class="field filter-field--wide">
          <label>Modelo do relatório</label>
          <select v-model="filtros.modeloRelatorio">
            <option value="cartao_ponto">0) Cartão de ponto (padrão)</option>
            <option value="folha_resumida">1) Folha resumida</option>
            <option value="folha_interjornada">2) Folha com inter/intra jornada</option>
            <option value="folha_com_he">3) Folha com HE e atrasos</option>
            <option value="folha_completa">4) Folha completa com resumos</option>
          </select>
        </div>
      <template #actions>
        <button class="secondary" type="button" :disabled="loading" @click="clearCartaoFilters">Limpar filtros</button>
        <button class="primary" type="button" :disabled="loading" @click="carregarCartao">Aplicar filtros</button>
      </template>
      <template #summary>
        <div class="inline-info-strip">
          <span><strong>Visão:</strong> {{ periodoLabel }}</span>
          <span><strong>Colaborador:</strong> {{ funcionarioNomeSelecionado }}</span>
          <span><strong>Dias inconsistentes:</strong> {{ inconsistenciasNoPeriodo }}</span>
          <span><strong>Dias com ocorrência:</strong> {{ diasComOcorrenciaNoPeriodo }}</span>
          <span v-if="activeView === 'edicao'"><strong>Dia selecionado:</strong> {{ selectedDayLabel }}</span>
        </div>
        <div v-if="activeView === 'edicao'" class="inline-info-strip subtle">
          <span><strong>Operação inline:</strong> Enter salva e avança, Del remove, setas navegam entre células.</span>
          <span>{{ gridStatus }}</span>
        </div>
      </template>
    </BaseFilterBar>

    <div
      v-if="activeView === 'previsualizacao'"
      id="cartao-panel-preview"
      class="card cartao-preview-card preview-only"
      :data-orientation="previewOrientation"
      role="tabpanel"
      aria-labelledby="cartao-tab-preview"
    >
      <div v-if="loading" class="cartao-preview-state" role="status" aria-live="polite">
        <span class="cartao-preview-spinner" aria-hidden="true"></span>
        <strong>Preparando a pré-visualização do relatório...</strong>
      </div>
      <iframe
        v-else-if="reportHtml"
        class="report-frame"
        title="Pré-visualização do cartão de ponto"
        :srcdoc="reportHtml"
        sandbox=""
        referrerpolicy="no-referrer"
      ></iframe>
      <div v-else class="cartao-preview-state" role="status">
        <strong>Não há relatório disponível para os filtros selecionados.</strong>
        <span>Revise o funcionário e o período e aplique os filtros novamente.</span>
      </div>
    </div>

    <div
      v-else
      id="cartao-panel-edicao"
      class="cartao-vb6-shell"
      :class="{ 'sidebar-collapsed': sidePanelCollapsed }"
      role="tabpanel"
      aria-labelledby="cartao-tab-edicao"
    >
      <div class="card cartao-vb6-grid-panel table-wrap">
        <div class="vb6-group-header">
          <h3>Grade diária do cartão</h3>
          <div class="actions compact-actions">
            <button class="secondary" @click="openNovaBatida(selectedDate)">Nova marcação</button>
            <button class="secondary" @click="openNovaOcorrencia(selectedDate)">Nova ocorrência</button>
          </div>
        </div>
        <table class="quick-table table-compact vb6-main-grid">
          <thead>
            <tr>
              <th>Data</th>
              <th>Entrada 1</th>
              <th>Saída 1</th>
              <th>Entrada 2</th>
              <th>Saída 2</th>
              <th>Entrada 3</th>
              <th>Saída 3</th>
              <th>Comp.</th>
              <th>Folga</th>
              <th>Obs.</th>
              <th>Ação</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in dailyGridRows" :key="row.isoDate" :class="dailyRowClass(row)" @click="selectDay(row.isoDate)">
              <td class="date-cell"><strong>{{ row.day }}</strong> - {{ row.dayLabel }}</td>
              <td v-for="slot in (gridSlotsByDate[row.isoDate] || [])" :key="slot.key" class="grid-cell-editable">
                <input
                  :ref="(el) => setGridCellRef(slot.key, el)"
                  v-model="gridEditor[slot.key]"
                  class="grid-time-input"
                  maxlength="5"
                  placeholder="--:--"
                  :disabled="gridSaving[slot.key] || !funcionarioIdNumero || isProtectedPunch(slot.record)"
                  :title="isProtectedPunch(slot.record) ? 'Marcação oficial protegida contra alteração' : 'Enter salva; Delete remove a marcação manual'"
                  @focus="selectDay(row.isoDate)"
                  @blur="commitGridCell(row.isoDate, slot.slotIndex)"
                  @keydown="onGridCellKeydown($event, row.isoDate, slot.slotIndex)"
                />
              </td>
              <td>{{ row.expectedMinutes > 0 ? 'x' : '' }}</td>
              <td>{{ row.expectedMinutes === 0 ? 'x' : '' }}</td>
              <td class="obs-cell">{{ row.mensagens[0] || (row.ocorrenciasCount > 0 ? `${row.ocorrenciasCount} ocorrência(s)` : (row.inconsistente ? 'Revisar' : '-')) }}</td>
              <td class="action-cell">
                <div class="actions compact-actions">
                  <button class="secondary" @click.stop="openNovaBatida(row.isoDate)">+ Batida</button>
                  <button class="secondary" @click.stop="openNovaOcorrencia(row.isoDate)">+ Ocor.</button>
                </div>
              </td>
            </tr>
            <tr v-if="!dailyGridRows.length">
              <td colspan="11" class="empty-cell">Nenhum dia disponível para o período informado.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="cartao-vb6-side sticky-card" :class="{ collapsed: sidePanelCollapsed }">
        <button class="side-collapse-btn" type="button" @click="toggleSidebar" :title="sidePanelCollapsed ? 'Expandir sidebar' : 'Recolher sidebar'">
          {{ sidePanelCollapsed ? '◀' : '▶' }}
        </button>
        <template v-if="!sidePanelCollapsed">
          <div class="side-content">
            <div class="side-tabs">
              <button class="side-tab-btn" :class="{ active: activeSideTab === 'marcacoes' }" @click="activeSideTab = 'marcacoes'">Marcações</button>
              <button class="side-tab-btn" :class="{ active: activeSideTab === 'ocorrencias' }" @click="activeSideTab = 'ocorrencias'">Ocorrências</button>
              <button class="side-tab-btn" :class="{ active: activeSideTab === 'smart' }" @click="activeSideTab = 'smart'">Smart</button>
              <button class="side-tab-btn" :class="{ active: activeSideTab === 'exclusao' }" @click="activeSideTab = 'exclusao'">Exclusão</button>
            </div>
            <div class="card table-wrap card-tight side-panel">
            <div v-if="activeSideTab === 'marcacoes'">
              <div class="vb6-group-header">
                <h3>Marcações do dia selecionado</h3>
                <button class="secondary" @click="openNovaBatida(selectedDate)">Nova marcação</button>
              </div>
              <table class="quick-table table-compact">
                <thead><tr><th>Data</th><th>Hora</th><th>Tipo</th><th>Origem</th><th>Just.</th><th>Ação</th></tr></thead>
                <tbody>
                  <tr v-for="row in batidasSelecionadas" :key="String(row.id)" :class="rowBadgeClass(row)">
                    <td>{{ row.data_referencia }}</td><td>{{ row.hora }}</td><td>{{ row.tipo }}</td>
                    <td>
                      <span v-if="isProtectedPunch(row)" class="official-punch-badge" title="Registro oficial imutável">Protegida</span>
                      <span v-else>{{ row.origem || '-' }}</span>
                    </td><td>{{ row.justificativa_nome || '-' }}</td>
                    <td><div class="actions compact-actions">
                      <button class="secondary icon-btn" title="Editar" :disabled="isProtectedPunch(row)" @click="editarBatida(row)">✎</button>
                      <button class="secondary action-mini" :disabled="isProtectedPunch(row)" @click="moverBatida(row, -1)">-1m</button>
                      <button class="secondary action-mini" :disabled="isProtectedPunch(row)" @click="moverBatida(row, 1)">+1m</button>
                      <button class="danger icon-btn" title="Remover marcação manual" :disabled="isProtectedPunch(row)" @click="removerBatida(row)">🗑</button>
                    </div></td>
                  </tr>
                  <tr v-if="!batidasSelecionadas.length"><td colspan="6" class="empty-cell">Nenhuma marcação encontrada para o dia selecionado.</td></tr>
                </tbody>
              </table>
            </div>

            <div v-else-if="activeSideTab === 'ocorrencias'">
              <div class="vb6-group-header">
                <h3>Ocorrências do dia selecionado</h3>
                <button class="secondary" @click="openNovaOcorrencia(selectedDate)">Nova ocorrência</button>
              </div>
              <table class="quick-table table-compact">
                <thead><tr><th>Data</th><th>Tipo</th><th>Justificativa</th><th>Abono</th><th>Obs.</th><th>Ação</th></tr></thead>
                <tbody>
                  <tr v-for="row in ocorrenciasSelecionadas" :key="String(row.id)" :class="rowBadgeClass(row)">
                    <td>{{ row.data_referencia }}</td><td>{{ row.tipo }}</td><td>{{ row.justificativa_nome || '-' }}</td>
                    <td>{{ Number(row.minutos_abonados || 0) > 0 ? row.minutos_abonados : (Number(row.abonar_dia) === 1 ? 'Dia abonado' : '-') }}</td>
                    <td>{{ row.observacao || '-' }}</td>
                    <td><div class="actions compact-actions">
                      <button class="secondary icon-btn" title="Editar" @click="editarOcorrencia(row)">✎</button>
                      <button class="danger icon-btn" title="Remover" @click="removerOcorrencia(row)">🗑</button>
                    </div></td>
                  </tr>
                  <tr v-if="!ocorrenciasSelecionadas.length"><td colspan="6" class="empty-cell">Nenhuma ocorrência encontrada para o dia selecionado.</td></tr>
                </tbody>
              </table>
            </div>

            <div v-else-if="activeSideTab === 'smart'" class="vb6-group">
              <div class="vb6-group-header">
                <h3>Motor smart</h3>
                <div class="actions compact-actions">
                  <button class="secondary" :disabled="smartBusy" @click="analisarSugestoes">Analisar</button>
                  <button class="primary" :disabled="smartBusy" @click="tratarTodosAutomaticos">Tratar</button>
                </div>
              </div>
              <div class="smart-summary-grid">
                <div><strong>Esquec.</strong><span>{{ smartResumo.esquecimentos }}</span></div>
                <div><strong>Faltas</strong><span>{{ smartResumo.faltas }}</span></div>
                <div><strong>Folga</strong><span>{{ smartResumo.trocasFolga }}</span></div>
                <div><strong>Meia</strong><span>{{ smartResumo.meiasFolga }}</span></div>
              </div>
              <div class="actions compact-actions">
                <button class="secondary" :disabled="smartBusy" @click="aplicarSugestoesSelecionadas(false)">Selecionadas</button>
                <button class="secondary" :disabled="smartBusy" @click="aplicarSugestoesSelecionadas(true)">Seguras</button>
              </div>
              <div class="compact-table-wrap">
                <table class="quick-table table-compact">
                  <thead><tr><th></th><th>Data</th><th>Tipo</th><th>Batidas</th></tr></thead>
                  <tbody>
                    <tr v-for="item in smartSuggestions" :key="item.key" @click="selectDay(item.date)">
                      <td><input v-model="smartSuggestionSelection[item.key]" type="checkbox" /></td>
                      <td>{{ item.date }}</td>
                      <td><span class="badge" :class="suggestionBadgeClass(item.tipo)">{{ item.titulo }}</span></td>
                      <td>{{ item.batidas.join(' | ') || '-' }}</td>
                    </tr>
                    <tr v-if="!smartSuggestions.length"><td colspan="4" class="empty-cell">Nenhuma sugestão smart gerada para o período atual.</td></tr>
                  </tbody>
                </table>
              </div>
            </div>

            <div v-else class="vb6-group">
              <div class="vb6-group-header">
                <div>
                  <h3>Duplicidades e reativação</h3>
                  <div class="muted-text">Nenhuma marcação é apagada: a duplicidade fica oculta e auditável.</div>
                </div>
                <div class="actions compact-actions">
                  <button class="secondary" :disabled="duplicateBusy" @click="localizarDuplicidades">Localizar</button>
                  <button class="primary" :disabled="duplicateBusy" @click="excluirDuplicidadesSelecionadas">Marcar duplicidade</button>
                </div>
              </div>
              <div class="compact-table-wrap">
                <table class="quick-table table-compact">
                  <thead><tr><th></th><th>Data</th><th>Hora</th><th>Rep.</th><th>Principal</th><th>Duplicadas</th></tr></thead>
                  <tbody>
                    <tr v-for="item in duplicateCandidates" :key="item.key" @click="selectDay(item.date)">
                      <td><input v-model="duplicateSelection[item.key]" type="checkbox" /></td>
                      <td>{{ item.date }}</td><td>{{ item.horarioBase }}</td><td>{{ item.repeticoes }}</td>
                      <td>#{{ item.principalId }} · {{ item.principalOrigem }}</td><td>{{ item.duplicateIds.map((id) => `#${id}`).join(', ') }}</td>
                    </tr>
                    <tr v-if="!duplicateCandidates.length"><td colspan="6" class="empty-cell">Nenhuma duplicidade localizada para o filtro atual.</td></tr>
                  </tbody>
                </table>
              </div>
              <div class="vb6-group-header" style="margin-top: 14px">
                <h3>Batidas ocultadas</h3>
                <span class="inactive-punch-badge">{{ batidasInativas.length }} recuperável(is)</span>
              </div>
              <div class="compact-table-wrap">
                <table class="quick-table table-compact">
                  <thead><tr><th>Data</th><th>Hora</th><th>Origem</th><th>Principal</th><th>Ação</th></tr></thead>
                  <tbody>
                    <tr v-for="row in batidasInativas" :key="`inactive-${row.id}`">
                      <td>{{ row.data_referencia }}</td><td>{{ row.hora }}</td><td>{{ row.origem || '-' }}</td>
                      <td>{{ row.duplicada_de_id ? `#${row.duplicada_de_id}` : '-' }}</td>
                      <td><button class="secondary" :disabled="duplicateBusy" @click="reativarBatidaDuplicada(row)">Reativar</button></td>
                    </tr>
                    <tr v-if="!batidasInativas.length"><td colspan="5" class="empty-cell">Nenhuma batida marcada como duplicidade.</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
            </div>
          </div>
        </template>
      </div>
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

  </div>
</template>
