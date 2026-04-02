<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { apurarPeriodo, listEmployees, type ApuracaoDia, type ApuracaoResumo, type ComboOption } from "../services/crud";
import { formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

type ModoColaborador = "todos" | "ativos" | "inativos" | "selecionados";
type ModoPeriodo = "competencia" | "intervalo";
type ModoVisualizacao = "sintetico" | "analitico";

const session = useSessionStore();
const loading = ref(false);
const error = ref("");
const message = ref("");
const result = ref<ApuracaoResumo | null>(null);
const employeeOptions = ref<ComboOption[]>([]);

const hoje = new Date();
const filters = reactive({
  modoColaborador: "ativos" as ModoColaborador,
  selectedIds: [] as number[],
  modoPeriodo: "competencia" as ModoPeriodo,
  competenciaAno: hoje.getFullYear(),
  competenciaMes: hoje.getMonth() + 1,
  dataInicial: new Date(hoje.getFullYear(), hoje.getMonth(), 1).toISOString().slice(0, 10),
  dataFinal: new Date(hoje.getFullYear(), hoje.getMonth() + 1, 0).toISOString().slice(0, 10),
  visualizacao: "sintetico" as ModoVisualizacao,
});

const resumoSintetico = computed(() => {
  const rows = result.value?.rows ?? [];
  const grouped = new Map<number, {
    funcionarioNome: string;
    previsto: number;
    trabalhado: number;
    saldo: number;
    extras: number;
    faltantes: number;
    bancoCredor: number;
    bancoDevedor: number;
    atrasos: number;
    saidasAntecipadas: number;
    faltas: number;
    folgasDescansos: number;
    abonos: number;
    ajustesManuais: number;
  }>();

  for (const row of rows) {
    if (!grouped.has(row.funcionario_id)) {
      grouped.set(row.funcionario_id, {
        funcionarioNome: row.funcionario_nome,
        previsto: 0,
        trabalhado: 0,
        saldo: 0,
        extras: 0,
        faltantes: 0,
        bancoCredor: 0,
        bancoDevedor: 0,
        atrasos: 0,
        saidasAntecipadas: 0,
        faltas: 0,
        folgasDescansos: 0,
        abonos: 0,
        ajustesManuais: 0,
      });
    }
    const current = grouped.get(row.funcionario_id)!;
    current.previsto += row.horario_esperado_minutos;
    current.trabalhado += row.trabalhado_minutos;
    current.saldo += row.saldo_minutos;
    current.extras += row.extra_minutos;
    current.atrasos += row.atraso_minutos;
    current.saidasAntecipadas += row.saida_antecipada_minutos;
    current.abonos += row.minutos_abonados;
    if (row.saldo_minutos < 0) {
      current.faltantes += Math.abs(row.saldo_minutos);
      current.bancoDevedor += Math.abs(row.saldo_minutos);
    } else {
      current.bancoCredor += row.saldo_minutos;
    }
    if (row.horario_esperado_minutos > 0 && row.trabalhado_minutos === 0 && !row.abonado) {
      current.faltas += 1;
    }
    if (row.tipo_jornada.startsWith("folga") || row.ocorrencias.some((item) => item.toLowerCase().includes("feriado"))) {
      current.folgasDescansos += 1;
    }
    if (row.mensagens.some((item) => item.toLowerCase().includes("manual"))) {
      current.ajustesManuais += 1;
    }
  }

  return Array.from(grouped.values()).sort((a, b) => a.funcionarioNome.localeCompare(b.funcionarioNome));
});

const analiticoPorFuncionario = computed(() => {
  const grouped = new Map<string, ApuracaoDia[]>();
  for (const row of result.value?.rows ?? []) {
    const key = row.funcionario_nome;
    if (!grouped.has(key)) grouped.set(key, []);
    grouped.get(key)!.push(row);
  }
  return Array.from(grouped.entries()).sort((a, b) => a[0].localeCompare(b[0]));
});

async function loadEmployees() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: false });
  employeeOptions.value = rows.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
}

function parseMultiSelect(event: Event) {
  const select = event.target as HTMLSelectElement;
  filters.selectedIds = Array.from(select.selectedOptions).map((opt) => Number(opt.value)).filter((id) => id > 0);
}

async function gerarRelatorio() {
  error.value = "";
  message.value = "";
  if (filters.modoColaborador === "selecionados" && filters.selectedIds.length === 0) {
    error.value = "Selecione pelo menos um colaborador para o filtro manual.";
    return;
  }

  loading.value = true;
  try {
    result.value = await apurarPeriodo({
      empresaId: session.activeCompanyId ?? null,
      employeeStatus: filters.modoColaborador,
      funcionarioIds: filters.modoColaborador === "selecionados" ? filters.selectedIds : null,
      competenciaAno: filters.modoPeriodo === "competencia" ? filters.competenciaAno : null,
      competenciaMes: filters.modoPeriodo === "competencia" ? filters.competenciaMes : null,
      dataInicial: filters.modoPeriodo === "intervalo" ? filters.dataInicial : null,
      dataFinal: filters.modoPeriodo === "intervalo" ? filters.dataFinal : null,
    });
    message.value = "Relatório consolidado gerado com sucesso.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao gerar relatório de horas.";
  } finally {
    loading.value = false;
  }
}

function exportarCsvSintetico() {
  if (!resumoSintetico.value.length) return;
  const header = ["colaborador", "previsto", "trabalhado", "saldo", "extras", "faltantes", "banco_credor", "banco_devedor", "atrasos", "saidas_antecipadas", "faltas", "folgas_descansos", "abonos", "ajustes_manuais"];
  const body = resumoSintetico.value.map((row) => [
    row.funcionarioNome,
    row.previsto,
    row.trabalhado,
    row.saldo,
    row.extras,
    row.faltantes,
    row.bancoCredor,
    row.bancoDevedor,
    row.atrasos,
    row.saidasAntecipadas,
    row.faltas,
    row.folgasDescansos,
    row.abonos,
    row.ajustesManuais,
  ]);
  const csv = [header, ...body].map((line) => line.map((value) => `"${String(value).replace(/"/g, '""')}"`).join(",")).join("\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `relatorio_horas_sintetico_${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

function imprimir() {
  window.print();
}

watch(() => session.activeCompanyId, loadEmployees);
onMounted(loadEmployees);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Relatório consolidado de horas</h2>
        <div class="muted">Base para fechamento de folha e envio à contabilidade (modo sintético e analítico).</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="exportarCsvSintetico" :disabled="!resumoSintetico.length">Exportar CSV</button>
        <button class="primary" @click="imprimir" :disabled="!result">Imprimir</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-if="message" class="alert success">{{ message }}</div>

    <div class="card grid page-gap">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Colaboradores</label>
          <select v-model="filters.modoColaborador">
            <option value="todos">Todos</option>
            <option value="ativos">Apenas ativos</option>
            <option value="inativos">Apenas inativos</option>
            <option value="selecionados">Selecionar manualmente</option>
          </select>
        </div>
        <div class="field">
          <label>Período</label>
          <select v-model="filters.modoPeriodo">
            <option value="competencia">Competência (mês/ano)</option>
            <option value="intervalo">Intervalo de datas</option>
          </select>
        </div>
        <div v-if="filters.modoPeriodo === 'competencia'" class="field">
          <label>Competência</label>
          <div class="grid columns-2">
            <input v-model.number="filters.competenciaMes" type="number" min="1" max="12" />
            <input v-model.number="filters.competenciaAno" type="number" min="2020" max="2100" />
          </div>
        </div>
        <template v-else>
          <div class="field"><label>Data inicial</label><input v-model="filters.dataInicial" type="date" /></div>
          <div class="field"><label>Data final</label><input v-model="filters.dataFinal" type="date" /></div>
        </template>
      </div>

      <div v-if="filters.modoColaborador === 'selecionados'" class="field">
        <label>Seleção manual de colaboradores</label>
        <select multiple size="8" @change="parseMultiSelect">
          <option v-for="item in employeeOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
        </select>
        <small class="muted">Use Ctrl/Cmd + clique para selecionar múltiplos colaboradores.</small>
      </div>

      <div class="grid columns-3 mobile-columns-1">
        <div class="field">
          <label>Visualização</label>
          <select v-model="filters.visualizacao">
            <option value="sintetico">Sintético</option>
            <option value="analitico">Analítico</option>
          </select>
        </div>
        <div class="actions align-end">
          <button class="primary" @click="gerarRelatorio" :disabled="loading">{{ loading ? 'Apurando...' : 'Gerar relatório' }}</button>
        </div>
      </div>
    </div>

    <div v-if="result" class="grid page-gap">
      <div class="kpis">
        <div class="kpi"><strong>Colaboradores</strong><span>{{ result.total_funcionarios }}</span></div>
        <div class="kpi"><strong>Dias apurados</strong><span>{{ result.total_dias }}</span></div>
        <div class="kpi"><strong>Previsto</strong><span>{{ formatMinutes(result.total_esperado_minutos) }}</span></div>
        <div class="kpi"><strong>Trabalhado</strong><span>{{ formatMinutes(result.total_trabalhado_minutos) }}</span></div>
        <div class="kpi"><strong>Saldo</strong><span>{{ formatMinutes(result.total_saldo_minutos) }}</span></div>
        <div class="kpi"><strong>Horas extras</strong><span>{{ formatMinutes(result.total_extra_minutos) }}</span></div>
      </div>

      <div v-if="filters.visualizacao === 'sintetico'" class="card">
        <h3 style="margin-top: 0;">Resumo sintético por colaborador</h3>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Colaborador</th>
                <th>Previsto</th>
                <th>Trabalhado</th>
                <th>Saldo</th>
                <th>Extras</th>
                <th>Faltantes</th>
                <th>Banco credor</th>
                <th>Banco devedor</th>
                <th>Atrasos</th>
                <th>Saídas antecipadas</th>
                <th>Faltas</th>
                <th>Folgas/descansos</th>
                <th>Abonos</th>
                <th>Ajustes manuais</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in resumoSintetico" :key="row.funcionarioNome">
                <td>{{ row.funcionarioNome }}</td>
                <td>{{ formatMinutes(row.previsto) }}</td>
                <td>{{ formatMinutes(row.trabalhado) }}</td>
                <td>{{ formatMinutes(row.saldo) }}</td>
                <td>{{ formatMinutes(row.extras) }}</td>
                <td>{{ formatMinutes(row.faltantes) }}</td>
                <td>{{ formatMinutes(row.bancoCredor) }}</td>
                <td>{{ formatMinutes(row.bancoDevedor) }}</td>
                <td>{{ formatMinutes(row.atrasos) }}</td>
                <td>{{ formatMinutes(row.saidasAntecipadas) }}</td>
                <td>{{ row.faltas }}</td>
                <td>{{ row.folgasDescansos }}</td>
                <td>{{ formatMinutes(row.abonos) }}</td>
                <td>{{ row.ajustesManuais }}</td>
              </tr>
              <tr v-if="!resumoSintetico.length"><td colspan="14" class="muted">Nenhum dado para o período selecionado.</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div v-else class="card grid page-gap">
        <h3 style="margin-top: 0;">Detalhamento analítico</h3>
        <div v-for="[funcionario, rows] in analiticoPorFuncionario" :key="funcionario" class="table-wrap">
          <h4>{{ funcionario }}</h4>
          <table>
            <thead>
              <tr>
                <th>Data</th>
                <th>Jornada</th>
                <th>Batidas</th>
                <th>Ocorrências</th>
                <th>Previsto</th>
                <th>Trabalhado</th>
                <th>Saldo</th>
                <th>Atraso</th>
                <th>Saída antecipada</th>
                <th>Extra</th>
                <th>Mensagens</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="`${row.funcionario_id}-${row.data}`">
                <td>{{ row.data }}</td>
                <td>{{ row.jornada_nome }}</td>
                <td>{{ row.batidas.join(' | ') || '-' }}</td>
                <td>{{ row.ocorrencias.join(' | ') || '-' }}</td>
                <td>{{ formatMinutes(row.horario_esperado_minutos) }}</td>
                <td>{{ formatMinutes(row.trabalhado_minutos) }}</td>
                <td>{{ formatMinutes(row.saldo_minutos) }}</td>
                <td>{{ formatMinutes(row.atraso_minutos) }}</td>
                <td>{{ formatMinutes(row.saida_antecipada_minutos) }}</td>
                <td>{{ formatMinutes(row.extra_minutos) }}</td>
                <td>{{ row.mensagens.join(' | ') || '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
