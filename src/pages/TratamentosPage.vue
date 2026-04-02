<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import AppSwitch from "../components/AppSwitch.vue";
import { comboList, deleteOcorrencia, exportOcorrenciaAnexo, listEmployees, listOcorrencias, saveOcorrencia, type ComboOption } from "../services/crud";
import { RouterLink } from "vue-router";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const funcionarioOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const rows = ref<Record<string, unknown>[]>([]);
const message = ref("");
const error = ref("");
const modalOpen = ref(false);

type FormFieldValue = string | number | boolean | undefined;
type TextBindableValue = string | number | readonly string[] | null | undefined;

function textValue(value: unknown): TextBindableValue {
  if (typeof value === "string" || typeof value === "number") return value;
  if (Array.isArray(value)) return value.filter((item): item is string => typeof item === "string");
  return value == null ? undefined : String(value);
}

function switchValue(value: FormFieldValue): boolean | number | string | undefined {
  if (typeof value === "boolean" || typeof value === "number" || typeof value === "string") return value;
  return undefined;
}

function normalizeFormValue(value: unknown, fallback: FormFieldValue): FormFieldValue {
  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") return value;
  return value == null ? fallback : String(value);
}

const filters = reactive({
  funcionarioId: "",
  dataInicial: "",
  dataFinal: ""
});

const form = reactive<Record<string, FormFieldValue>>({
  id: undefined,
  funcionario_id: "",
  data_referencia: "",
  tipo: "atestado",
  justificativa_id: "",
  abonar_dia: true,
  minutos_abonados: 0,
  observacao: "",
  anexo_nome: "",
  anexo_mime: "",
  anexo_base64: ""
});

function closeModal() {
  modalOpen.value = false;
}

function openNewModal() {
  resetForm();
  if (funcionarioOptions.value.length && !form.funcionario_id) form.funcionario_id = funcionarioOptions.value[0].id;
  modalOpen.value = true;
}

function resetForm() {
  form.id = undefined;
  form.funcionario_id = "";
  form.data_referencia = "";
  form.tipo = "atestado";
  form.justificativa_id = "";
  form.abonar_dia = true;
  form.minutos_abonados = 0;
  form.observacao = "";
  form.anexo_nome = "";
  form.anexo_mime = "";
  form.anexo_base64 = "";
}

async function loadCombos() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
  funcionarioOptions.value = rows.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
  justificativaOptions.value = await comboList("justificativas");
  if (!filters.funcionarioId && funcionarioOptions.value.length) filters.funcionarioId = String(funcionarioOptions.value[0].id);
  if (!form.funcionario_id && funcionarioOptions.value.length) form.funcionario_id = funcionarioOptions.value[0].id;
}

async function load() {
  rows.value = await listOcorrencias({
    funcionarioId: filters.funcionarioId || null,
    dataInicial: filters.dataInicial || null,
    dataFinal: filters.dataFinal || null
  });
}

function editRow(row: Record<string, unknown>) {
  form.id = normalizeFormValue(row.id, undefined);
  form.funcionario_id = normalizeFormValue(row.funcionario_id, "");
  form.data_referencia = normalizeFormValue(row.data_referencia, "");
  form.tipo = normalizeFormValue(row.tipo, "atestado");
  form.justificativa_id = normalizeFormValue(row.justificativa_id, "");
  form.abonar_dia = normalizeFormValue(row.abonar_dia, true);
  form.minutos_abonados = normalizeFormValue(row.minutos_abonados, 0);
  form.observacao = normalizeFormValue(row.observacao, "");
  form.anexo_nome = normalizeFormValue(row.anexo_nome, "");
  form.anexo_mime = normalizeFormValue(row.anexo_mime, "");
  form.anexo_base64 = normalizeFormValue(row.anexo_base64, "");
  modalOpen.value = true;
}

async function persist() {
  message.value = "";
  error.value = "";
  try {
    await saveOcorrencia({ ...form });
    await load();
    resetForm();
    closeModal();
    message.value = "Ocorrência/justificativa salva com sucesso.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar ocorrência.";
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja excluir esta ocorrência?")) return;
  await deleteOcorrencia(id);
  await load();
}

async function exportarAnexo(id: number) {
  try {
    const filePath = await exportOcorrenciaAnexo(id);
    message.value = `Anexo exportado em: ${filePath}`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar anexo.";
  }
}

function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = () => {
    form.anexo_nome = file.name;
    form.anexo_mime = file.type || "application/octet-stream";
    form.anexo_base64 = typeof reader.result === "string" ? reader.result : "";
  };
  reader.readAsDataURL(file);
}

watch(() => session.activeCompanyId, async () => { await loadCombos(); await load(); });

onMounted(async () => {
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Tratamento de ponto</h2>
        <div class="muted">Justificativas, faltas, atestados, abonos e anexos. Para lançar batidas esquecidas ou autorizadas, use também a tela de <RouterLink to="/batidas">batidas</RouterLink>.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="openNewModal">Nova ocorrência</button>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-1">
      <div class="card">
        <h3 style="margin-top: 0;">Filtros</h3>
        <div class="grid">
          <div class="field">
            <label>Funcionário</label>
            <select v-model="filters.funcionarioId">
              <option value="">Todos</option>
              <option v-for="item in funcionarioOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Data inicial</label>
            <input v-model="filters.dataInicial" type="date" />
          </div>
          <div class="field">
            <label>Data final</label>
            <input v-model="filters.dataFinal" type="date" />
          </div>
          <div class="actions">
            <button class="secondary" @click="load">Filtrar</button>
          </div>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 style="margin-top: 0;">Ocorrências lançadas</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Funcionário</th>
              <th>Data</th>
              <th>Tipo</th>
              <th>Justificativa</th>
              <th>Abono dia</th>
              <th>Min. abonados</th>
              <th>Anexo</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="String(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.funcionario_nome }}</td>
              <td>{{ row.data_referencia }}</td>
              <td>{{ row.tipo }}</td>
              <td>{{ row.justificativa_nome || '-' }}</td>
              <td>{{ Number(row.abonar_dia) === 1 ? 'Sim' : 'Não' }}</td>
              <td>{{ row.minutos_abonados }}</td>
              <td>{{ row.anexo_nome || '-' }}</td>
              <td>
                <div class="actions compact-actions">
                  <button class="secondary" @click="editRow(row)">Editar</button>
                  <button v-if="row.anexo_nome" class="secondary" @click="exportarAnexo(Number(row.id))">Exportar anexo</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="9" class="muted">Nenhuma ocorrência lançada.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <AppModal
      :open="modalOpen"
      :title="form.id ? 'Editar ocorrência / justificativa' : 'Nova ocorrência / justificativa'"
      subtitle="Inclusão e edição convertidas para modal, preservando a listagem e os filtros atuais."
      width="lg"
      @close="closeModal"
    >
      <form class="grid" @submit.prevent="persist">
        <div class="field">
          <label>Funcionário</label>
          <select v-model="form.funcionario_id">
            <option value="">Selecione</option>
            <option v-for="item in funcionarioOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Data</label>
            <input v-model="form.data_referencia" type="date" />
          </div>
          <div class="field">
            <label>Tipo</label>
            <select v-model="form.tipo">
              <option value="atestado">Atestado</option>
              <option value="falta_justificada">Falta justificada</option>
              <option value="falta_nao_justificada">Falta não justificada</option>
              <option value="abono">Abono</option>
              <option value="ajuste_manual">Ajuste manual / esquecimento de marcação</option>
            </select>
          </div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Justificativa</label>
            <select v-model="form.justificativa_id">
              <option value="">Selecione</option>
              <option v-for="item in justificativaOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Minutos abonados</label>
            <input v-model="form.minutos_abonados" type="number" min="0" />
          </div>
        </div>

        <div class="actions">
          <AppSwitch :model-value="switchValue(form.abonar_dia)" label="Abonar o dia inteiro" @update:model-value="form.abonar_dia = $event" />
        </div>

        <div class="field">
          <label>Observação</label>
          <textarea :value="textValue(form.observacao)" rows="3" @input="form.observacao = ($event.target as HTMLTextAreaElement).value" />
        </div>

        <div class="field">
          <label>Anexo / atestado</label>
          <input type="file" @change="onFileSelected" />
          <small class="muted" v-if="form.anexo_nome">Arquivo selecionado: {{ String(form.anexo_nome) }}</small>
        </div>

        <div class="actions">
          <button class="primary" type="submit">{{ form.id ? 'Atualizar' : 'Salvar' }}</button>
          <button class="secondary" type="button" @click="resetForm">Limpar</button>
        </div>
      </form>
    </AppModal>

  </div>
</template>
