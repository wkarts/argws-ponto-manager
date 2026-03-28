<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { comboList, listBatidas, listEmployees, listOcorrencias, saveBatida, saveOcorrencia, type ComboOption, type GenericRecord } from "../services/crud";
import { logAppError, logAppInfo } from "../services/logger";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const savingBatida = ref(false);
const savingOcorrencia = ref(false);
const error = ref("");
const message = ref("");

const employeeOptions = ref<ComboOption[]>([]);
const justificativaOptions = ref<ComboOption[]>([]);
const batidas = ref<GenericRecord[]>([]);
const ocorrencias = ref<GenericRecord[]>([]);

const filtros = reactive({
  funcionarioId: "",
  dataInicial: new Date().toISOString().slice(0, 10),
  dataFinal: new Date().toISOString().slice(0, 10),
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
    const [employees, justificativas] = await Promise.all([
      listEmployees({ empresaId: session.activeCompanyId ?? null, onlyActive: true }),
      comboList("justificativas"),
    ]);
    employeeOptions.value = employees.map((item) => ({ id: Number(item.id), label: String(item.nome || item.id) }));
    justificativaOptions.value = justificativas;

    if (!filtros.funcionarioId && employeeOptions.value.length > 0) {
      filtros.funcionarioId = String(employeeOptions.value[0].id);
    }

    if (!batidaForm.funcionario_id) batidaForm.funcionario_id = filtros.funcionarioId;
    if (!ocorrenciaForm.funcionario_id) ocorrenciaForm.funcionario_id = filtros.funcionarioId;
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
    const [rowsBatida, rowsOcorrencia] = await Promise.all([
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
    ]);
    batidas.value = rowsBatida;
    ocorrencias.value = rowsOcorrencia;
  } catch (err) {
    batidas.value = [];
    ocorrencias.value = [];
    error.value = err instanceof Error ? err.message : "Falha ao carregar o cartão de ponto.";
    logAppError("cartao_ponto", "Falha ao carregar visão operacional do cartão.", {
      error: error.value,
      filtros: { ...filtros, funcionarioId: funcionarioIdNumero.value, empresaId: session.activeCompanyId ?? null },
    });
  } finally {
    loading.value = false;
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
    await carregarCartao();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar ocorrência.";
    logAppError("cartao_ponto", "Falha ao salvar ocorrência.", { error: error.value, payload: { ...ocorrenciaForm } });
  } finally {
    savingOcorrencia.value = false;
  }
}

function editarBatida(row: GenericRecord) {
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
  ocorrenciaForm.id = Number(row.id);
  ocorrenciaForm.funcionario_id = String(row.funcionario_id || filtros.funcionarioId || "");
  ocorrenciaForm.data_referencia = String(row.data_referencia || filtros.dataInicial);
  ocorrenciaForm.justificativa_id = row.justificativa_id ? String(row.justificativa_id) : "";
  ocorrenciaForm.tipo = String(row.tipo || "ajuste_manual");
  ocorrenciaForm.abonar_dia = Number(row.abonar_dia) === 1 || row.abonar_dia === true;
  ocorrenciaForm.minutos_abonados = Number(row.minutos_abonados || 0);
  ocorrenciaForm.observacao = String(row.observacao || "");
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
        <div class="actions align-end">
          <button class="primary" :disabled="loading" @click="carregarCartao">Aplicar filtros</button>
        </div>
      </div>
    </div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Lançamento/edição de marcação</div>
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
          <div class="field checkbox-line"><input v-model="batidaForm.manual_ajuste" class="checkbox-input" type="checkbox" /><label>Ajuste manual</label></div>
          <div class="field checkbox-line"><input v-model="batidaForm.validado" class="checkbox-input" type="checkbox" /><label>Validado</label></div>
        </div>
        <div class="actions">
          <button class="primary" :disabled="savingBatida" @click="salvarBatida">{{ savingBatida ? 'Salvando...' : batidaForm.id ? 'Atualizar marcação' : 'Salvar marcação' }}</button>
          <button class="secondary" @click="resetBatida">Limpar</button>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Justificativas e ocorrências</div>
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
          <div class="field checkbox-line"><input v-model="ocorrenciaForm.abonar_dia" class="checkbox-input" type="checkbox" /><label>Abonar dia</label></div>
        </div>
        <div class="actions">
          <button class="primary" :disabled="savingOcorrencia" @click="salvarOcorrencia">{{ savingOcorrencia ? 'Salvando...' : ocorrenciaForm.id ? 'Atualizar ocorrência' : 'Salvar ocorrência' }}</button>
          <button class="secondary" @click="resetOcorrencia">Limpar</button>
        </div>
      </div>
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
            <td><button class="secondary" @click="editarBatida(row)">Editar</button></td>
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
            <td><button class="secondary" @click="editarOcorrencia(row)">Editar</button></td>
          </tr>
          <tr v-if="!ocorrencias.length">
            <td colspan="6" class="empty-cell">Nenhuma ocorrência encontrada para o período.</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
