<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import {
  comboJornadas,
  comboList,
  deleteEmployee,
  getEmployee,
  listEmployees,
  saveEmployee,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { booleanLabel, formatCpf, formatPhone, formatPis } from "../services/format";

const rows = ref<GenericRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const search = ref("");
const filterEmpresaId = ref<number | null>(null);
const onlyActive = ref(true);

const companyOptions = ref<ComboOption[]>([]);
const departamentoOptions = ref<ComboOption[]>([]);
const funcaoOptions = ref<ComboOption[]>([]);
const centroCustoOptions = ref<ComboOption[]>([]);
const horarioOptions = ref<ComboOption[]>([]);
const escalaOptions = ref<ComboOption[]>([]);
const jornadaOptions = ref<ComboOption[]>([]);

function defaultForm() {
  return {
    id: undefined as number | undefined,
    empresa_id: "",
    matricula: "",
    nome: "",
    nome_social: "",
    documento: "",
    rg: "",
    pis: "",
    email: "",
    telefone: "",
    celular: "",
    data_nascimento: "",
    data_admissao: "",
    data_demissao: "",
    sexo: "",
    estado_civil: "",
    cep: "",
    endereco: "",
    numero: "",
    complemento: "",
    bairro: "",
    cidade: "",
    estado: "",
    departamento_id: "",
    funcao_id: "",
    centro_custo_id: "",
    horario_id: "",
    escala_id: "",
    jornada_id: "",
    observacoes: "",
    ativo: true
  };
}

const form = reactive(defaultForm());

function resetForm() {
  Object.assign(form, defaultForm());
}

function toSelectValue(value: unknown): string {
  if (value === undefined || value === null || value === "") return "";
  return String(value);
}

async function loadOptions() {
  const [empresas, departamentos, funcoes, centros, horarios, escalas, jornadas] = await Promise.all([
    comboList("empresas"),
    comboList("departamentos"),
    comboList("funcoes"),
    comboList("centro_custos"),
    comboList("horarios"),
    comboList("escalas"),
    comboJornadas()
  ]);

  companyOptions.value = empresas;
  departamentoOptions.value = departamentos;
  funcaoOptions.value = funcoes;
  centroCustoOptions.value = centros;
  horarioOptions.value = horarios;
  escalaOptions.value = escalas;
  jornadaOptions.value = jornadas;
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listEmployees({
      search: search.value,
      empresaId: filterEmpresaId.value,
      onlyActive: onlyActive.value
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar funcionários.";
  } finally {
    loading.value = false;
  }
}

async function editRow(id: number) {
  error.value = "";
  try {
    const record = await getEmployee(id);
    Object.assign(form, defaultForm(), record, {
      empresa_id: toSelectValue(record.empresa_id),
      departamento_id: toSelectValue(record.departamento_id),
      funcao_id: toSelectValue(record.funcao_id),
      centro_custo_id: toSelectValue(record.centro_custo_id),
      horario_id: toSelectValue(record.horario_id),
      escala_id: toSelectValue(record.escala_id),
      jornada_id: toSelectValue(record.jornada_id),
      ativo: Number(record.ativo) === 1 || record.ativo === true
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar funcionário.";
  }
}

async function persist() {
  saving.value = true;
  error.value = "";
  try {
    await saveEmployee({ ...form });
    await load();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar funcionário.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja excluir este funcionário?")) return;
  try {
    await deleteEmployee(id);
    await load();
    if (Number(form.id) === id) {
      resetForm();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao excluir funcionário.";
  }
}

onMounted(async () => {
  try {
    await loadOptions();
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao inicializar cadastro de funcionários.";
  }
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Cadastro de funcionários</h2>
        <div class="muted-text">Gerencie dados cadastrais, lotação, jornada de trabalho e vínculos operacionais.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="resetForm">Novo cadastro</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="section-title">Vínculo e identificação</div>
      <div class="grid columns-2 mobile-columns-1">
        <div class="field">
          <label>Empresa *</label>
          <select v-model="form.empresa_id">
            <option value="">Selecione</option>
            <option v-for="item in companyOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Matrícula *</label>
          <input v-model="form.matricula" type="text" placeholder="Matrícula interna" />
        </div>
        <div class="field">
          <label>Nome completo *</label>
          <input v-model="form.nome" type="text" placeholder="Nome do funcionário" />
        </div>
        <div class="field">
          <label>Nome social</label>
          <input v-model="form.nome_social" type="text" placeholder="Nome social" />
        </div>
        <div class="field">
          <label>CPF *</label>
          <input v-model="form.documento" type="text" placeholder="000.000.000-00" />
        </div>
        <div class="field">
          <label>RG</label>
          <input v-model="form.rg" type="text" placeholder="RG" />
        </div>
        <div class="field">
          <label>PIS / PASEP</label>
          <input v-model="form.pis" type="text" placeholder="000.00000.00-0" />
        </div>
        <div class="grid columns-2 nested-grid mobile-columns-1">
          <div class="field">
            <label>Sexo</label>
            <select v-model="form.sexo">
              <option value="">Selecione</option>
              <option value="M">Masculino</option>
              <option value="F">Feminino</option>
              <option value="O">Outro</option>
            </select>
          </div>
          <div class="field">
            <label>Estado civil</label>
            <select v-model="form.estado_civil">
              <option value="">Selecione</option>
              <option value="solteiro">Solteiro(a)</option>
              <option value="casado">Casado(a)</option>
              <option value="divorciado">Divorciado(a)</option>
              <option value="viuvo">Viúvo(a)</option>
              <option value="uniao_estavel">União estável</option>
            </select>
          </div>
        </div>
      </div>

      <div class="section-title">Contato e datas</div>
      <div class="grid columns-2 mobile-columns-1">
        <div class="field">
          <label>E-mail</label>
          <input v-model="form.email" type="email" placeholder="funcionario@empresa.com" />
        </div>
        <div class="grid columns-2 nested-grid mobile-columns-1">
          <div class="field">
            <label>Telefone</label>
            <input v-model="form.telefone" type="text" placeholder="(00) 0000-0000" />
          </div>
          <div class="field">
            <label>Celular</label>
            <input v-model="form.celular" type="text" placeholder="(00) 00000-0000" />
          </div>
        </div>
        <div class="field">
          <label>Data de nascimento</label>
          <input v-model="form.data_nascimento" type="date" />
        </div>
        <div class="grid columns-2 nested-grid mobile-columns-1">
          <div class="field">
            <label>Data de admissão *</label>
            <input v-model="form.data_admissao" type="date" />
          </div>
          <div class="field">
            <label>Data de desligamento</label>
            <input v-model="form.data_demissao" type="date" />
          </div>
        </div>
      </div>

      <div class="section-title">Lotação, horário e jornada</div>
      <div class="grid columns-2 mobile-columns-1">
        <div class="field">
          <label>Departamento</label>
          <select v-model="form.departamento_id">
            <option value="">Selecione</option>
            <option v-for="item in departamentoOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Função</label>
          <select v-model="form.funcao_id">
            <option value="">Selecione</option>
            <option v-for="item in funcaoOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Centro de custo</label>
          <select v-model="form.centro_custo_id">
            <option value="">Selecione</option>
            <option v-for="item in centroCustoOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Jornada de trabalho</label>
          <select v-model="form.jornada_id">
            <option value="">Selecione</option>
            <option v-for="item in jornadaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Horário auxiliar</label>
          <select v-model="form.horario_id">
            <option value="">Selecione</option>
            <option v-for="item in horarioOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Escala auxiliar</label>
          <select v-model="form.escala_id">
            <option value="">Selecione</option>
            <option v-for="item in escalaOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
      </div>

      <div class="section-title">Endereço e observações</div>
      <div class="grid columns-2 mobile-columns-1">
        <div class="field">
          <label>CEP</label>
          <input v-model="form.cep" type="text" />
        </div>
        <div class="grid columns-2 nested-grid mobile-columns-1">
          <div class="field">
            <label>UF</label>
            <input v-model="form.estado" type="text" maxlength="2" />
          </div>
          <div class="field">
            <label>Cidade</label>
            <input v-model="form.cidade" type="text" />
          </div>
        </div>
        <div class="field">
          <label>Endereço</label>
          <input v-model="form.endereco" type="text" />
        </div>
        <div class="grid columns-2 nested-grid mobile-columns-1">
          <div class="field">
            <label>Número</label>
            <input v-model="form.numero" type="text" />
          </div>
          <div class="field">
            <label>Complemento</label>
            <input v-model="form.complemento" type="text" />
          </div>
        </div>
        <div class="field">
          <label>Bairro</label>
          <input v-model="form.bairro" type="text" />
        </div>
        <div class="field span-2">
          <label>Observações</label>
          <textarea v-model="form.observacoes" rows="3" />
        </div>
        <div class="field checkbox-line span-2">
          <input v-model="form.ativo" class="checkbox-input" type="checkbox" />
          <label>Funcionário ativo</label>
        </div>
      </div>

      <div class="actions">
        <button class="primary" :disabled="saving" @click="persist">
          {{ saving ? "Salvando..." : form.id ? "Atualizar funcionário" : "Salvar funcionário" }}
        </button>
        <button class="secondary" @click="resetForm">Limpar</button>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3>Funcionários cadastrados</h3>
          <div class="muted-text">Consulte vínculos, documentos e jornada atribuída.</div>
        </div>
        <div class="actions align-end">
          <div class="field min-field">
            <label>Buscar</label>
            <input v-model="search" type="text" placeholder="Nome, matrícula ou CPF" @keyup.enter="load" />
          </div>
          <div class="field min-field">
            <label>Empresa</label>
            <select v-model="filterEmpresaId">
              <option :value="null">Todas</option>
              <option v-for="item in companyOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
            </select>
          </div>
          <div class="field checkbox-line compact-checkbox">
            <input v-model="onlyActive" class="checkbox-input" type="checkbox" />
            <label>Somente ativos</label>
          </div>
          <button class="secondary" :disabled="loading" @click="load">{{ loading ? "Carregando..." : "Atualizar" }}</button>
        </div>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Funcionário</th>
              <th>Empresa</th>
              <th>Documentos</th>
              <th>Contato</th>
              <th>Jornada</th>
              <th>Status</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="Number(row.id)">
              <td>{{ row.id }}</td>
              <td>
                <div><strong>{{ row.nome }}</strong></div>
                <div class="muted-row">Matrícula: {{ row.matricula || '-' }}</div>
              </td>
              <td>{{ row.empresa_nome || '-' }}</td>
              <td>
                <div>CPF: {{ formatCpf(row.documento) || '-' }}</div>
                <div class="muted-row">PIS: {{ formatPis(row.pis) || '-' }}</div>
              </td>
              <td>
                <div>{{ row.email || '-' }}</div>
                <div class="muted-row">{{ formatPhone(row.celular || row.telefone) || '-' }}</div>
              </td>
              <td>
                <div>{{ row.jornada_nome || '-' }}</div>
                <div class="muted-row">Horário: {{ row.horario_nome || '-' }}</div>
              </td>
              <td>{{ booleanLabel(row.ativo) }}</td>
              <td>
                <div class="compact-actions actions">
                  <button class="secondary" @click="editRow(Number(row.id))">Editar</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="8" class="empty-cell">Nenhum funcionário encontrado.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
