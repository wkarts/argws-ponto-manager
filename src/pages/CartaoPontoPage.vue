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
  ids: number[];
  repeticoes: number;
  diferencaSegundos: number;
}

const smartSuggestions = ref<SmartSuggestionItem[]>([]);
const duplicateCandidates = ref<DuplicatePunchCandidate[]>([]);
const gridEditor = reactive<Record<string, string>>({});
const gridSaving = reactive<Record<string, boolean>>({});
const gridCellRefs = ref<Record<string, HTMLInputElement | null>>({});
const gridStatus = ref('Pronto para edição inline. Use Enter, setas e Del para operar a grade.');

type SidebarTab = 'marcacoes' | 'ocorrencias' | 'smart' | 'duplicadas';
const sidebarCollapsed = ref(false);
const sidebarTab = ref<SidebarTab>('marcacoes');

function toggleSidebarCollapse() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

function setSidebarTab(tab: SidebarTab) {
  sidebarTab.value = tab;
  sidebarCollapsed.value = false;
}

const hoje = new Date();
const filtros = reactive({
  funcionarioId: "",
  modoPeriodo: "intervalo" as "intervalo" | "competencia",
  competenciaMes: hoje.getMonth() + 1,
  competenciaAno: hoje.getFullYear(),
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
const inconsistenciasNoPeriodo = computed(() => (apuracaoResumo.value?.rows || []).filter((row) => row.inconsistente).length);
const diasComOcorrenciaNoPeriodo = computed(() => (apuracaoResumo.value?.rows || []).filter((row) => (row.ocorrencias || []).length > 0).length);
const periodoLabel = computed(() => {
  if (filtros.modoPeriodo === "competencia") {
    return `${String(filtros.competenciaMes).padStart(2, "0")}/${filtros.competenciaAno}`;
  }
  return `${filtros.dataInicial}..${filtros.dataFinal}`;
});

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
      if (gridEditor[key] == null) gridEditor[key] = defaultValue;
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
    }))
    .filter((item) => item.id > 0 && item.date && item.hora)
    .sort((a, b) => `${a.date} ${a.hora}`.localeCompare(`${b.date} ${b.hora}`));

  const byDay: Record<string, typeof rows> = {};
  for (const row of rows) {
    const key = `${row.funcionarioNome}::${row.date}`;
    (byDay[key] ||= []).push(row);
  }

  for (const [groupKey, items] of Object.entries(byDay)) {
    let current: typeof items = [];
    for (const item of items) {
      const currMinutes = parseTimeToMinutes(item.hora);
      const lastMinutes = current.length ? parseTimeToMinutes(current[current.length - 1].hora) : null;
      if (current.length === 0 || currMinutes == null || lastMinutes == null || (currMinutes - lastMinutes) > 1) {
        if (current.length > 1) {
          const [funcionarioNome, date] = groupKey.split('::');
          grouped.push({
            key: `${groupKey}:${current[0].hora}`,
            date,
            funcionarioNome,
            horarioBase: current[0].hora,
            ids: current.map((entry) => entry.id),
            repeticoes: current.length,
            diferencaSegundos: Math.max(0, (parseTimeToMinutes(current[current.length - 1].hora) || 0) - (parseTimeToMinutes(current[0].hora) || 0)) * 60,
          });
        }
        current = [item];
      } else {
        current.push(item);
      }
    }
    if (current.length > 1) {
      const [funcionarioNome, date] = groupKey.split('::');
      grouped.push({
        key: `${groupKey}:${current[0].hora}`,
        date,
        funcionarioNome,
        horarioBase: current[0].hora,
        ids: current.map((entry) => entry.id),
        repeticoes: current.length,
        diferencaSegundos: Math.max(0, (parseTimeToMinutes(current[current.length - 1].hora) || 0) - (parseTimeToMinutes(current[0].hora) || 0)) * 60,
      });
    }
  }

  duplicateCandidates.value = grouped;
  resetSelectionMap(duplicateSelection);
  for (const item of grouped) {
    duplicateSelection[item.key] = true;
  }
  message.value = grouped.length ? `${grouped.length} agrupamento(s) de batidas muito próximas localizado(s).` : 'Nenhuma batida duplicada ou muito próxima foi localizada no filtro atual.';
}

async function excluirDuplicidadesSelecionadas() {
  const ids = duplicateCandidates.value
    .filter((item) => duplicateSelection[item.key])
    .flatMap((item) => item.ids.slice(1));
  if (!ids.length) {
    message.value = 'Nenhuma duplicidade selecionada para exclusão.';
    showSplashInfo(message.value);
    return;
  }
  if (!confirm(`Excluir ${ids.length} batida(s) duplicada(s)/muito próxima(s)?`)) return;

  duplicateBusy.value = true;
  error.value = '';
  message.value = '';
  try {
    for (const id of ids) {
      await deleteBatida(id);
    }
    message.value = `${ids.length} batida(s) removida(s) com sucesso.`;
    showSplashSuccess(message.value);
    await carregarCartao();
    localizarDuplicidades();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Falha ao excluir batidas duplicadas.';
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
        competenciaAno: filtros.modoPeriodo === "competencia" ? Number(filtros.competenciaAno) : null,
        competenciaMes: filtros.modoPeriodo === "competencia" ? Number(filtros.competenciaMes) : null,
        dataInicial: filtros.modoPeriodo === "competencia" ? null : (filtros.dataInicial || null),
        dataFinal: filtros.modoPeriodo === "competencia" ? null : (filtros.dataFinal || null),
      }),
    ]);
    batidas.value = rowsBatida;
    ocorrencias.value = rowsOcorrencia;
    apuracaoResumo.value = apuracao;
    reportHtml.value = buildCartaoHtml();
    const availableDates = apuracao.rows.map((item) => item.data);
    if (!selectedDate.value || !availableDates.includes(selectedDate.value)) {
      selectedDate.value = availableDates[0] || filtros.dataInicial;
    }
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

function buildCartaoHtmlFromSummary(summary: ApuracaoResumo | null, employeeName: string, dataInicial: string, dataFinal: string): string {
  if (!dataInicial || !dataFinal) return "";
  const initial = new Date(`${dataInicial}T00:00:00`);
  const final = new Date(`${dataFinal}T00:00:00`);
  if (Number.isNaN(initial.getTime()) || Number.isNaN(final.getTime()) || initial > final) return "";

  const { rows: dailyRows, totals } = buildDailyRows(summary, initial, final);

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
          <div class="meta">Período: ${dataInicial.split("-").reverse().join("/")} até ${dataFinal.split("-").reverse().join("/")}</div>
          <div class="meta">Competência/visão: ${periodoLabel.value}</div>
          <div class="meta">Empresa: ${session.activeCompanyName || "-"}</div>
          <div class="meta">Colaborador: ${employeeName}</div>
        </div>
        <div class="meta">Emitido em ${new Date().toLocaleDateString("pt-BR")}</div>
      </div>
      <table>
        ${tableByModel[filtros.modeloRelatorio] || tableByModel.folha_resumida}
      </table>
      ${summaryByModel}
      <div class="sign"><div class="line">${employeeName}</div><div class="line">${empresaResponsavel.value}</div></div>
    </body></html>`;
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

function buildAllCardsHtml(cards: { employeeName: string; html: string }[]) {
  const content = cards
    .map((card, index) => {
      const pageBreak = index < cards.length - 1 ? '<div style="page-break-after: always;"></div>' : "";
      return `<section data-employee="${card.employeeName}">${card.html}</section>${pageBreak}`;
    })
    .join("");

  return `<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"><title>Cartões da competência</title>
    <style>
      body{margin:0;background:#fff}
      section{padding:0}
      @page{size:A4 portrait;margin:10mm}
    </style></head><body>${content}</body></html>`;
}

async function generateCompetenciaCardsHtml() {
  syncPeriodFilters();
  const periodo = periodoAtual();
  const targetEmployees = employeeOptions.value.filter((item) => item.id > 0);
  const cards: { employeeName: string; html: string }[] = [];

  for (const employee of targetEmployees) {
    const summary = await apurarPeriodo({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: employee.id,
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
    throw new Error("Não foi possível inicializar o modo de impressão.");
  }

  doc.open();
  doc.write(html);
  doc.close();
  frame.contentWindow.focus();
  await new Promise<void>((resolve) => {
    setTimeout(() => {
      try {
        frame.contentWindow?.print();
      } finally {
        setTimeout(() => frame.remove(), 1000);
        resolve();
      }
    }, 250);
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
    showSplashSuccess(message.value);
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover batida.";
    showSplashError(error.value);
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

watch(() => [filtros.modoPeriodo, filtros.competenciaMes, filtros.competenciaAno], () => {
  if (filtros.modoPeriodo === "competencia") {
    syncPeriodFilters();
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
  await carregarBase();
  await carregarCartao();
});
</script>

<template>
  <div class="grid page-gap cartao-vb6-page">
    <div class="toolbar cartao-vb6-toolbar">
      <div>
        <h2>Cartão de ponto</h2>
        <div class="muted-text">Visão centralizada no estilo operacional: dias do período expostos em grade, ações rápidas e tratamento na própria tela.</div>
      </div>
      <div class="actions compact-toolbar">
        <button class="secondary" :disabled="loading" @click="carregarCartao">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
        <button class="secondary" @click="exportarHtml">Exportar HTML</button>
        <button class="secondary" @click="exportarExcel">Exportar Excel</button>
        <button class="secondary" :disabled="printingAllCompetencia" @click="imprimirTodosCompetencia">{{ printingAllCompetencia ? 'Preparando lote...' : 'Imprimir todos da competência' }}</button>
        <button class="primary" @click="imprimirOuSalvarPdf">Imprimir / Salvar PDF</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-if="message" class="alert success">{{ message }}</div>

    <div class="card card-tight cartao-filter-card">
      <div class="filter-grid compact cartao-filter-grid">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="filtros.funcionarioId">
            <option value="">Todos</option>
            <option v-for="item in employeeOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Período</label>
          <select v-model="filtros.modoPeriodo">
            <option value="intervalo">Intervalo</option>
            <option value="competencia">Competência</option>
          </select>
        </div>
        <div v-if="filtros.modoPeriodo === 'competencia'" class="field">
          <label>Competência</label>
          <div class="inline-grid compact-inline-grid">
            <input v-model.number="filtros.competenciaMes" type="number" min="1" max="12" />
            <input v-model.number="filtros.competenciaAno" type="number" min="2020" max="2100" />
          </div>
        </div>
        <template v-else>
          <div class="field">
            <label>Data inicial</label>
            <input v-model="filtros.dataInicial" type="date" />
          </div>
          <div class="field">
            <label>Data final</label>
            <input v-model="filtros.dataFinal" type="date" />
          </div>
        </template>
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
      <div class="inline-info-strip">
        <span><strong>Visão:</strong> {{ periodoLabel }}</span>
        <span><strong>Colaborador:</strong> {{ funcionarioNomeSelecionado }}</span>
        <span><strong>Dias inconsistentes:</strong> {{ inconsistenciasNoPeriodo }}</span>
        <span><strong>Dias com ocorrência:</strong> {{ diasComOcorrenciaNoPeriodo }}</span>
        <span><strong>Dia selecionado:</strong> {{ selectedDayLabel }}</span>
      </div>
      <div class="inline-info-strip subtle">
        <span><strong>Operação inline:</strong> Enter salva e avança, Del remove, setas navegam entre células.</span>
        <span>{{ gridStatus }}</span>
      </div>
    </div>

    <div class="cartao-vb6-shell cartao-shell-split">
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
                  :disabled="gridSaving[slot.key] || !funcionarioIdNumero"
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

      <div class="cartao-right-workspace" :class="{ collapsed: sidebarCollapsed }">
        <div class="card card-tight lateral-tabs-card">
          <div class="lateral-tabs-shell">
            <div class="vertical-tabs-rail">
              <button type="button" class="rail-toggle" @click="toggleSidebarCollapse">{{ sidebarCollapsed ? '»' : '«' }}</button>
              <button type="button" class="vertical-tab" :class="{ active: sidebarTab === 'marcacoes' }" @click="setSidebarTab('marcacoes')">Marcações</button>
              <button type="button" class="vertical-tab" :class="{ active: sidebarTab === 'ocorrencias' }" @click="setSidebarTab('ocorrencias')">Ocorrências</button>
              <button type="button" class="vertical-tab" :class="{ active: sidebarTab === 'smart' }" @click="setSidebarTab('smart')">Smart</button>
              <button type="button" class="vertical-tab" :class="{ active: sidebarTab === 'duplicadas' }" @click="setSidebarTab('duplicadas')">Exclusão</button>
            </div>
            <div v-if="!sidebarCollapsed" class="lateral-tab-content">
              <template v-if="sidebarTab === 'marcacoes'">
                <div class="vb6-group-header">
                  <h3>Marcações do dia selecionado</h3>
                  <div class="actions compact-actions">
                    <button class="secondary" @click="openNovaBatida(selectedDate)">Nova marcação</button>
                  </div>
                </div>
                <div class="compact-table-wrap">
                  <table class="quick-table table-compact">
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
                      <tr v-for="row in batidasSelecionadas" :key="String(row.id)" :class="rowBadgeClass(row)">
                        <td>{{ row.data_referencia }}</td>
                        <td>{{ row.hora }}</td>
                        <td>{{ row.tipo }}</td>
                        <td>{{ row.origem || '-' }}</td>
                        <td>{{ row.justificativa_nome || '-' }}</td>
                        <td>
                          <div class="actions compact-actions">
                            <button class="secondary" @click="editarBatida(row)">Editar</button>
                            <button class="secondary" @click="moverBatida(row, -1)">-1m</button>
                            <button class="secondary" @click="moverBatida(row, 1)">+1m</button>
                            <button class="danger" @click="removerBatida(row)">Remover</button>
                          </div>
                        </td>
                      </tr>
                      <tr v-if="!batidasSelecionadas.length">
                        <td colspan="6" class="empty-cell">Nenhuma marcação encontrada para o dia selecionado.</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </template>

              <template v-else-if="sidebarTab === 'ocorrencias'">
                <div class="vb6-group-header">
                  <h3>Ocorrências do dia selecionado</h3>
                  <div class="actions compact-actions">
                    <button class="secondary" @click="openNovaOcorrencia(selectedDate)">Nova ocorrência</button>
                  </div>
                </div>
                <div class="compact-table-wrap">
                  <table class="quick-table table-compact">
                    <thead><tr><th>Data</th><th>Tipo</th><th>Justificativa</th><th>Abono</th><th>Observação</th><th>Ação</th></tr></thead>
                    <tbody>
                      <tr v-for="row in ocorrenciasSelecionadas" :key="String(row.id)" :class="rowBadgeClass(row)">
                        <td>{{ row.data_referencia }}</td><td>{{ row.tipo }}</td><td>{{ row.justificativa_nome || '-' }}</td><td>{{ Number(row.minutos_abonados || 0) > 0 ? row.minutos_abonados : (Number(row.abonar_dia) === 1 ? 'Dia abonado' : '-') }}</td><td>{{ row.observacao || '-' }}</td>
                        <td><div class="actions compact-actions"><button class="secondary" @click="editarOcorrencia(row)">Editar</button><button class="danger" @click="removerOcorrencia(row)">Remover</button></div></td>
                      </tr>
                      <tr v-if="!ocorrenciasSelecionadas.length"><td colspan="6" class="empty-cell">Nenhuma ocorrência encontrada para o dia selecionado.</td></tr>
                    </tbody>
                  </table>
                </div>
              </template>

              <template v-else-if="sidebarTab === 'smart'">
                <div class="vb6-group-header">
                  <h3>Motor smart</h3>
                  <div class="actions compact-actions">
                    <button class="secondary" :disabled="smartBusy" @click="analisarSugestoes">Analisar</button>
                    <button class="primary" :disabled="smartBusy" @click="tratarTodosAutomaticos">Tratar todos automáticos</button>
                  </div>
                </div>
                <div class="smart-summary-grid">
                  <div><strong>Esquecimentos</strong><span>{{ smartResumo.esquecimentos }}</span></div>
                  <div><strong>Faltas</strong><span>{{ smartResumo.faltas }}</span></div>
                  <div><strong>Folgas móveis</strong><span>{{ smartResumo.trocasFolga }}</span></div>
                  <div><strong>Meia folga</strong><span>{{ smartResumo.meiasFolga }}</span></div>
                </div>
                <div class="filter-grid compact">
                  <div class="field">
                    <label>Tipo padrão para faltas</label>
                    <select v-model="smartFaltaTipo">
                      <option value="falta">Falta</option>
                      <option value="falta_justificada">Falta justificada</option>
                      <option value="atestado">Atestado</option>
                      <option value="abono">Abono</option>
                    </select>
                  </div>
                  <div class="field">
                    <label>Justificativa padrão</label>
                    <select v-model="smartJustificativaId">
                      <option value="">Sem justificativa</option>
                      <option v-for="item in justificativaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
                    </select>
                  </div>
                </div>
                <div class="actions compact-actions">
                  <button class="secondary" :disabled="smartBusy" @click="aplicarSugestoesSelecionadas(false)">Aplicar selecionadas</button>
                  <button class="secondary" :disabled="smartBusy" @click="aplicarSugestoesSelecionadas(true)">Aplicar seguras</button>
                </div>
                <div class="compact-table-wrap">
                  <table class="quick-table table-compact">
                    <thead><tr><th></th><th>Data</th><th>Tipo</th><th>Batidas</th><th>Observação</th></tr></thead>
                    <tbody>
                      <tr v-for="item in smartSuggestions" :key="item.key" @click="selectDay(item.date)">
                        <td><input v-model="smartSuggestionSelection[item.key]" type="checkbox" /></td>
                        <td>{{ item.date }}</td>
                        <td><span class="badge" :class="suggestionBadgeClass(item.tipo)">{{ item.titulo }}</span></td>
                        <td>{{ item.batidas.join(' | ') || '-' }}</td>
                        <td>{{ item.observacao }}</td>
                      </tr>
                      <tr v-if="!smartSuggestions.length"><td colspan="5" class="empty-cell">Nenhuma sugestão smart gerada para o período atual.</td></tr>
                    </tbody>
                  </table>
                </div>
              </template>

              <template v-else>
                <div class="vb6-group-header">
                  <h3>Exclusão assistida de batidas</h3>
                  <div class="actions compact-actions">
                    <button class="secondary" :disabled="duplicateBusy" @click="localizarDuplicidades">Localizar</button>
                    <button class="danger" :disabled="duplicateBusy" @click="excluirDuplicidadesSelecionadas">Excluir</button>
                  </div>
                </div>
                <div class="compact-table-wrap">
                  <table class="quick-table table-compact">
                    <thead><tr><th></th><th>Data</th><th>Funcionário</th><th>Hora</th><th>Rep.</th><th>IDs</th></tr></thead>
                    <tbody>
                      <tr v-for="item in duplicateCandidates" :key="item.key" @click="selectDay(item.date)">
                        <td><input v-model="duplicateSelection[item.key]" type="checkbox" /></td>
                        <td>{{ item.date }}</td><td>{{ item.funcionarioNome }}</td><td>{{ item.horarioBase }}</td><td>{{ item.repeticoes }}</td><td>{{ item.ids.join(', ') }}</td>
                      </tr>
                      <tr v-if="!duplicateCandidates.length"><td colspan="6" class="empty-cell">Nenhuma duplicidade exata ou muito próxima localizada para o filtro atual.</td></tr>
                    </tbody>
                  </table>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="card table-wrap card-tight">
      <div class="vb6-group-header">
        <h3>Pré-visualização do relatório</h3>
        <div class="muted-text">Mantida a prévia do cartão para conferência e emissão final.</div>
      </div>
      <iframe class="report-frame" :srcdoc="reportHtml"></iframe>
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
