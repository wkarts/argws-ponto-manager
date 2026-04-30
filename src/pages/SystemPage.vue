<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { RouterLink } from "vue-router";
import { getSystemInfo, setSystemDataDir } from "../services/crud";

const info = ref<Record<string, unknown>>({});
const form = reactive({ dataDir: "" });
const loading = ref(false);
const message = ref("");
const error = ref("");

async function load() {
  loading.value = true;
  error.value = "";
  try {
    info.value = await getSystemInfo();
    form.dataDir = String(info.value.data_dir || "");
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar informações do sistema.";
  } finally {
    loading.value = false;
  }
}

async function persist() {
  message.value = "";
  error.value = "";
  try {
    info.value = await setSystemDataDir(form.dataDir);
    message.value = "Diretório de parâmetros/dados atualizado. Reinicie a aplicação se necessário.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao atualizar diretório de dados.";
  }
}

onMounted(load);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Sistema e parâmetros</h2>
        <div class="muted-text">Informações técnicas, diretório de dados e parâmetros operacionais foram concentrados aqui para manter o dashboard limpo.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Diretórios</div>
        <div class="info-grid">
          <div class="info-item">
            <strong>Banco local</strong>
            <code>{{ info.db_path || '-' }}</code>
          </div>
          <div class="info-item">
            <strong>Diretório de dados</strong>
            <code>{{ info.data_dir || '-' }}</code>
          </div>
          <div class="info-item">
            <strong>Exportações</strong>
            <code>{{ info.exports_dir || '-' }}</code>
          </div>
          <div class="info-item">
            <strong>Bootstrap config</strong>
            <code>{{ info.bootstrap_config || '-' }}</code>
          </div>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Aplicação</div>
        <div class="info-grid">
          <div class="info-item">
            <strong>Produto</strong>
            <code>{{ info.product_name || 'Ponto Manager' }}</code>
          </div>
          <div class="info-item">
            <strong>Versão</strong>
            <code>{{ info.version || '-' }}</code>
          </div>
          <div class="info-item">
            <strong>Build</strong>
            <code>{{ info.build_hash || '-' }}</code>
          </div>
        </div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Integrações</div>
      <div class="muted-text">Os parâmetros da API do Ponto Manager Conector ficam em uma tela própria, evitando repetir URL/token em cada REP.</div>
      <div class="actions">
        <RouterLink class="button secondary" to="/conector-config">Abrir parâmetros do Ponto Conector</RouterLink>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Personalização do local dos parâmetros</div>
      <div class="muted-text">O diretório abaixo controla a localização do banco local e das exportações. A alteração é segura e copia o banco atual para o novo local.</div>
      <div class="grid columns-3 mobile-columns-1">
        <div class="field" style="grid-column: span 2;">
          <label>Diretório de dados/parâmetros</label>
          <input v-model="form.dataDir" type="text" placeholder="C:\\ARGWS\\PontoManager\\dados" />
        </div>
        <div class="actions align-end">
          <button class="primary" :disabled="loading" @click="persist">Salvar diretório</button>
          <button class="secondary" :disabled="loading" @click="load">Recarregar</button>
        </div>
      </div>
      <div class="muted-text">Observação: o diretório de instalação do executável é definido pelo instalador. O diretório acima controla os parâmetros e dados locais da aplicação.</div>
    </div>
  </div>
</template>
