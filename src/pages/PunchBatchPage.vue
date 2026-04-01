<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import { deleteBatida, listBatidas, listEmployees, saveBatida, saveOcorrencia, type GenericRecord } from "../services/crud";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const employees = ref<{ id: number; nome: string }[]>([]);
const selectedEmployeeId = ref<number | null>(null);
const dataInicial = ref(new Date().toISOString().slice(0, 10));
const dataFinal = ref(new Date().toISOString().slice(0, 10));
const rows = ref<Record<string, any>[]>([]);
const justificativaId = ref<number | null>(null);
const observacao = ref("");
const message = ref("");
const error = ref("");
const loading = ref(false);
const modalOpen = ref(false);
const savingLine = ref(false);
const editingIndex = ref<number | null>(null);
const rowForm = ref<Record<string, any>>(emptyRow());

const currentEmployee = computed(() => employees.value.find((item) => item.id === selectedEmployeeId.value));

function emptyRow() {
  return {
    id: undefined,
    funcionario_id: selectedEmployeeId.value,
    data_referencia: dataInicial.value,
    hora: "08:00",
    tipo: "entrada",
    origem: "ajuste_manual",
    manual_ajuste: true,
    validado: true,
    observacao: observacao.value || "Ajuste manual em lote",
  };
}

async function loadEmployeesForCompany() {
  const rows = await listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true });
  employees.value = rows.map((item) => ({ id: Number(item.id), nome: String(item.nome || item.id) }));
  if (!selectedEmployeeId.value && employees.value.length) {
    selectedEmployeeId.value = employees.value[0].id;
  }
}

async function load() {
  if (!selectedEmployeeId.value) return;
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listBatidas({
      empresaId: session.activeCompanyId ?? null,
      funcionarioId: selectedEmployeeId.value,
      dataInicial: dataInicial.value,
      dataFinal: dataFinal.value,
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar batidas do colaborador.";
  } finally {
    loading.value = false;
  }
}

async function saveRow(row: GenericRecord) {
  await saveBatida({ ...row, funcionario_id: selectedEmployeeId.value });
}

async function removeRow(id: number) {
  if (!confirm("Remover esta batida?")) return;
  await deleteBatida(id);
  await load();
}

function addRow() {
  editingIndex.value = null;
  rowForm.value = emptyRow();
  modalOpen.value = true;
}

function editRow(row: Record<string, any>, index: number) {
  editingIndex.value = index;
  rowForm.value = {
    id: row.id,
    funcionario_id: selectedEmployeeId.value,
    data_referencia: row.data_referencia || dataInicial.value,
    hora: row.hora || "08:00",
    tipo: row.tipo || "entrada",
    origem: row.origem || "ajuste_manual",
    manual_ajuste: row.manual_ajuste ?? true,
    validado: row.validado ?? true,
    observacao: row.observacao || "",
  };
  modalOpen.value = true;
}

function closeModal() {
  modalOpen.value = false;
  editingIndex.value = null;
  rowForm.value = emptyRow();
}

function persistLocalRow() {
  const payload = {
    ...rowForm.value,
    funcionario_id: selectedEmployeeId.value,
    data_referencia: rowForm.value.data_referencia || dataInicial.value,
    hora: rowForm.value.hora || "08:00",
    tipo: rowForm.value.tipo || "entrada",
    origem: rowForm.value.origem || "ajuste_manual",
    manual_ajuste: rowForm.value.manual_ajuste ?? true,
    validado: rowForm.value.validado ?? true,
    observacao: rowForm.value.observacao || "",
  };

  if (editingIndex.value === null) {
    rows.value.unshift(payload);
  } else {
    rows.value.splice(editingIndex.value, 1, payload);
  }

  closeModal();
}

async function salvarLinhaModal() {
  if (!selectedEmployeeId.value) {
    error.value = "Selecione um colaborador antes de lançar uma batida.";
    return;
  }

  savingLine.value = true;
  error.value = "";
  try {
    persistLocalRow();
    message.value = "Linha preparada para o lote com sucesso.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao preparar a linha para o lote.";
  } finally {
    savingLine.value = false;
  }
}

async function salvarLote() {
  error.value = "";
  message.value = "";
  try {
    for (const row of rows.value) {
      await saveRow(row);
    }
    if (justificativaId.value) {
      await saveOcorrencia({
        funcionario_id: selectedEmployeeId.value,
        data_referencia: dataInicial.value,
        tipo: "ajuste_manual",
        justificativa_id: justificativaId.value,
        abonar_dia: false,
        minutos_abonados: 0,
        observacao: observacao.value || "Tratamento em lote de batidas",
      });
    }
    message.value = "Tratamento em lote salvo com sucesso.";
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar tratamento em lote.";
  }
}

watch([selectedEmployeeId, dataInicial, dataFinal], load);
watch(() => session.activeCompanyId, loadEmployeesForCompany);

onMounted(async () => {
  await loadEmployeesForCompany();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Tratamento em lote de batidas</h2>
        <div class="muted-text">Página centralizada para selecionar o colaborador, revisar várias batidas, aplicar justificativa e lançar ajustes de forma prática.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="addRow">Nova linha</button>
        <button class="primary" @click="salvarLote">Salvar lote</button>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="grid columns-4 mobile-columns-1">
        <div class="field">
          <label>Colaborador</label>
          <select v-model="selectedEmployeeId">
            <option v-for="item in employees" :key="String(item.id)" :value="Number(item.id)">{{ item.nome }}</option>
          </select>
        </div>
        <div class="field">
          <label>Data inicial</label>
          <input v-model="dataInicial" type="date" />
        </div>
        <div class="field">
          <label>Data final</label>
          <input v-model="dataFinal" type="date" />
        </div>
        <div class="field">
          <label>Justificativa padrão (ID)</label>
          <input v-model="justificativaId" type="number" min="1" />
        </div>
      </div>
      <div class="field">
        <label>Observação geral</label>
        <textarea v-model="observacao" rows="2" placeholder="Justificativa ou contexto do ajuste em lote" />
      </div>
      <div class="muted-text">Colaborador ativo: <strong>{{ currentEmployee?.nome || '-' }}</strong>. Empresa ativa: <strong>{{ session.activeCompanyName }}</strong>.</div>
    </div>

    <div class="card">
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Data</th>
              <th>Hora</th>
              <th>Tipo</th>
              <th>Origem</th>
              <th>Obs.</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, index) in rows" :key="String(row.id || `${row.data_referencia}-${row.hora}-${index}`)">
              <td>{{ row.data_referencia || '-' }}</td>
              <td>{{ row.hora || '-' }}</td>
              <td>{{ row.tipo || '-' }}</td>
              <td>{{ row.origem || '-' }}</td>
              <td>{{ row.observacao || '-' }}</td>
              <td>
                <div class="actions compact-actions">
                  <button class="secondary" @click="editRow(row, index)">Editar</button>
                  <button v-if="row.id" class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="!rows.length && !loading">
              <td colspan="6" class="empty-cell">Nenhuma batida encontrada para o filtro informado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <AppModal
      :open="modalOpen"
      :title="rowForm.id ? 'Editar batida em lote' : 'Nova batida em lote'"
      subtitle="A inclusão e a edição da linha agora seguem o padrão em modal, preservando o salvamento consolidado do lote."
      width="md"
      @close="closeModal"
    >
      <form class="grid" @submit.prevent="salvarLinhaModal">
        <div class="field">
          <label>Data</label>
          <input v-model="rowForm.data_referencia" type="date" required />
        </div>

        <div class="field">
          <label>Hora</label>
          <input v-model="rowForm.hora" type="time" required />
        </div>

        <div class="field">
          <label>Tipo</label>
          <select v-model="rowForm.tipo" required>
            <option value="entrada">Entrada</option>
            <option value="saida">Saída</option>
            <option value="intervalo_saida">Intervalo saída</option>
            <option value="intervalo_retorno">Intervalo retorno</option>
          </select>
        </div>

        <div class="field">
          <label>Origem</label>
          <input v-model="rowForm.origem" type="text" />
        </div>

        <div class="field full-width">
          <label>Observação</label>
          <input v-model="rowForm.observacao" type="text" />
        </div>

        <div class="grid columns-2 mobile-columns-1 full-width">
          <label class="checkbox-field">
            <input v-model="rowForm.manual_ajuste" type="checkbox" />
            <span>Ajuste manual</span>
          </label>
          <label class="checkbox-field">
            <input v-model="rowForm.validado" type="checkbox" />
            <span>Validado</span>
          </label>
        </div>

        <div class="actions modal-actions full-width">
          <button class="secondary" type="button" @click="closeModal">Cancelar</button>
          <button class="primary" type="submit" :disabled="savingLine">
            {{ savingLine ? 'Salvando...' : rowForm.id ? 'Atualizar linha' : 'Adicionar linha' }}
          </button>
        </div>
      </form>
    </AppModal>
  </div>
</template>
