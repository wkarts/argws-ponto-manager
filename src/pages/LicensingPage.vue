<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  checkLicensingRuntime,
  getLicensingDeviceInfo,
  getLicensingStatus,
  loadLicensingSettings,
  saveLicensingSettings,
  startTrialLicense,
} from "../services/crud";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const loading = ref(false);
const saving = ref(false);
const checking = ref(false);
const trialLoading = ref(false);
const message = ref("");
const error = ref("");
const status = ref<Record<string, unknown> | null>(null);
const runtime = ref<Record<string, unknown> | null>(null);
const deviceInfo = ref<Record<string, unknown> | null>(null);
const settings = ref<Record<string, unknown>>({});

const empresaId = computed(() => session.activeCompanyId ?? session.user?.company_ids?.[0] ?? null);
const companyName = computed(() => String(status.value?.empresa_nome || runtime.value?.empresa_nome || "-"));
const companyDocument = computed(() => String(status.value?.cnpj || runtime.value?.cnpj || "-"));
const localLicense = computed(() => ((status.value?.license as Record<string, unknown> | null) || (runtime.value?.local_license as Record<string, unknown> | null) || null));
const onlineDecision = computed(() => (runtime.value?.decision as Record<string, unknown> | null) || null);
const disabledMode = computed({
  get: () => Boolean(settings.value.licensing_disabled),
  set: (value: boolean) => { settings.value.licensing_disabled = value; }
});

function stringValue(key: string) {
  return String(settings.value[key] ?? "");
}

function setString(key: string, value: string) {
  settings.value[key] = value;
}

function setBool(key: string, value: boolean) {
  settings.value[key] = value;
}

async function loadPage() {
  loading.value = true;
  error.value = "";
  try {
    const [settingsData, statusData, deviceData] = await Promise.all([
      loadLicensingSettings(),
      getLicensingStatus(empresaId.value),
      getLicensingDeviceInfo(),
    ]);
    settings.value = { ...settingsData };
    if (!settings.value.company_name && statusData.empresa_nome) settings.value.company_name = statusData.empresa_nome;
    if (!settings.value.company_document && statusData.cnpj) settings.value.company_document = statusData.cnpj;
    if (!settings.value.company_email) settings.value.company_email = "";
    status.value = statusData;
    deviceInfo.value = deviceData;
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar o licenciamento.";
  } finally {
    loading.value = false;
  }
}

async function saveSettings() {
  saving.value = true;
  message.value = "";
  error.value = "";
  try {
    settings.value = await saveLicensingSettings(settings.value);
    message.value = "Configurações de licenciamento salvas com sucesso.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao salvar configurações de licenciamento.";
  } finally {
    saving.value = false;
  }
}

async function runCheck() {
  checking.value = true;
  message.value = "";
  error.value = "";
  try {
    runtime.value = await checkLicensingRuntime(empresaId.value);
    message.value = "Validação de licenciamento concluída.";
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao validar licenciamento.";
  } finally {
    checking.value = false;
  }
}

async function activateTrial() {
  trialLoading.value = true;
  message.value = "";
  error.value = "";
  try {
    await startTrialLicense(empresaId.value);
    message.value = "Licença de teste de 45 dias liberada com sucesso.";
    await loadPage();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao liberar licença de teste.";
  } finally {
    trialLoading.value = false;
  }
}

watch(empresaId, async () => {
  runtime.value = null;
  await loadPage();
});

onMounted(loadPage);
</script>

<template>
  <div class="grid page-gap licensing-page">
    <div class="toolbar licensing-header">
      <div>
        <h2>Licenciamento</h2>
        <div class="muted-text">
          Módulo dedicado de licenciamento usando a mesma tecnologia genérica do Integra Desktop,
          sem modal bloqueante e com suporte a licença de teste local de 45 dias vinculada ao CNPJ.
        </div>
      </div>
      <div class="actions wrap">
        <button class="secondary" :disabled="loading || checking" @click="loadPage">Atualizar</button>
        <button class="primary" :disabled="saving || checking" @click="saveSettings">Salvar configurações</button>
        <button class="primary" :disabled="checking || loading" @click="runCheck">Validar licença</button>
      </div>
    </div>

    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="grid columns-2 mobile-columns-1 licensing-grid">
      <div class="card grid page-gap">
        <div class="section-title">Empresa e contexto</div>
        <div class="info-grid">
          <div class="info-item"><strong>Empresa ativa</strong><code>{{ companyName }}</code></div>
          <div class="info-item"><strong>CNPJ</strong><code>{{ companyDocument }}</code></div>
          <div class="info-item"><strong>Dispositivo</strong><code>{{ deviceInfo?.station_name || status?.machine_key || '-' }}</code></div>
          <div class="info-item"><strong>Fingerprint</strong><code>{{ deviceInfo?.device_key || status?.machine_key || '-' }}</code></div>
        </div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Licença local / teste</div>
        <div v-if="localLicense" class="info-grid">
          <div class="info-item"><strong>Tipo</strong><code>{{ localLicense.license_kind }}</code></div>
          <div class="info-item"><strong>Status</strong><code>{{ localLicense.status }}</code></div>
          <div class="info-item"><strong>Emissão</strong><code>{{ localLicense.issued_at }}</code></div>
          <div class="info-item"><strong>Validade</strong><code>{{ localLicense.expires_at }}</code></div>
        </div>
        <div v-else class="muted-text">Nenhuma licença local gravada para a empresa ativa.</div>
        <div class="actions wrap">
          <button class="primary" :disabled="trialLoading || !!localLicense" @click="activateTrial">Liberar teste 45 dias</button>
        </div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Configuração do componente genérico</div>
      <div class="form-grid two-columns licensing-form-grid">
        <label>
          <span>Service URL</span>
          <input :value="stringValue('service_url')" @input="setString('service_url', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>Instância / app slug</span>
          <input :value="stringValue('app_instance')" @input="setString('app_instance', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>Razão social</span>
          <input :value="stringValue('company_name')" @input="setString('company_name', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>CNPJ</span>
          <input :value="stringValue('company_document')" @input="setString('company_document', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>E-mail da empresa</span>
          <input :value="stringValue('company_email')" @input="setString('company_email', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>Estação</span>
          <input :value="stringValue('station_name')" @input="setString('station_name', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>Validation mode</span>
          <input :value="stringValue('auto_register_validation_mode')" @input="setString('auto_register_validation_mode', ($event.target as HTMLInputElement).value)" />
        </label>
        <label>
          <span>Interface mode</span>
          <input :value="stringValue('auto_register_interface_mode')" @input="setString('auto_register_interface_mode', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>
      <div class="checkbox-grid licensing-checkboxes">
        <label class="checkbox-inline">
          <input type="checkbox" :checked="Boolean(settings.auto_register_machine)" @change="setBool('auto_register_machine', ($event.target as HTMLInputElement).checked)" />
          <span>Permitir auto cadastro de empresa/dispositivo</span>
        </label>
        <label class="checkbox-inline">
          <input type="checkbox" v-model="disabledMode" />
          <span>Desabilitar licenciamento na aplicação</span>
        </label>
      </div>
    </div>

    <div class="grid columns-2 mobile-columns-1 licensing-grid">
      <div class="card grid page-gap">
        <div class="section-title">Resultado da validação</div>
        <div v-if="onlineDecision" class="info-grid">
          <div class="info-item"><strong>Permitido</strong><code>{{ onlineDecision.allowed ? 'Sim' : 'Não' }}</code></div>
          <div class="info-item"><strong>Decisão</strong><code>{{ onlineDecision.decision || '-' }}</code></div>
          <div class="info-item"><strong>Motivo</strong><code>{{ onlineDecision.reason_code || '-' }}</code></div>
          <div class="info-item"><strong>Etapa</strong><code>{{ onlineDecision.step || '-' }}</code></div>
          <div class="info-item info-item-full"><strong>Mensagem</strong><code>{{ onlineDecision.message || '-' }}</code></div>
        </div>
        <div v-else class="muted-text">Execute a validação para consultar o status remoto/offline através do componente genérico.</div>
      </div>

      <div class="card grid page-gap">
        <div class="section-title">Informações do dispositivo</div>
        <div class="info-grid">
          <div class="info-item"><strong>Hostname</strong><code>{{ deviceInfo?.hostname || '-' }}</code></div>
          <div class="info-item"><strong>Usuário</strong><code>{{ deviceInfo?.logged_user || '-' }}</code></div>
          <div class="info-item"><strong>Sistema</strong><code>{{ deviceInfo?.os_name || '-' }}</code></div>
          <div class="info-item"><strong>Arquitetura</strong><code>{{ deviceInfo?.os_arch || '-' }}</code></div>
          <div class="info-item"><strong>Modo de instalação</strong><code>{{ deviceInfo?.install_mode || '-' }}</code></div>
          <div class="info-item info-item-full"><strong>MACs</strong><code>{{ Array.isArray(deviceInfo?.mac_addresses) ? (deviceInfo?.mac_addresses as unknown[]).join(', ') : '-' }}</code></div>
        </div>
      </div>
    </div>

    <div class="card grid page-gap">
      <div class="section-title">Regras aplicadas</div>
      <ul class="summary-list">
        <li>Nenhum modal de licenciamento bloqueia o uso da aplicação.</li>
        <li>O licenciamento fica centralizado em uma página/seção própria no menu.</li>
        <li>A licença de teste é vinculada ao CNPJ da empresa ativa e gravada de forma criptografada no banco local.</li>
        <li>A validação online/offline usa o mesmo componente genérico <code>generic-license-tauri</code> aplicado no Integra Desktop.</li>
      </ul>
    </div>
  </div>
</template>
