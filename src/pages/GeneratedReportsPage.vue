<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { comboList, downloadGeneratedReport, listGeneratedReports, listUsers, type GenericRecord } from "../services/crud";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const rows = ref<GenericRecord[]>([]);
const error = ref("");
const users = ref<GenericRecord[]>([]);
const funcionarios = ref<{ id: number; label: string }[]>([]);

const filters = reactive({
  competencia: "",
  funcionarioNome: "",
  tipoRelatorio: "",
  formato: "",
  usuarioLogin: "",
});

const tipos = computed(() => Array.from(new Set(rows.value.map((r) => String(r.tipo_relatorio || "")).filter(Boolean))).sort());
const formatos = ["PDF", "HTML", "EXCEL", "XML", "TXT"];

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listGeneratedReports({ ...filters });
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar relatórios gerados.";
  } finally {
    loading.value = false;
  }
}

function base64ToBlob(base64: string, mimeType: string): Blob {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return new Blob([bytes], { type: mimeType || "application/octet-stream" });
}

async function downloadRow(id: number) {
  try {
    const payload = await downloadGeneratedReport(id);
    const blob = base64ToBlob(payload.content_base64, payload.mime_type);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = payload.file_name || `relatorio_${id}`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 600);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao baixar relatório.";
  }
}

async function loadCombos() {
  if (!session.sessionToken) await session.restore();
  if (session.sessionToken) {
    users.value = await listUsers(session.sessionToken, { onlyActive: true });
  }
  const funcionariosCombo = await comboList("funcionarios");
  funcionarios.value = funcionariosCombo.map((f) => ({ id: Number(f.id), label: String(f.label || f.id) }));
}

onMounted(async () => {
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Relatórios gerados</h2>
        <div class="muted-text">Central única para consultar e baixar novamente arquivos gerados pelo sistema.</div>
      </div>
      <div class="actions">
        <button class="secondary" :disabled="loading" @click="load">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
      </div>
    </div>

    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card">
      <div class="grid columns-6 mobile-columns-1">
        <div class="field"><label>Competência</label><input v-model="filters.competencia" type="text" placeholder="02/2026" /></div>
        <div class="field">
          <label>Colaborador</label>
          <select v-model="filters.funcionarioNome">
            <option value="">Todos</option>
            <option v-for="f in funcionarios" :key="f.id" :value="f.label">{{ f.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Tipo</label>
          <select v-model="filters.tipoRelatorio">
            <option value="">Todos</option>
            <option v-for="t in tipos" :key="t" :value="t">{{ t }}</option>
          </select>
        </div>
        <div class="field">
          <label>Formato</label>
          <select v-model="filters.formato">
            <option value="">Todos</option>
            <option v-for="f in formatos" :key="f" :value="f">{{ f }}</option>
          </select>
        </div>
        <div class="field">
          <label>Usuário</label>
          <select v-model="filters.usuarioLogin">
            <option value="">Todos</option>
            <option v-for="u in users" :key="String(u.id)" :value="String(u.login || '')">{{ u.nome }} ({{ u.login }})</option>
          </select>
        </div>
        <div class="actions align-end"><button class="primary" :disabled="loading" @click="load">Aplicar filtros</button></div>
      </div>
    </div>

    <div class="card table-wrap">
      <table>
        <thead>
          <tr>
            <th>Descrição</th>
            <th>Tipo</th>
            <th>Origem</th>
            <th>Detalhado</th>
            <th>Gerado por</th>
            <th>Competência</th>
            <th>Colaborador</th>
            <th>Emissão</th>
            <th>Formato</th>
            <th>Status</th>
            <th>Ação</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="String(row.id)">
            <td>{{ row.descricao }}</td>
            <td>{{ row.tipo_relatorio }}</td>
            <td>{{ row.origem_rotina }}</td>
            <td>{{ row.detalhado ? 'Sim' : 'Não' }}</td>
            <td>{{ row.usuario_login || '-' }}</td>
            <td>{{ row.competencia || '-' }}</td>
            <td>{{ row.funcionario_nome || '-' }}</td>
            <td>{{ row.created_at }}</td>
            <td>{{ row.formato }}</td>
            <td>{{ row.status }}</td>
            <td><button class="secondary" @click="downloadRow(Number(row.id))">Download</button></td>
          </tr>
          <tr v-if="rows.length === 0"><td colspan="11" class="muted-text">Nenhum relatório encontrado.</td></tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
