<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { deleteCompany, getCompany, listCompanies, saveCompany, type GenericRecord } from "../services/crud";
import { booleanLabel, formatCpfCnpj, formatPhone } from "../services/format";

const rows = ref<GenericRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const search = ref("");
const onlyActive = ref(false);

function defaultForm() {
  return {
    id: undefined as number | undefined,
    nome: "",
    nome_fantasia: "",
    documento: "",
    inscricao_estadual: "",
    inscricao_municipal: "",
    telefone: "",
    email: "",
    responsavel_nome: "",
    responsavel_telefone: "",
    cep: "",
    endereco: "",
    numero: "",
    complemento: "",
    bairro: "",
    cidade: "",
    estado: "",
    observacoes: "",
    ativo: true
  };
}

const form = reactive(defaultForm());

function resetForm() {
  Object.assign(form, defaultForm());
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listCompanies({
      search: search.value,
      onlyActive: onlyActive.value
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar empresas.";
  } finally {
    loading.value = false;
  }
}

async function editRow(id: number) {
  error.value = "";
  try {
    const record = await getCompany(id);
    Object.assign(form, defaultForm(), record, {
      ativo: Number(record.ativo) === 1 || record.ativo === true
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar empresa.";
  }
}

async function persist() {
  saving.value = true;
  error.value = "";
  try {
    await saveCompany({ ...form });
    await load();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar empresa.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!confirm("Deseja excluir esta empresa usuária?")) return;

  try {
    await deleteCompany(id);
    await load();
    if (Number(form.id) === id) {
      resetForm();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao excluir empresa.";
  }
}

onMounted(load);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Cadastro de empresa usuária</h2>
        <div class="muted-text">Cadastre a empresa que utiliza o sistema com dados fiscais, contato e endereço.</div>
      </div>
      <div class="actions">
        <button class="secondary" @click="resetForm">Novo cadastro</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="section-title">Dados principais</div>
      <div class="grid columns-2">
        <div class="field">
          <label>Razão social *</label>
          <input v-model="form.nome" type="text" placeholder="Razão social da empresa" />
        </div>
        <div class="field">
          <label>Nome fantasia</label>
          <input v-model="form.nome_fantasia" type="text" placeholder="Nome fantasia" />
        </div>
        <div class="field">
          <label>CNPJ / CPF *</label>
          <input v-model="form.documento" type="text" placeholder="00.000.000/0000-00" />
        </div>
        <div class="grid columns-2 nested-grid">
          <div class="field">
            <label>Inscrição estadual</label>
            <input v-model="form.inscricao_estadual" type="text" placeholder="Inscrição estadual" />
          </div>
          <div class="field">
            <label>Inscrição municipal</label>
            <input v-model="form.inscricao_municipal" type="text" placeholder="Inscrição municipal" />
          </div>
        </div>
      </div>

      <div class="section-title">Contato</div>
      <div class="grid columns-2">
        <div class="field">
          <label>Telefone principal</label>
          <input v-model="form.telefone" type="text" placeholder="(00) 00000-0000" />
        </div>
        <div class="field">
          <label>E-mail</label>
          <input v-model="form.email" type="email" placeholder="contato@empresa.com" />
        </div>
        <div class="field">
          <label>Responsável</label>
          <input v-model="form.responsavel_nome" type="text" placeholder="Nome do responsável" />
        </div>
        <div class="field">
          <label>Telefone do responsável</label>
          <input v-model="form.responsavel_telefone" type="text" placeholder="(00) 00000-0000" />
        </div>
      </div>

      <div class="section-title">Endereço</div>
      <div class="grid columns-2">
        <div class="field">
          <label>CEP</label>
          <input v-model="form.cep" type="text" placeholder="00000-000" />
        </div>
        <div class="grid columns-2 nested-grid">
          <div class="field">
            <label>UF</label>
            <input v-model="form.estado" type="text" maxlength="2" placeholder="BA" />
          </div>
          <div class="field">
            <label>Cidade</label>
            <input v-model="form.cidade" type="text" placeholder="Cidade" />
          </div>
        </div>
        <div class="field">
          <label>Endereço</label>
          <input v-model="form.endereco" type="text" placeholder="Rua / Avenida" />
        </div>
        <div class="grid columns-2 nested-grid">
          <div class="field">
            <label>Número</label>
            <input v-model="form.numero" type="text" placeholder="Número" />
          </div>
          <div class="field">
            <label>Complemento</label>
            <input v-model="form.complemento" type="text" placeholder="Complemento" />
          </div>
        </div>
        <div class="field">
          <label>Bairro</label>
          <input v-model="form.bairro" type="text" placeholder="Bairro" />
        </div>
      </div>

      <div class="section-title">Observações</div>
      <div class="grid columns-2">
        <div class="field span-2">
          <label>Observações</label>
          <textarea v-model="form.observacoes" rows="4" placeholder="Informações adicionais da empresa"></textarea>
        </div>
        <div class="field checkbox-line span-2">
          <input v-model="form.ativo" class="checkbox-input" type="checkbox" />
          <label>Empresa ativa</label>
        </div>
      </div>

      <div class="actions">
        <button class="primary" :disabled="saving" @click="persist">
          {{ saving ? "Salvando..." : form.id ? "Atualizar empresa" : "Salvar empresa" }}
        </button>
        <button class="secondary" @click="resetForm">Limpar</button>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3>Empresas cadastradas</h3>
          <div class="muted-text">Use a listagem para localizar, editar e revisar empresas usuárias.</div>
        </div>
        <div class="actions align-end">
          <div class="field min-field">
            <label>Buscar</label>
            <input v-model="search" type="text" placeholder="Nome, documento ou cidade" @keyup.enter="load" />
          </div>
          <div class="field checkbox-line compact-checkbox">
            <input v-model="onlyActive" class="checkbox-input" type="checkbox" />
            <label>Somente ativas</label>
          </div>
          <button class="secondary" :disabled="loading" @click="load">
            {{ loading ? "Carregando..." : "Atualizar" }}
          </button>
        </div>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Razão social</th>
              <th>Fantasia</th>
              <th>Documento</th>
              <th>Contato</th>
              <th>Cidade/UF</th>
              <th>Status</th>
              <th>Ações</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="Number(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.nome }}</td>
              <td>{{ row.nome_fantasia || '-' }}</td>
              <td>{{ formatCpfCnpj(row.documento) }}</td>
              <td>
                <div>{{ formatPhone(row.telefone) || '-' }}</div>
                <div class="muted-row">{{ row.email || '-' }}</div>
              </td>
              <td>{{ row.cidade || '-' }} / {{ row.estado || '-' }}</td>
              <td>{{ booleanLabel(row.ativo) }}</td>
              <td>
                <div class="compact-actions actions">
                  <button class="secondary" @click="editRow(Number(row.id))">Editar</button>
                  <button class="danger" @click="removeRow(Number(row.id))">Excluir</button>
                </div>
              </td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="8" class="empty-cell">Nenhuma empresa encontrada.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
