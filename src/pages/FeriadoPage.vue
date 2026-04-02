<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import AppModal from "../components/AppModal.vue";
import {
  comboList,
  deleteFeriado,
  getFeriado,
  listFeriados,
  saveFeriado,
  type ComboOption,
  type FeriadoRecord,
} from "../services/crud";
import { booleanLabel } from "../services/format";

const rows = ref<FeriadoRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const search = ref("");
const modalOpen = ref(false);
const viewMode = ref(false);

const empresasOptions = ref<ComboOption[]>([]);
const departamentosOptions = ref<ComboOption[]>([]);
const contextoOptions = ref<ComboOption[]>([]);
const regrasJornadaOptions = ref<ComboOption[]>([]);
const regrasCompensacaoOptions = ref<ComboOption[]>([]);

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
    form.empresa_id = payload.empresa_id == null || payload.empresa_id === "" ? null : Number(payload.empresa_id);
    form.departamento_id = payload.departamento_id == null || payload.departamento_id === "" ? null : Number(payload.departamento_id);
    form.regra_jornada = payload.regra_jornada == null || payload.regra_jornada === "" ? null : String(payload.regra_jornada);
    form.regra_compensacao = payload.regra_compensacao == null || payload.regra_compensacao === "" ? null : String(payload.regra_compensacao);
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
      :title="viewMode ? `Detalhes do feriado` : form.id ? `Editar feriado` : `Novo feriado`"
      :subtitle="subtitle"
      width="xl"
      @close="closeModal"
    >
      <form class="grid feriado-form" @submit.prevent="persist">
        <div class="field">
          <label for="feriado_data">Data <span class="required">*</span></label>
          <input id="feriado_data" v-model="form.data" type="date" :disabled="viewMode" />
        </div>

        <div class="field">
          <label for="feriado_descricao">Descrição <span class="required">*</span></label>
          <input id="feriado_descricao" v-model="form.descricao" type="text" :disabled="viewMode" />
        </div>

        <div class="field">
          <label for="feriado_contexto">Abrangência principal</label>
          <select id="feriado_contexto" v-model="form.contexto_tipo" :disabled="viewMode">
            <option v-for="item in contextoOptions" :key="item.id" :value="item.label">{{ item.label }}</option>
          </select>
        </div>

        <div class="field">
          <label for="feriado_regra_jornada">Tratamento da jornada</label>
          <select id="feriado_regra_jornada" v-model="form.regra_jornada" :disabled="viewMode">
            <option :value="null">Selecione</option>
            <option v-for="item in regrasJornadaOptions" :key="item.id" :value="item.label">{{ item.label }}</option>
          </select>
        </div>

        <div class="field">
          <label for="feriado_regra_compensacao">Regra de compensação</label>
          <select id="feriado_regra_compensacao" v-model="form.regra_compensacao" :disabled="viewMode">
            <option :value="null">Selecione</option>
            <option v-for="item in regrasCompensacaoOptions" :key="item.id" :value="item.label">{{ item.label }}</option>
          </select>
        </div>

        <div class="field field-checkbox">
          <label>
            <input v-model="form.ativo" type="checkbox" :disabled="viewMode" />
            Feriado ativo
          </label>
        </div>

        <div class="field full-width">
          <label for="feriado_observacoes">Observações</label>
          <textarea id="feriado_observacoes" v-model="form.observacoes" rows="3" :disabled="viewMode" />
        </div>

        <div class="field full-width">
          <label>Empresas abrangidas</label>
          <div class="check-grid muted-box">
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

        <div class="field full-width">
          <label>Departamentos abrangidos</label>
          <div class="check-grid muted-box">
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
          <div class="muted small-text">Abrangência selecionada: {{ labelsFromIds(departamentosOptions, form.departamento_ids) || 'nenhum' }}</div>
        </div>

        <div class="actions" v-if="!viewMode">
          <button class="primary" type="submit" :disabled="saving">
            {{ saving ? 'Salvando...' : 'Salvar' }}
          </button>
          <button class="secondary" type="button" @click="resetForm">Limpar</button>
        </div>
      </form>
    </AppModal>
  </div>
</template>
