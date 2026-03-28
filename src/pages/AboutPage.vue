<script setup lang="ts">
import { onMounted, reactive } from "vue";
import { getAppMeta } from "../services/crud";
import { logAppError } from "../services/logger";

const appInfo = reactive({
  appName: "Ponto Manager",
  version: "-",
  buildHash: "-",
  buildNumber: "-",
});

const support = {
  site: "https://wwsoftwares.com.br",
  email: "suporte@wwsoftwares.com.br",
  phone: "(75) 98333-4153",
};

onMounted(async () => {
  try {
    const meta = await getAppMeta();
    appInfo.appName = String(meta.product_name || appInfo.appName);
    appInfo.version = String(meta.version || "-");
    appInfo.buildHash = String(meta.build_hash || "-");
    appInfo.buildNumber = String(meta.build_number || meta.build || "-");
  } catch (err) {
    logAppError("about", "Falha ao carregar metadados da aplicação na tela Sobre.", {
      error: err instanceof Error ? err.message : String(err),
    });
  }
});
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Sobre e suporte</h2>
        <div class="muted-text">Área institucional e técnica da aplicação.</div>
      </div>
    </div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Aplicação</div>
        <ul class="summary-list">
          <li><strong>Nome:</strong> {{ appInfo.appName }}</li>
          <li><strong>Versão:</strong> {{ appInfo.version }}</li>
          <li><strong>Hash da build:</strong> {{ appInfo.buildHash }}</li>
          <li><strong>Número da build:</strong> {{ appInfo.buildNumber }}</li>
        </ul>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Suporte</div>
        <ul class="summary-list">
          <li><strong>Site:</strong> <a :href="support.site" target="_blank" rel="noopener noreferrer">{{ support.site }}</a></li>
          <li><strong>E-mail:</strong> <a :href="`mailto:${support.email}`">{{ support.email }}</a></li>
          <li><strong>WhatsApp / Telefone:</strong> {{ support.phone }}</li>
        </ul>
      </div>
    </div>
  </div>
</template>
