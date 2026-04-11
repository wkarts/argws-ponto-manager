<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { comboList, importAfdFile, listAfdImports, type ComboOption, type GenericRecord } from "../services/crud";
import { useSessionStore } from "../stores/session";
import { showSplashError } from "../services/splash";

const session = useSessionStore();
const rows = ref<GenericRecord[]>([]);
const companyOptions = ref<ComboOption[]>([]);
const equipmentOptions = ref<ComboOption[]>([]);
const selectedFile = ref<File | null>(null);
const empresaId = ref<string>("");
const equipamentoId = ref<string>("");
const mode = ref<string>("auto");
const loading = ref(false);
const importing = ref(false);
const error = ref("");
const message = ref("");

function handleFile(event: Event) {
  const input = event.target as HTMLInputElement;
  selectedFile.value = input.files?.[0] || null;
}

async function fileToText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(new Error("Falha ao ler o arquivo selecionado."));
    reader.readAsText(file, "latin1");
  });
}

async function loadCombos() {
  const [empresas, equipamentos] = await Promise.all([comboList("empresas"), comboList("equipamentos")]);
  companyOptions.value = empresas;
  equipmentOptions.value = equipamentos;
  empresaId.value = String(session.activeCompanyId || empresaId.value || "");
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    rows.value = await listAfdImports();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar histórico de AFD.";
  } finally {
    loading.value = false;
  }
}

async function processImport() {
  error.value = "";
  message.value = "";
  if (!selectedFile.value) {
    error.value = "Selecione um arquivo AFD para importar.";
    showSplashError(error.value);
    return;
  }

  importing.value = true;
  try {
    const content = await fileToText(selectedFile.value);
    const result = await importAfdFile({
      empresaId: empresaId.value ? Number(empresaId.value) : null,
      equipamentoId: equipamentoId.value ? Number(equipamentoId.value) : null,
      fileName: selectedFile.value.name,
      content,
      mode: mode.value
    });

    message.value = `Importação concluída. Layout ${result.layout_portaria}, processadas ${result.total_processadas}, descartadas ${result.total_descartadas}.`;
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao importar AFD.";
  } finally {
    importing.value = false;
  }
}

watch(() => session.activeCompanyId, (value) => { empresaId.value = String(value || ""); });

onMounted(async () => {
  empresaId.value = String(session.activeCompanyId || "");
  await loadCombos();
  await load();
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Importação e tratamento de AFD</h2>
        <div class="muted-text">Suporte ao AFD legado da Portaria 1.510/2009 e ao AFD da Portaria 671/2021.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="filter-grid">
        <div class="field">
          <label>Empresa</label>
          <select v-model="empresaId">
            <option value="">Selecione</option>
            <option v-for="item in companyOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Equipamento</label>
          <select v-model="equipamentoId">
            <option value="">Selecione</option>
            <option v-for="item in equipmentOptions" :key="item.id" :value="String(item.id)">{{ item.label }}</option>
          </select>
        </div>
        <div class="field">
          <label>Modo de leitura</label>
          <select v-model="mode">
            <option value="auto">Automático</option>
            <option value="1510">Portaria 1.510/2009</option>
            <option value="671">Portaria 671/2021</option>
          </select>
        </div>
        <div class="field">
          <label>Arquivo AFD</label>
          <input type="file" accept=".afd,.txt" @change="handleFile" />
        </div>
      </div>

      <div class="actions">
        <button class="primary" :disabled="importing" @click="processImport">
          {{ importing ? 'Importando...' : 'Importar arquivo AFD' }}
        </button>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="toolbar">
        <div>
          <h3>Histórico de importações</h3>
          <div class="muted-text">Registro bruto, layout detectado e resultado do tratamento local.</div>
        </div>
        <div class="actions">
          <button class="secondary" :disabled="loading" @click="load">{{ loading ? 'Atualizando...' : 'Atualizar' }}</button>
        </div>
      </div>

      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Arquivo</th>
              <th>Empresa</th>
              <th>Equipamento</th>
              <th>Layout</th>
              <th>Marcações</th>
              <th>Processadas</th>
              <th>Descartadas</th>
              <th>Data</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="Number(row.id)">
              <td>{{ row.id }}</td>
              <td>{{ row.nome_arquivo }}</td>
              <td>{{ row.empresa_nome || '-' }}</td>
              <td>{{ row.equipamento_nome || '-' }}</td>
              <td>{{ row.layout_portaria }}</td>
              <td>{{ row.total_marcacoes }}</td>
              <td>{{ row.total_processadas }}</td>
              <td>{{ row.total_descartadas }}</td>
              <td>{{ row.created_at }}</td>
            </tr>
            <tr v-if="!rows.length">
              <td colspan="9" class="empty-cell">Nenhuma importação AFD registrada.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
