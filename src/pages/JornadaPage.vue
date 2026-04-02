<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import AppModal from "../components/AppModal.vue";
import {
  comboList,
  deleteJornada,
  getJornada,
  listJornadas,
  saveJornada,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { booleanLabel, formatMinutes } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const rows = ref<GenericRecord[]>([]);
const companyOptions = ref<ComboOption[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const modalOpen = ref(false);

const dayLabels = ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];

function defaultDays() {
  return dayLabels.map((_, index) => ({
    dia_semana: index + 1,
    entrada_1: index < 5 ? "08:00" : "",
    saida_1: index < 5 ? "12:00" : "",
    entrada_2: index < 5 ? "13:00" : "",
    saida_2: index < 5 ? "17:00" : "",
    carga_prevista_minutos: index < 5 ? 480 : 0,
    intervalo_minutos: index < 5 ? 60 : 0,
    folga: index >= 5
  }));
}

function defaultForm() {
  return {
    id: undefined as number | undefined,
    empresa_id: "",
    codigo: "",
    descricao: "",
    tipo_jornada: "fixa",
    tolerancia_entrada_minutos: 5,
    tolerancia_saida_minutos: 5,
    tolerancia_intervalo_minutos: 5,
    carga_semanal_minutos: 2400,
    limite_diario_minutos: 600,
    banco_horas_ativo: true,
    exige_marcacao_intervalo: true,
    compensa_atraso_com_extra: true,
    modo_tratamento_afd: "auto",
    observacoes: "",
    ativo: true,
    dias: defaultDays()
  };
}

const form = reactive(defaultForm());

function closeModal() {
  modalOpen.value = false;
}

function openNewModal() {
  resetForm();
  if (session.activeCompanyId) form.empresa_id = String(session.activeCompanyId);
  modalOpen.value = true;
}

function resetForm() {
  Object.assign(form, defaultForm());
}

async function loadOptions() {
  companyOptions.value = await comboList("empresas");
}

const visibleRows = computed(() => rows.value.filter((row) => !session.activeCompanyId || !row.empresa_id || Number(row.empresa_id) === session.activeCompanyId));

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listJornadas();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar jornadas.";
  } finally {
    loading.value = false;
  }
}

async function editRow(id: number) {
  error.value = "";
  try {
    const record = await getJornada(id);
    Object.assign(form, defaultForm(), record, {
      empresa_id: record.empresa_id ? String(record.empresa_id) : "",
      banco_horas_ativo: Number(record.banco_horas_ativo) === 1 || record.banco_horas_ativo === true,
      exige_marcacao_intervalo: Number(record.exige_marcacao_intervalo) === 1 || record.exige_marcacao_intervalo === true,
      compensa_atraso_com_extra: Number(record.compensa_atraso_com_extra) === 1 || record.compensa_atraso_com_extra === true,
      ativo: Number(record.ativo) === 1 || record.ativo === true,
      dias: Array.isArray(record.dias)
        ? (record.dias as GenericRecord[]).map((day) => ({
            dia_semana: Number(day.dia_semana),
            entrada_1: String(day.entrada_1 || ""),
            saida_1: String(day.saida_1 || ""),
            entrada_2: String(day.entrada_2 || ""),
            saida_2: String(day.saida_2 || ""),
            carga_prevista_minutos: Number(day.carga_prevista_minutos || 0),
            intervalo_minutos: Number(day.intervalo_minutos || 0),
            folga: Number(day.folga) === 1 || day.folga === true
          }))
        : defaultDays()
    });
    modalOpen.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar jornada.";
  }
}

async function persist() {
  saving.value = true;
  error.value = "";
  try {
    await saveJornada({ ...form, dias: form.dias.map((day) => ({ ...day })) });
    await load();
    closeModal();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar jornada.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja excluir esta jornada?")) return;
  try {
    await deleteJornada(id);
    await load();
    if (Number(form.id) === id) {
      resetForm();
      closeModal();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao excluir jornada.";
  }
}

watch(() => session.activeCompanyId, (value) => { if (!form.empresa_id && value) form.empresa_id = String(value); });

onMounted(async () => {
  if (session.activeCompanyId) form.empresa_id = String(session.activeCompanyId);
  await loadOptions();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Cadastro de jornadas de trabalho</h2>
      </div>
      <div class="actions">
        <button class="secondary" @click="openNewModal">Nova jornada</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3>Jornadas cadastradas</h3>
          <div class="muted-text">Base semanal utilizada na apuração, no AFD e no banco de horas.</div>
        </div>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Empresa</th>
              <th>Código</th>
              <th>Descrição</th>
              <th>Tipo</th>
              <th>Carga semanal</th>
              <th>Dias</th>
              <th>Status</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in visibleRows" :key="Number(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.empresa_nome || 'Geral' }}</td>
              <td>{{ row.codigo || '-' }}</td>
              <td>{{ row.descricao }}</td>
              <td>{{ row.tipo_jornada || '-' }}</td>
              <td>{{ formatMinutes(Number(row.carga_semanal_minutos || 0)) }}</td>
              <td>{{ row.total_dias || 0 }}</td>
              <td>{{ booleanLabel(row.ativo) }}</td>
              <td>
                <div class="compact-actions actions">
                  <button class="secondary" @click="editRow(Number(row.id))">Editar</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="!visibleRows.length">
              <td colspan="9" class="empty-cell">Nenhuma jornada encontrada.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <AppModal
      :open="modalOpen"
      :title="form.id ? 'Editar jornada de trabalho' : 'Nova jornada de trabalho'"
      width="xl"
      @close="closeModal"
    >
      <div class="grid page-gap">
        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Empresa</label>
            <select v-model="form.empresa_id">
              <option value="">Todas / padrão</option>
              <option v-for="item in companyOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
          </div>
          <div class="field">
            <label>Código</label>
            <input v-model="form.codigo" type="text" placeholder="JRN-001" />
          </div>
          <div class="field">
            <label>Descrição *</label>
            <input v-model="form.descricao" type="text" placeholder="Jornada comercial" />
          </div>
          <div class="field">
            <label>Tipo</label>
            <select v-model="form.tipo_jornada">
              <option value="fixa">Fixa</option>
              <option value="flexivel">Flexível</option>
              <option value="12x36">12x36</option>
              <option value="livre">Livre</option>
            </select>
          </div>
        </div>

        <div class="grid columns-4 mobile-columns-1">
          <div class="field">
            <label>Tolerância entrada (min)</label>
            <input v-model="form.tolerancia_entrada_minutos" type="number" min="0" />
          </div>
          <div class="field">
            <label>Tolerância saída (min)</label>
            <input v-model="form.tolerancia_saida_minutos" type="number" min="0" />
          </div>
          <div class="field">
            <label>Tolerância intervalo (min)</label>
            <input v-model="form.tolerancia_intervalo_minutos" type="number" min="0" />
          </div>
          <div class="field">
            <label>Modo AFD</label>
            <select v-model="form.modo_tratamento_afd">
              <option value="auto">Automático</option>
              <option value="1510">Portaria 1.510/2009</option>
              <option value="671">Portaria 671/2021</option>
            </select>
          </div>
        </div>

        <div class="grid columns-3 mobile-columns-1">
          <div class="field">
            <label>Carga semanal (min)</label>
            <input v-model="form.carga_semanal_minutos" type="number" min="0" />
          </div>
          <div class="field">
            <label>Limite diário (min)</label>
            <input v-model="form.limite_diario_minutos" type="number" min="0" />
          </div>
          <div class="field">
            <label>Observações</label>
            <input v-model="form.observacoes" type="text" placeholder="Observações da jornada" />
          </div>
        </div>

        <div class="grid columns-2 mobile-columns-1">
          <label class="checkbox-line"><input v-model="form.banco_horas_ativo" class="checkbox-input" type="checkbox" /> Banco de horas ativo</label>
          <label class="checkbox-line"><input v-model="form.exige_marcacao_intervalo" class="checkbox-input" type="checkbox" /> Exige marcação de intervalo</label>
          <label class="checkbox-line"><input v-model="form.compensa_atraso_com_extra" class="checkbox-input" type="checkbox" /> Compensa atraso com extra</label>
          <label class="checkbox-line"><input v-model="form.ativo" class="checkbox-input" type="checkbox" /> Jornada ativa</label>
        </div>

        <div class="section-title">Distribuição semanal</div>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Dia</th>
                <th>Entrada 1</th>
                <th>Saída 1</th>
                <th>Entrada 2</th>
                <th>Saída 2</th>
                <th>Carga (min)</th>
                <th>Intervalo</th>
                <th>Folga</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(day, index) in form.dias" :key="day.dia_semana">
                <td>{{ dayLabels[index] }}</td>
                <td><input v-model="day.entrada_1" type="time" :disabled="day.folga" /></td>
                <td><input v-model="day.saida_1" type="time" :disabled="day.folga" /></td>
                <td><input v-model="day.entrada_2" type="time" :disabled="day.folga" /></td>
                <td><input v-model="day.saida_2" type="time" :disabled="day.folga" /></td>
                <td><input v-model="day.carga_prevista_minutos" type="number" min="0" :disabled="day.folga" /></td>
                <td><input v-model="day.intervalo_minutos" type="number" min="0" :disabled="day.folga" /></td>
                <td><input v-model="day.folga" type="checkbox" /></td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="actions">
          <button class="primary" :disabled="saving" @click="persist">{{ saving ? 'Salvando...' : form.id ? 'Atualizar jornada' : 'Salvar jornada' }}</button>
          <button class="secondary" @click="resetForm">Limpar</button>
        </div>
      </div>
    </AppModal>
  </div>
</template>
