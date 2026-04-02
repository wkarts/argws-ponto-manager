<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import AppSwitch from "../components/AppSwitch.vue";
import { comboList, deleteBatida, exportBatidasCsv, listBatidas, listEmployees, saveBatida, type ComboOption } from "../services/crud";
import { useSessionStore } from "../stores/session";
import { logAppError, logAppInfo } from "../services/logger";

const session = useSessionStore();
const funcionarioOptions = ref<ComboOption[]>([]);
const equipamentoOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const rows = ref<Record<string, unknown>[]>([]);
const message = ref("");
const error = ref("");
const modalOpen = ref(false);

function textValue(value: unknown): string | number | readonly string[] | null | undefined {
  if (typeof value === "string" || typeof value === "number") return value;
  if (Array.isArray(value)) return value.filter((item): item is string => typeof item === "string");
  return value == null ? undefined : String(value);
}

function selectedFuncionarioId(): number | null {
  if (!filters.funcionarioId) return null;
  const parsed = Number(filters.funcionarioId);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}


const filters = reactive({
  funcionarioId: "",
  dataInicial: "",
  dataFinal: ""
});

const form = reactive<Record<string, unknown>>({
  id: undefined,
  funcionario_id: "",
  equipamento_id: "",
  justificativa_id: "",
  manual_ajuste: false,
  validado: true,
  data_referencia: "",
  hora: "",
  nsr: "",
  origem: "manual",
  observacao: "",
  tipo: "entrada"
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
  form.equipamento_id = "";
  form.justificativa_id = "";
  form.manual_ajuste = false;
  form.validado = true;
  form.data_referencia = "";
  form.hora = "";
  form.nsr = "";
  form.origem = "manual";
  form.observacao = "";
  form.tipo = "entrada";
}

async function loadCombos() {
  error.value = "";
  try {
    const employees = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
    funcionarioOptions.value = employees.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
    equipamentoOptions.value = await comboList("equipamentos");
    justificativaOptions.value = await comboList("justificativas");
    if (!filters.funcionarioId && funcionarioOptions.value.length) filters.funcionarioId = String(funcionarioOptions.value[0].id);
    if (!form.funcionario_id && session.activeCompanyId && funcionarioOptions.value.length) form.funcionario_id = funcionarioOptions.value[0].id;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar dados auxiliares de batidas.";
    logAppError("batidas", "Falha ao carregar combos de batidas.", { error: error.value });
  }
}

async function load() {
  error.value = "";
  try {
    rows.value = await listBatidas({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: selectedFuncionarioId(),
      dataInicial: filters.dataInicial || null,
      dataFinal: filters.dataFinal || null
    });
  } catch (err) {
    rows.value = [];
    error.value = err instanceof Error ? err.message : "Falha ao carregar batidas.";
    logAppError("batidas", "Falha ao listar batidas.", {
      error: error.value,
      filters: { ...filters, empresaId: session.activeCompanyId ?? null }
    });
  }
}

function editRow(row: Record<string, unknown>) {
  Object.assign(form, row);
  modalOpen.value = true;
}

async function persist() {
  message.value = "";
  error.value = "";
  try {
    await saveBatida({ ...form });
    await load();
    resetForm();
    closeModal();
    message.value = "Batida salva com sucesso.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar batida.";
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja remover esta batida?")) return;
  try {
    await deleteBatida(id);
    await load();
    logAppInfo("batidas", "Batida removida com sucesso.", { id });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao remover batida.";
    logAppError("batidas", "Falha ao remover batida.", { id, error: error.value });
  }
}

async function exportar() {
  error.value = "";
  try {
    const filePath = await exportBatidasCsv({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: selectedFuncionarioId(),
      dataInicial: filters.dataInicial || null,
      dataFinal: filters.dataFinal || null
    });
    message.value = `CSV gerado em: ${filePath}`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar CSV.";
  }
}

watch(() => session.activeCompanyId, async () => {
  await loadCombos();
  await load();
});

onMounted(async () => {
  try {
    await loadCombos();
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao inicializar a página de batidas.";
    logAppError("batidas", "Falha inesperada na inicialização da página de batidas.", { error: error.value });
  }
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2 style="margin: 0;">Batidas</h2>
        <div class="muted">Registro manual, ajustes autorizados e exportação local.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="openNewModal">Nova batida</button>
        <button class="secondary" @click="exportar">Exportar CSV</button>
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
      <h3 style="margin-top: 0;">Listagem</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Funcionário</th>
              <th>Equipamento</th>
              <th>Data</th>
              <th>Hora</th>
              <th>Tipo</th>
              <th>Origem</th>
              <th>Justificativa</th>
              <th>Ajuste</th>
              <th>Validado</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="String(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.funcionario_nome }}</td>
              <td>{{ row.equipamento_nome }}</td>
              <td>{{ row.data_referencia }}</td>
              <td>{{ row.hora }}</td>
              <td>{{ row.tipo }}</td>
              <td>{{ row.origem }}</td>
              <td>{{ row.justificativa_nome || '-' }}</td>
              <td>{{ Number(row.manual_ajuste) === 1 ? 'Sim' : 'Não' }}</td>
              <td>{{ Number(row.validado) === 1 ? 'Sim' : 'Não' }}</td>
              <td>
                <div class="actions compact-actions">
                  <button class="secondary" @click="editRow(row)">Editar</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="rows.length === 0">
              <td colspan="11" class="muted">Nenhuma batida encontrada.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <AppModal
      :open="modalOpen"
      :title="form.id ? 'Editar batida' : 'Nova batida'"
      subtitle="Inclusão e edição convertidas para modal, preservando a listagem principal."
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
            <label>Equipamento</label>
            <select v-model="form.equipamento_id">
              <option value="">Selecione</option>
              <option v-for="item in equipamentoOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Justificativa</label>
            <select v-model="form.justificativa_id">
              <option value="">Selecione</option>
              <option v-for="item in justificativaOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Data</label>
            <input v-model="form.data_referencia" type="date" />
          </div>
          <div class="field">
            <label>Hora</label>
            <input v-model="form.hora" type="time" />
          </div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Tipo</label>
            <select v-model="form.tipo">
              <option value="entrada">Entrada</option>
              <option value="saida">Saída</option>
              <option value="intervalo_saida">Intervalo saída</option>
              <option value="intervalo_retorno">Intervalo retorno</option>
              <option value="marcacao">Marcação importada</option>
            </select>
          </div>
          <div class="field">
            <label>NSR</label>
            <input v-model="form.nsr" type="text" />
          </div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Origem</label>
            <input v-model="form.origem" type="text" />
          </div>
          <div class="field">
            <label>Observação</label>
            <textarea :value="textValue(form.observacao)" rows="2" @input="form.observacao = ($event.target as HTMLTextAreaElement).value" />
          </div>
        </div>

        <div class="actions">
          <AppSwitch v-model="form.manual_ajuste" label="Ajuste manual autorizado" />
          <AppSwitch v-model="form.validado" label="Validado" />
        </div>

        <div class="actions">
          <button class="primary" type="submit">{{ form.id ? 'Atualizar' : 'Salvar' }}</button>
          <button class="secondary" type="button" @click="resetForm">Limpar</button>
        </div>
      </form>
    </AppModal>

  </div>
</template>
