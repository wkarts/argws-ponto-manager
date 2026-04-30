<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { comboList, exportRepEmpresaTxt, exportRepFuncionariosTxt, type ComboOption } from "../services/crud";
import { carregarConfiguracaoConector, salvarConfiguracaoConector, testarConector } from "../services/conectorService";
import { useSessionStore } from "../stores/session";
import { showSplashError, showSplashSuccess } from "../services/splash";

const session = useSessionStore();
const companies = ref<ComboOption[]>([]);
const brand = ref("henry");
const empresaId = ref<number | null>(null);
const message = ref("");
const error = ref("");
const savingConfig = ref(false);
const testingConfig = ref(false);
const conectorBaseUrl = ref("");
const conectorApiToken = ref("");
const conectorTokenConfigurado = ref(false);
const conectorTimeout = ref<number>(30);

function download(fileName: string, content: string) {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

async function loadCompanies() {
  const all = await comboList("empresas");
  companies.value = session.isMaster ? all : all.filter((item) => session.user?.company_ids.includes(item.id));
  empresaId.value = session.activeCompanyId ?? companies.value[0]?.id ?? null;
}

async function loadConectorConfig() {
  try {
    const cfg = await carregarConfiguracaoConector();
    conectorBaseUrl.value = String(cfg.base_url || "");
    conectorTokenConfigurado.value = Boolean(cfg.api_token_configurado);
    conectorTimeout.value = Number(cfg.timeout_secs || 30);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar configuração do conector.";
  }
}

async function salvarConfig() {
  savingConfig.value = true;
  error.value = "";
  try {
    const cfg = await salvarConfiguracaoConector({
      base_url: conectorBaseUrl.value,
      api_token: conectorApiToken.value || null,
      timeout_secs: conectorTimeout.value || 30,
    });
    conectorTokenConfigurado.value = Boolean(cfg.api_token_configurado);
    conectorApiToken.value = "";
    message.value = "Configuração da API do Ponto Manager Conector salva com sucesso.";
    showSplashSuccess(message.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar configuração do conector.";
    showSplashError(error.value);
  } finally {
    savingConfig.value = false;
  }
}

async function testar() {
  testingConfig.value = true;
  error.value = "";
  try {
    const res = await testarConector();
    message.value = `Conector respondeu com status ${res}.`;
    showSplashSuccess(message.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao testar o conector.";
    showSplashError(error.value);
  } finally {
    testingConfig.value = false;
  }
}

async function exportEmpresa() {
  if (!empresaId.value) return;
  error.value = "";
  try {
    const result = await exportRepEmpresaTxt(brand.value, empresaId.value);
    download(String(result.file_name), String(result.content || ""));
    message.value = `Cadastro da empresa exportado para ${brand.value.toUpperCase()}.`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar empresa para REP.";
  }
}

async function exportFuncionarios() {
  if (!empresaId.value) return;
  error.value = "";
  try {
    const result = await exportRepFuncionariosTxt(brand.value, empresaId.value);
    download(String(result.file_name), String(result.content || ""));
    message.value = `Funcionários exportados para ${brand.value.toUpperCase()} (${result.total} registro(s)).`;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao exportar funcionários para REP.";
  }
}

watch(() => session.activeCompanyId, loadCompanies);
onMounted(async () => {
  await Promise.all([loadCompanies(), loadConectorConfig()]);
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Exportação para REP</h2>
        <div class="muted-text">Exportação TXT e configuração central da API do Ponto Manager Conector.</div>
      </div>
    </div>
    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div>
        <h3>API do Ponto Manager Conector</h3>
        <div class="muted-text">A URL e o token ficam centralizados aqui. No cadastro do equipamento/REP marque apenas se ele usa o conector e informe o ID do dispositivo no conector.</div>
      </div>
      <div class="grid columns-3 mobile-columns-1">
        <div class="field">
          <label>URL base da API</label>
          <input v-model="conectorBaseUrl" type="text" placeholder="http://127.0.0.1:3000" />
        </div>
        <div class="field">
          <label>Token da API</label>
          <input v-model="conectorApiToken" type="password" :placeholder="conectorTokenConfigurado ? 'Token já configurado; informe apenas para trocar' : 'Informe o token da API'" />
        </div>
        <div class="field">
          <label>Timeout (segundos)</label>
          <input v-model.number="conectorTimeout" type="number" min="5" />
        </div>
      </div>
      <div class="actions">
        <button class="primary" :disabled="savingConfig" @click="salvarConfig">{{ savingConfig ? 'Salvando...' : 'Salvar configuração da API' }}</button>
        <button class="secondary" :disabled="testingConfig" @click="testar">{{ testingConfig ? 'Testando...' : 'Testar API do conector' }}</button>
      </div>
    </div>

    <div class="card grid page-gap">
      <div>
        <h3>Arquivos TXT para REP</h3>
        <div class="muted-text">Estrutura extensível para Henry, Evo, Blue, Dimep e novas marcas, com exportação separada de empresa e funcionários.</div>
      </div>
      <div class="grid columns-3 mobile-columns-1">
        <div class="field">
          <label>Marca / layout</label>
          <select v-model="brand">
            <option value="henry">Henry</option>
            <option value="evo">Evo</option>
            <option value="blue">Blue</option>
            <option value="dimep">Dimep</option>
            <option value="generic">Genérico</option>
          </select>
        </div>
        <div class="field">
          <label>Empresa</label>
          <select v-model="empresaId">
            <option v-for="item in companies" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>
        <div class="actions align-end">
          <button class="primary" @click="exportEmpresa">Exportar empresa</button>
          <button class="secondary" @click="exportFuncionarios">Exportar funcionários</button>
        </div>
      </div>
    </div>
  </div>
</template>
