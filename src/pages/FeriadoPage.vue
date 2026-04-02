<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import AppModal from "../components/AppModal.vue";
import {
  comboList,
  deleteFeriado,
  getFeriado,
  importCompanyDefaultHolidays,
  listFeriados,
  loadHolidaySourceSettings,
  saveFeriado,
  saveHolidaySourceSettings,
  type ComboOption,
  type FeriadoRecord,
  type HolidaySourceSettings,
} from "../services/crud";
import { booleanLabel } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const rows = ref<FeriadoRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const importing = ref(false);
const savingSource = ref(false);
const error = ref("");
const info = ref("");
const search = ref("");
const modalOpen = ref(false);
const viewMode = ref(false);

const empresasOptions = ref<ComboOption[]>([]);
const departamentosOptions = ref<ComboOption[]>([]);
const contextoOptions = ref<ComboOption[]>([]);
const regrasJornadaOptions = ref<ComboOption[]>([]);
const regrasCompensacaoOptions = ref<ComboOption[]>([]);

const sourceSettings = reactive<HolidaySourceSettings>({
  mode: "embedded",
  year: 2026,
  remote_json_url: "",
  api_url: "",
});

const form = reactive<FeriadoRecord>({
  id: undefined,
  data: "",
  descricao: "",
  contexto_tipo: "global",
  empresa_id: null,
  departamento_id: null,
  regra_jornada: null,
  regra_compensacao: null,
  observacoes: "",
  ativo: true,
  empresa_ids: [],
  departamento_ids: [],
});

const subtitle = computed(() => {
  if (viewMode.value) return "Detalhes do feriado e da abrangência aplicada.";
  return "Cadastro de feriados com abrangência por empresa e departamento.";
});

const activeCompanyLabel = computed(() => session.activeCompanyName || "Empresa ativa");

function toNullableNumber(value: unknown): number | null {
  if (value == null) return null;
  if (typeof value === "number") return Number.isFinite(value) && value > 0 ? value : null;
  const text = String(value).trim();
  if (!text) return null;
  const numeric = Number(text);
  return Number.isFinite(numeric) && numeric > 0 ? numeric : null;
}

function resetForm() {
  form.id = undefined;
  form.data = "";
  form.descricao = "";
  form.contexto_tipo = "global";
  form.empresa_id = null;
  form.departamento_id = null;
  form.regra_jornada = null;
  form.regra_compensacao = null;
  form.observacoes = "";
  form.ativo = true;
  form.empresa_ids = [];
  form.departamento_ids = [];
}

function closeModal() {
  modalOpen.value = false;
  viewMode.value = false;
}

function optionLabel(options: ComboOption[], value: unknown): string {
  const found = options.find((item) => String(item.id) === String(value));
  return found?.label ?? String(value ?? "");
}

function labelsFromIds(options: ComboOption[], ids?: number[]): string {
  if (!ids?.length) return "";
  return ids
    .map((id) => optionLabel(options, id))
    .filter(Boolean)
    .join(", ");
}

function normalizeIds(value: unknown): number[] {
  if (!Array.isArray(value)) return [];
  return value
    .map((item) => Number(item))
    .filter((item) => Number.isFinite(item) && item > 0);
}

async function loadOptions() {
  const [empresas, departamentos, contextos, regrasJornada, regrasCompensacao] = await Promise.all([
    comboList("empresas"),
    comboList("departamentos"),
    comboList("contextos_feriado"),
    comboList("regras_jornada"),
    comboList("regras_compensacao"),
  ]);
  empresasOptions.value = empresas;
  departamentosOptions.value = departamentos;
  contextoOptions.value = contextos;
  regrasJornadaOptions.value = regrasJornada;
  regrasCompensacaoOptions.value = regrasCompensacao;
}

async function loadSourceSettings() {
  const payload = await loadHolidaySourceSettings();
  sourceSettings.mode = String(payload.mode || "embedded");
  sourceSettings.year = Number(payload.year || 2026);
  sourceSettings.remote_json_url = String(payload.remote_json_url || "");
  sourceSettings.api_url = String(payload.api_url || "");
}

async function persistSourceSettings() {
  savingSource.value = true;
  error.value = "";
  info.value = "";
  try {
    const saved = await saveHolidaySourceSettings({ ...sourceSettings });
    sourceSettings.mode = String(saved.mode || sourceSettings.mode || "embedded");
    sourceSettings.year = Number(saved.year || sourceSettings.year || 2026);
    sourceSettings.remote_json_url = String(saved.remote_json_url || "");
    sourceSettings.api_url = String(saved.api_url || "");
    info.value = "Configuração da fonte de feriados salva.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar configuração da fonte de feriados.";
  } finally {
    savingSource.value = false;
  }
}

async function importDefaultHolidays() {
  if (!session.activeCompanyId) {
    error.value = "Selecione uma empresa ativa para importar os feriados padrão de 2026.";
    return;
  }
  importing.value = true;
  error.value = "";
  info.value = "";
  try {
    const result = await importCompanyDefaultHolidays(session.activeCompanyId, Number(sourceSettings.year || 2026));
    info.value = `Importação concluída para ${activeCompanyLabel.value}: ${Number(result.total_importado || 0)} feriado(s) novo(s).`;
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao importar feriados padrão.";
  } finally {
    importing.value = false;
  }
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listFeriados(search.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar feriados.";
  } finally {
    loading.value = false;
  }
}

function openNew() {
  resetForm();
  viewMode.value = false;
  modalOpen.value = true;
}

async function openEdit(row: FeriadoRecord, readOnly = false) {
  error.value = "";
  try {
    const payload = await getFeriado(Number(row.id));
    form.id = Number(payload.id);
    form.data = String(payload.data ?? "");
    form.descricao = String(payload.descricao ?? "");
    form.contexto_tipo = String(payload.contexto_tipo ?? "global");
    form.empresa_id = toNullableNumber(payload.empresa_id);
    form.departamento_id = toNullableNumber(payload.departamento_id);
    form.regra_jornada = payload.regra_jornada == null || String(payload.regra_jornada).trim() === "" ? null : String(payload.regra_jornada);
    form.regra_compensacao = payload.regra_compensacao == null || String(payload.regra_compensacao).trim() === "" ? null : String(payload.regra_compensacao);
    form.observacoes = String(payload.observacoes ?? "");
    form.ativo = Boolean(Number(payload.ativo ?? 1));
    form.empresa_ids = normalizeIds(payload.empresa_ids);
    form.departamento_ids = normalizeIds(payload.departamento_ids);
    viewMode.value = readOnly;
    modalOpen.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar feriado.";
  }
}

function toggleMulti(listKey: "empresa_ids" | "departamento_ids", value: number) {
  const current = [...(form[listKey] || [])] as number[];
  const index = current.indexOf(value);
  if (index >= 0) current.splice(index, 1);
  else current.push(value);
  form[listKey] = current;
}

function isChecked(listKey: "empresa_ids" | "departamento_ids", value: number) {
  return (form[listKey] || []).includes(value);
}

async function persist() {
  saving.value = true;
  error.value = "";
  try {
    await saveFeriado({ ...form });
    await load();
    closeModal();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar feriado.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja remover este feriado?")) return;
  try {
    await deleteFeriado(id);
    await load();
    if (Number(form.id) === id) {
      closeModal();
      resetForm();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover feriado.";
  }
}

onMounted(async () => {
  await loadOptions();
  await loadSourceSettings();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Tabela de feriados</h2>
        <div class="muted">Cadastro central de feriados e respectivos vínculos de abrangência.</div>
      </div>
      <div class="actions">
        <input v-model="search" placeholder="Pesquisar feriado..." @keyup.enter="load" />
        <button class="secondary" @click="load">Buscar</button>
        <button class="secondary" @click="openNew">Novo feriado</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-else-if="info" class="alert success">{{ info }}</div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3 style="margin: 0;">Fonte padrão de feriados</h3>
          <div class="muted">A build distribui o JSON 2026 embarcado. Você pode manter o modo local ou apontar uma URL/API própria.</div>
        </div>
        <div class="actions">
          <button class="secondary" :disabled="savingSource" @click="persistSourceSettings">
            {{ savingSource ? "Salvando..." : "Salvar configuração" }}
          </button>
          <button class="secondary" :disabled="importing" @click="importDefaultHolidays">
            {{ importing ? "Importando..." : `Importar 2026 para ${activeCompanyLabel}` }}
          </button>
        </div>
      </div>
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Modo</label>
          <select v-model="sourceSettings.mode">
            <option value="embedded">JSON embarcado</option>
            <option value="remote_json">JSON remoto</option>
            <option value="api">API própria</option>
          </select>
        </div>
        <div class="field">
          <label>Ano</label>
          <input v-model="sourceSettings.year" type="number" min="2026" max="2100" />
        </div>
        <div class="field span-2">
          <label>URL JSON remota</label>
          <input v-model="sourceSettings.remote_json_url" type="text" placeholder="https://seu-servidor/feriados/{year}.json" />
        </div>
        <div class="field span-4">
          <label>URL da API</label>
          <input v-model="sourceSettings.api_url" type="text" placeholder="https://api.exemplo/feriados?empresa={empresa_id}&ano={year}&uf={uf}&cidade={cidade}" />
        </div>
      </div>
    </div>

    <div class="card">
      <h3 style="margin-top: 0;">Listagem</h3>
      <div v-if="loading" class="muted">Carregando feriados...</div>

      <div v-else class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>id</th>
              <th>data</th>
              <th>descrição</th>
              <th>contexto</th>
              <th>vínculos</th>
              <th>empresas</th>
              <th>departamentos</th>
              <th>compensação</th>
              <th>ativo</th>
              <th>ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="String(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.data }}</td>
              <td>{{ row.descricao }}</td>
              <td>{{ row.contexto_tipo }}</td>
              <td>{{ Number(row.empresas_count || 0) }} empresa(s) · {{ Number(row.departamentos_count || 0) }} departamento(s)</td>
              <td>
                {{ Array.isArray(row.empresas_labels) && row.empresas_labels.length
                  ? row.empresas_labels.join(", ")
                  : "-" }}
              </td>
              <td>
                {{ Array.isArray(row.departamentos_labels) && row.departamentos_labels.length
                  ? row.departamentos_labels.join(", ")
                  : "-" }}
              </td>
              <td>{{ row.regra_compensacao || "-" }}</td>
              <td>{{ booleanLabel(row.ativo) }}</td>
              <td>
                <div class="actions compact-actions">
                  <button class="secondary" @click="openEdit(row, true)">Ver</button>
                  <button class="secondary" @click="openEdit(row)">Editar</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="10" class="muted">Nenhum feriado encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <AppModal
      :open="modalOpen"
      :title="form.id ? 'Editar feriado' : 'Novo feriado'"
      :subtitle="subtitle"
      width="xl"
      @close="closeModal"
    >
      <div class="grid page-gap">
        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Data</label>
            <input v-model="form.data" type="date" :disabled="viewMode" />
          </div>
          <div class="field">
            <label>Descrição</label>
            <input v-model="form.descricao" type="text" :disabled="viewMode" />
          </div>
          <div class="field">
            <label>Contexto</label>
            <select v-model="form.contexto_tipo" :disabled="viewMode">
              <option v-for="item in contextoOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
          <div class="field checkbox-line compact-checkbox">
            <input v-model="form.ativo" class="checkbox-input" type="checkbox" :disabled="viewMode" />
            <label>Ativo</label>
          </div>
        </div>

        <div class="section-title">Abrangência por empresa</div>
        <div class="grid page-gap">
          <div class="check-grid">
            <label v-for="item in empresasOptions" :key="item.id" class="check-item">
              <input
                :checked="isChecked('empresa_ids', item.id)"
                type="checkbox"
                :disabled="viewMode"
                @change="toggleMulti('empresa_ids', item.id)"
              />
              <span>{{ item.label }}</span>
            </label>
            <div v-if="empresasOptions.length === 0" class="muted">Nenhuma empresa cadastrada.</div>
          </div>
          <div class="muted small-text">Abrangência selecionada: {{ labelsFromIds(empresasOptions, form.empresa_ids) || 'nenhuma' }}</div>
        </div>

        <div class="section-title">Abrangência por departamento</div>
        <div class="grid page-gap">
          <div class="check-grid">
            <label v-for="item in departamentosOptions" :key="item.id" class="check-item">
              <input
                :checked="isChecked('departamento_ids', item.id)"
                type="checkbox"
                :disabled="viewMode"
                @change="toggleMulti('departamento_ids', item.id)"
              />
              <span>{{ item.label }}</span>
            </label>
            <div v-if="departamentosOptions.length === 0" class="muted">Nenhum departamento cadastrado.</div>
          </div>
          <div class="muted small-text">Abrangência selecionada: {{ labelsFromIds(departamentosOptions, form.departamento_ids) || 'nenhuma' }}</div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Regra de jornada</label>
            <select v-model="form.regra_jornada" :disabled="viewMode">
              <option :value="null">Selecione</option>
              <option v-for="item in regrasJornadaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Regra de compensação</label>
            <select v-model="form.regra_compensacao" :disabled="viewMode">
              <option :value="null">Selecione</option>
              <option v-for="item in regrasCompensacaoOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
        </div>

        <div class="field">
          <label>Observações</label>
          <textarea v-model="form.observacoes" rows="4" :disabled="viewMode"></textarea>
        </div>

        <div v-if="!viewMode" class="actions">
          <button class="primary" :disabled="saving" @click="persist">
            {{ saving ? 'Salvando...' : form.id ? 'Atualizar feriado' : 'Salvar feriado' }}
          </button>
          <button class="secondary" @click="resetForm">Limpar</button>
        </div>
      </div>
    </AppModal>
  </div>
</template>
