<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  comboList,
  deleteUser,
  getUser,
  listProfiles,
  listUsers,
  saveUser,
  type ComboOption,
  type GenericRecord
} from "../services/crud";
import { booleanLabel, formatPhone } from "../services/format";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const rows = ref<GenericRecord[]>([]);
const loading = ref(false);
const saving = ref(false);
const error = ref("");
const search = ref("");
const onlyActive = ref(true);
const filterEmpresaId = ref<number | null>(null);

const companyOptions = ref<ComboOption[]>([]);
const profileOptions = ref<{ id: number; label: string }[]>([]);

function defaultForm() {
  return {
    id: undefined as number | undefined,
    nome: "",
    login: "",
    email: "",
    telefone: "",
    cargo: "",
    observacoes: "",
    senha: "",
    master_user: false,
    administrador: false,
    senha_provisoria: false,
    ativo: true,
    empresa_ids: [] as string[],
    profile_ids: [] as string[]
  };
}

const form = reactive(defaultForm());

const canManage = computed(() => session.can("usuarios:manage"));

function resetForm() {
  Object.assign(form, defaultForm());
}

function toStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.map((item) => String(item));
}

async function loadOptions() {
  const [empresas, perfis] = await Promise.all([
    comboList("empresas"),
    listProfiles(session.sessionToken!, { onlyActive: true })
  ]);
  companyOptions.value = empresas;
  profileOptions.value = perfis.map((item) => ({
    id: Number(item.id),
    label: String(item.nome || item.label || `Perfil ${item.id}`)
  }));
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listUsers(session.sessionToken!, {
      search: search.value,
      onlyActive: onlyActive.value,
      empresaId: filterEmpresaId.value
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar usuários.";
  } finally {
    loading.value = false;
  }
}

async function editRow(id: number) {
  error.value = "";
  try {
    const record = await getUser(session.sessionToken!, id);
    Object.assign(form, defaultForm(), record, {
      master_user: Number(record.master_user) === 1 || record.master_user === true,
      administrador: Number(record.administrador) === 1 || record.administrador === true,
      senha_provisoria: Number(record.senha_provisoria) === 1 || record.senha_provisoria === true,
      ativo: Number(record.ativo) === 1 || record.ativo === true,
      empresa_ids: toStringArray(record.empresa_ids),
      profile_ids: toStringArray(record.profile_ids),
      senha: ""
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar usuário.";
  }
}

async function persist() {
  if (!canManage.value) return;
  saving.value = true;
  error.value = "";
  try {
    await saveUser(session.sessionToken!, {
      ...form,
      empresa_ids: form.empresa_ids.map((item) => Number(item)),
      profile_ids: form.profile_ids.map((item) => Number(item))
    });
    await load();
    resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar usuário.";
  } finally {
    saving.value = false;
  }
}

async function removeRow(id: number) {
  if (!canManage.value) return;
  if (!confirm("Deseja excluir este usuário?")) return;
  try {
    await deleteUser(session.sessionToken!, id);
    await load();
    if (Number(form.id) === id) resetForm();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao excluir usuário.";
  }
}

onMounted(async () => {
  try {
    await loadOptions();
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao inicializar cadastro de usuários.";
  }
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Cadastro de usuários</h2>
        <div class="muted-text">Estrutura completa com usuário master, perfis múltiplos, empresas vinculadas e senha provisória.</div>
      </div>
      <div class="actions">
        <button class="secondary" :disabled="!canManage" @click="resetForm">Novo cadastro</button>
      </div>
    </div>

    <div v-if="!session.can('usuarios:view')" class="alert error">Você não possui permissão para visualizar usuários.</div>
    <div v-else class="grid page-gap">
      <div v-if="error" class="alert error">{{ error }}</div>

      <div class="card grid page-gap">
        <div class="section-title">Dados do acesso</div>
        <div class="grid columns-2 mobile-columns-1">
          <div class="field">
            <label>Nome *</label>
            <input v-model="form.nome" type="text" :disabled="!canManage" placeholder="Nome completo do usuário" />
          </div>
          <div class="field">
            <label>Login *</label>
            <input v-model="form.login" type="text" :disabled="!canManage" placeholder="login" />
          </div>
          <div class="field">
            <label>E-mail</label>
            <input v-model="form.email" type="email" :disabled="!canManage" placeholder="usuario@empresa.com" />
          </div>
          <div class="field">
            <label>Telefone</label>
            <input v-model="form.telefone" type="text" :disabled="!canManage" placeholder="(00) 00000-0000" />
          </div>
          <div class="field">
            <label>Cargo</label>
            <input v-model="form.cargo" type="text" :disabled="!canManage" placeholder="Cargo / função" />
          </div>
          <div class="field">
            <label>{{ form.id ? 'Nova senha (opcional)' : 'Senha *' }}</label>
            <input v-model="form.senha" type="password" :disabled="!canManage" placeholder="mínimo 6 caracteres" />
          </div>
        </div>

        <div class="section-title">Perfis e empresas</div>
        <div class="grid columns-2 mobile-columns-1">
          <div class="field span-2">
            <label>Perfis de acesso</label>
            <select v-model="form.profile_ids" multiple size="5" :disabled="!canManage || form.master_user">
              <option v-for="item in profileOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
            <div class="muted-row">Segure Ctrl/Cmd para selecionar múltiplos perfis.</div>
          </div>
          <div class="field span-2">
            <label>Empresas vinculadas</label>
            <select v-model="form.empresa_ids" multiple size="5" :disabled="!canManage || form.master_user">
              <option v-for="item in companyOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
            </select>
            <div class="muted-row">Usuário master pode operar sem vínculo específico de empresa.</div>
          </div>
        </div>

        <div class="section-title">Status e observações</div>
        <div class="grid columns-2 mobile-columns-1">
          <div class="field span-2">
            <label>Observações</label>
            <textarea v-model="form.observacoes" rows="4" :disabled="!canManage" placeholder="Observações internas sobre o usuário"></textarea>
          </div>
          <div class="field checkbox-line"><input v-model="form.master_user" class="checkbox-input" type="checkbox" :disabled="!canManage" /><label>Usuário master</label></div>
          <div class="field checkbox-line"><input v-model="form.administrador" class="checkbox-input" type="checkbox" :disabled="!canManage" /><label>Administrador</label></div>
          <div class="field checkbox-line"><input v-model="form.senha_provisoria" class="checkbox-input" type="checkbox" :disabled="!canManage" /><label>Senha provisória / exigir troca</label></div>
          <div class="field checkbox-line"><input v-model="form.ativo" class="checkbox-input" type="checkbox" :disabled="!canManage" /><label>Usuário ativo</label></div>
        </div>

        <div class="actions">
          <button class="primary" :disabled="saving || !canManage" @click="persist">
            {{ saving ? "Salvando..." : form.id ? "Atualizar usuário" : "Salvar usuário" }}
          </button>
          <button class="secondary" @click="resetForm">Limpar</button>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="toolbar">
          <div>
            <h3>Usuários cadastrados</h3>
            <div class="muted-text">Controle de login, sessão e vínculo de perfis por empresa.</div>
          </div>
          <div class="actions align-end">
            <div class="field min-field">
              <label>Empresa</label>
              <select v-model="filterEmpresaId">
                <option :value="null">Todas</option>
                <option v-for="item in companyOptions" :key="item.id" :value="item.id">{{ item.label }}</option>
              </select>
            </div>
            <div class="field min-field">
              <label>Buscar</label>
              <input v-model="search" type="text" placeholder="Nome, login ou e-mail" @keyup.enter="load" />
            </div>
            <div class="field checkbox-line compact-checkbox"><input v-model="onlyActive" class="checkbox-input" type="checkbox" /><label>Somente ativos</label></div>
            <button class="secondary" :disabled="loading" @click="load">{{ loading ? "Carregando..." : "Atualizar" }}</button>
          </div>
        </div>

        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Usuário</th>
                <th>Contato</th>
                <th>Perfis</th>
                <th>Empresas</th>
                <th>Status</th>
                <th>Último login</th>
                <th>Ações</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="Number(row.id)">
                <td>{{ row.id }}</td>
                <td>
                  <strong>{{ row.nome }}</strong>
                  <div class="muted-row">{{ row.login }}</div>
                  <div class="pill-box top-gap-6">
                    <span v-if="Number(row.master_user) === 1 || row.master_user === true" class="status-pill pill-master">MASTER</span>
                    <span v-if="Number(row.administrador) === 1 || row.administrador === true" class="status-pill pill-secondary">ADMIN</span>
                    <span v-if="Number(row.senha_provisoria) === 1 || row.senha_provisoria === true" class="status-pill pill-warning">SENHA PROVISÓRIA</span>
                  </div>
                </td>
                <td>
                  <div>{{ row.email || '-' }}</div>
                  <div class="muted-row">{{ formatPhone(row.telefone) || '-' }}</div>
                </td>
                <td>{{ row.perfis || '-' }}</td>
                <td>{{ row.empresas || '-' }}</td>
                <td>{{ booleanLabel(row.ativo) }}</td>
                <td>{{ row.ultimo_login_em || '-' }}</td>
                <td>
                  <div class="table-actions">
                    <button class="secondary small" @click="editRow(Number(row.id))">Editar</button>
                    <button class="danger small" :disabled="!canManage" @click="removeRow(Number(row.id))">Excluir</button>
                  </div>
                </td>
              </tr>
              <tr v-if="!rows.length">
                <td colspan="8" class="empty-cell">Nenhum usuário encontrado.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
