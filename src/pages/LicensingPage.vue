<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { getLicensingStatus, startTrialLicense } from "../services/crud";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const status = ref<Record<string, unknown> | null>(null);
const loading = ref(false);
const message = ref("");
const error = ref("");

const empresaId = computed(() => session.activeCompanyId ?? session.user?.company_ids?.[0] ?? null);
const license = computed(() => (status.value?.license as Record<string, unknown> | null) || null);

async function load() {
  loading.value = true;
  error.value = "";
  try {
    status.value = await getLicensingStatus(empresaId.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao consultar licenciamento.";
  } finally {
    loading.value = false;
  }
}

async function startTrial() {
  message.value = "";
  error.value = "";
  try {
    await startTrialLicense(empresaId.value);
    message.value = "Licença de teste de 45 dias liberada com sucesso.";
    await load();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao iniciar licença de teste.";
  }
}

watch(empresaId, load);
onMounted(load);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Licenciamento</h2>
        <div class="muted-text">Página própria de licenciamento, sem modal bloqueante. A aplicação continua utilizável conforme a política adotada, com suporte a licença de teste de 45 dias vinculada ao CNPJ da empresa.</div>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-2 mobile-columns-1">
      <div class="card grid page-gap">
        <div class="section-title">Empresa vinculada</div>
        <div class="info-grid">
          <div class="info-item"><strong>Empresa</strong><code>{{ status?.empresa_nome || '-' }}</code></div>
          <div class="info-item"><strong>CNPJ</strong><code>{{ status?.cnpj || '-' }}</code></div>
          <div class="info-item"><strong>Fingerprint</strong><code>{{ status?.machine_key || '-' }}</code></div>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Status da licença</div>
        <div v-if="license" class="info-grid">
          <div class="info-item"><strong>Tipo</strong><code>{{ license.license_kind }}</code></div>
          <div class="info-item"><strong>Status</strong><code>{{ license.status }}</code></div>
          <div class="info-item"><strong>Emissão</strong><code>{{ license.issued_at }}</code></div>
          <div class="info-item"><strong>Validade</strong><code>{{ license.expires_at }}</code></div>
        </div>
        <div v-else class="muted-text">Nenhuma licença local encontrada para a empresa ativa.</div>
        <div class="actions">
          <button class="primary" :disabled="loading || !!license" @click="startTrial">Liberar teste 45 dias</button>
          <button class="secondary" :disabled="loading" @click="load">Atualizar</button>
        </div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Observações</div>
      <ul class="summary-list">
        <li>A licença de teste é vinculada ao CNPJ da empresa e ao dispositivo atual.</li>
        <li>Os dados da licença são gravados de forma criptografada no banco local.</li>
        <li>Esta tela substitui qualquer modal invasivo e centraliza a manutenção do licenciamento.</li>
      </ul>
    </div>
  </div>
</template>
