<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { getBootstrap } from "../services/crud";
import { appFeatures, dashboardConfig, projectConfig } from "../config/projectConfig";
import DashboardMetricCard from "../components/dashboard/DashboardMetricCard.vue";
import DashboardQuickActions from "../components/dashboard/DashboardQuickActions.vue";
import DashboardHealthPanel from "../components/dashboard/DashboardHealthPanel.vue";
import DashboardRecentEvents from "../components/dashboard/DashboardRecentEvents.vue";
import DashboardChartCard from "../components/dashboard/DashboardChartCard.vue";
import AppPageTitleBar from "../components/base/AppPageTitleBar.vue";
import IconSymbol from "../components/base/IconSymbol.vue";

const loading = ref(false);
const stats = ref<Record<string, unknown>>({});
const error = ref("");
const dashboardCustomizerOpen = ref(false);
const dashboardWidgets = ref({
  metrics: true,
  operational: true,
  health: true,
  quickActions: true,
  recentEvents: true,
  charts: true,
});

function resetDashboardWidgets() {
  dashboardWidgets.value = {
    metrics: true,
    operational: true,
    health: true,
    quickActions: true,
    recentEvents: true,
    charts: true,
  };
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    stats.value = await getBootstrap();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao carregar dashboard.";
  } finally {
    loading.value = false;
  }
}

const quickActions = computed(() => [
  { title: "Cartão de ponto", route: "/cartao-ponto", icon: "timeCard" },
  { title: "Importar AFD", route: "/afd", icon: "fileImport" },
  { title: "Ponto Conector", route: "/conector-dashboard", icon: "connector" },
  { title: "Tratamento", route: "/tratamentos", icon: "clipboardCheck" },
  { title: "Funcionários", route: "/funcionarios", icon: "idBadge" },
  { title: "Relatórios", route: "/relatorios", icon: "reports" },
]);

function formatDateTime(value: unknown): string {
  const raw = String(value || "");
  if (!raw) return "Nenhum registro";
  const parsed = new Date(raw);
  return Number.isNaN(parsed.getTime())
    ? raw
    : new Intl.DateTimeFormat("pt-BR", { dateStyle: "short", timeStyle: "short" }).format(parsed);
}

const recentEvents = computed(() => [
  { title: "Última importação AFD", subtitle: formatDateTime(stats.value.ultima_importacao_afd), tone: "info" },
  { title: "Última coleta pelo Connector", subtitle: formatDateTime(stats.value.ultima_coleta_conector), tone: "success" },
  { title: "Fila de sincronização", subtitle: `${Number(stats.value.sync_pendente || 0)} item(ns) pendente(s)`, tone: Number(stats.value.sync_pendente || 0) > 0 ? "warning" : "success" },
  { title: "Base operacional", subtitle: `${projectConfig.database.driver.toUpperCase()} · ${String(stats.value.database_status || "ok")}`, tone: "info" },
]);

interface DailyPunchMetric {
  data: string;
  total: number;
  label: string;
  height: string;
}

const punchesByDay = computed<DailyPunchMetric[]>(() => {
  const source = Array.isArray(stats.value.batidas_por_dia)
    ? stats.value.batidas_por_dia as Array<Record<string, unknown>>
    : [];
  const totals = new Map(source.map((row) => [String(row.data || ""), Number(row.total || 0)]));
  const days = Array.from({ length: 7 }, (_, offset) => {
    const date = new Date();
    date.setHours(12, 0, 0, 0);
    date.setDate(date.getDate() - (6 - offset));
    const key = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
    return { data: key, total: totals.get(key) || 0, label: new Intl.DateTimeFormat("pt-BR", { day: "2-digit", month: "2-digit" }).format(date) };
  });
  const max = Math.max(1, ...days.map((item) => item.total));
  return days.map((item) => ({ ...item, height: `${Math.max(8, Math.round((item.total / max) * 100))}%` }));
});

const operationalGroups = computed(() => [
  {
    title: "Funcionários",
    icon: "idBadge",
    route: "/funcionarios",
    items: [
      { label: "Ativos", value: Number(stats.value.funcionarios_ativos || 0) },
      { label: "Em férias hoje", value: Number(stats.value.funcionarios_ferias_hoje || 0) },
      { label: "Inativos", value: Number(stats.value.funcionarios_inativos || 0) },
    ],
  },
  {
    title: "Coleta AFD e Connector",
    icon: "connector",
    route: "/conector-dashboard",
    items: [
      { label: "Coletas hoje", value: Number(stats.value.conector_coletas_hoje || 0) },
      { label: "Importadas hoje", value: Number(stats.value.conector_importadas_hoje || 0) + Number(stats.value.afd_processadas_hoje || 0) },
      { label: "Duplicadas bloqueadas", value: Number(stats.value.conector_duplicadas_hoje || 0) + Number(stats.value.afd_descartadas_hoje || 0) },
    ],
  },
  {
    title: "Ponto e conferência",
    icon: "timeCard",
    route: "/cartao-ponto",
    items: [
      { label: "Batidas hoje", value: Number(stats.value.batidas_hoje || 0) },
      { label: "Inconsistências hoje", value: Number(stats.value.inconsistencias_hoje || 0) },
      { label: "Duplicidades ocultas", value: Number(stats.value.batidas_duplicadas_ocultas || 0) },
    ],
  },
]);

onMounted(load);
</script>

<template>
  <div class="page-content-scroll dashboard-page">
    <AppPageTitleBar
      title="Dashboard"
      subtitle="Resumo operacional de funcionários, batidas, coletas, integridade AFD e relatórios."
      eyebrow="Ponto Manager"
      icon="chart"
    >
      <template #actions>
        <button class="secondary" type="button" @click="dashboardCustomizerOpen = !dashboardCustomizerOpen">Personalizar dashboard</button>
        <button class="primary" :disabled="loading" @click="load">{{ loading ? "Atualizando..." : "Atualizar" }}</button>
      </template>
    </AppPageTitleBar>

    <section v-if="dashboardCustomizerOpen" class="dashboard-customizer-panel">
      <div>
        <p class="eyebrow">Personalização da dashboard</p>
        <h3>Widgets visíveis</h3>
        <p>Preferência visual local para organizar a tela inicial sem alterar regras de negócio.</p>
      </div>
      <div class="dashboard-customizer-options">
        <label><input v-model="dashboardWidgets.metrics" type="checkbox" /> Indicadores</label>
        <label><input v-model="dashboardWidgets.operational" type="checkbox" /> Resumo operacional</label>
        <label><input v-model="dashboardWidgets.health" type="checkbox" /> Saúde do sistema</label>
        <label><input v-model="dashboardWidgets.quickActions" type="checkbox" /> Atalhos rápidos</label>
        <label><input v-model="dashboardWidgets.recentEvents" type="checkbox" /> Últimos eventos</label>
        <label><input v-model="dashboardWidgets.charts" type="checkbox" /> Gráficos</label>
      </div>
      <div class="dashboard-customizer-actions">
        <button class="secondary" type="button" @click="resetDashboardWidgets">Restaurar padrão</button>
        <button class="primary" type="button" @click="dashboardCustomizerOpen = false">Aplicar</button>
      </div>
    </section>

    <div v-if="error" class="alert error">{{ error }}</div>
    <div v-if="loading" class="muted-text">Carregando indicadores reais...</div>

    <section v-if="dashboardWidgets.metrics" class="dashboard-metrics-grid">
      <DashboardMetricCard title="Funcionários ativos" :value="Number(stats.funcionarios_ativos || 0)" subtitle="Quadro atual" icon="idBadge" status="info" />
      <DashboardMetricCard title="Batidas hoje" :value="Number(stats.batidas_hoje || 0)" subtitle="Marcações válidas" icon="fingerprint" status="success" />
      <DashboardMetricCard title="Inconsistências" :value="Number(stats.inconsistencias_hoje || 0)" subtitle="Funcionários com marcação ímpar hoje" icon="alert" :status="Number(stats.inconsistencias_hoje || 0) > 0 ? 'warning' : 'success'" />
      <DashboardMetricCard title="AFD processadas" :value="Number(stats.afd_processadas_hoje || 0)" :subtitle="`${Number(stats.afd_importacoes_hoje || 0)} arquivo(s) hoje`" icon="fileImport" status="info" />
      <DashboardMetricCard title="Duplicidades bloqueadas" :value="Number(stats.afd_descartadas_hoje || 0) + Number(stats.conector_duplicadas_hoje || 0)" subtitle="AFD e Connector hoje" icon="layersCheck" status="warning" />
      <DashboardMetricCard title="Ajustes pendentes" :value="Number(stats.batidas_pendentes_validacao || 0)" subtitle="Marcações manuais sem validação" icon="clipboardCheck" :status="Number(stats.batidas_pendentes_validacao || 0) > 0 ? 'warning' : 'success'" />
    </section>

    <section v-if="dashboardWidgets.operational" class="dashboard-operational-grid dashboard-card-grid">
      <RouterLink v-for="group in operationalGroups" :key="group.title" :to="group.route" class="card dashboard-operation-card">
        <header>
          <span class="dashboard-operation-icon"><IconSymbol :name="group.icon" :size="18" /></span>
          <strong>{{ group.title }}</strong>
          <span aria-hidden="true">›</span>
        </header>
        <div>
          <span v-for="item in group.items" :key="item.label"><small>{{ item.label }}</small><strong>{{ item.value }}</strong></span>
        </div>
      </RouterLink>
    </section>

    <section class="dashboard-grid-main dashboard-card-grid">
      <DashboardHealthPanel
        v-if="dashboardWidgets.health"
        :items="[
          { label: 'Banco de dados', value: `${projectConfig.database.driver.toUpperCase()} / ${String(stats.database_status || 'ok')}`, status: 'info' },
          { label: 'Modo de execução', value: projectConfig.app.mode, status: 'info' },
          { label: 'API interna', value: appFeatures.internalApi ? String(stats.internal_api_status || 'ativável') : 'desativada neste projeto', status: appFeatures.internalApi ? 'success' : 'neutral' },
          { label: 'Licenciamento', value: appFeatures.licensing ? 'ativo no template' : 'não utilizado', status: appFeatures.licensing ? 'info' : 'neutral' }
        ]"
      />
      <DashboardQuickActions v-if="dashboardWidgets.quickActions" :actions="quickActions" />
      <DashboardRecentEvents v-if="dashboardWidgets.recentEvents" :rows="recentEvents" />
    </section>

    <section v-if="dashboardConfig.showCharts && dashboardWidgets.charts" class="dashboard-chart-grid">
      <DashboardChartCard title="Batidas por dia" subtitle="Últimos 7 dias · dados reais do cartão">
        <div class="demo-bars-chart operational-bars-chart" aria-label="Batidas por dia nos últimos sete dias">
          <span v-for="item in punchesByDay" :key="item.data" :style="{ '--h': item.height }" :title="`${item.label}: ${item.total} batida(s)`">
            <strong>{{ item.total }}</strong>
            <small>{{ item.label }}</small>
          </span>
        </div>
      </DashboardChartCard>
      <DashboardChartCard title="Coleta e integridade hoje" subtitle="AFD, Connector e sincronização">
        <div class="dashboard-integrity-summary">
          <RouterLink to="/afd"><span>Arquivos AFD</span><strong>{{ Number(stats.afd_importacoes_hoje || 0) }}</strong></RouterLink>
          <RouterLink to="/conector-dashboard"><span>Coletas Connector</span><strong>{{ Number(stats.conector_coletas_hoje || 0) }}</strong></RouterLink>
          <RouterLink to="/cartao-ponto"><span>Duplicidades ocultas</span><strong>{{ Number(stats.batidas_duplicadas_ocultas || 0) }}</strong></RouterLink>
          <RouterLink to="/sync-queue"><span>Sincronizações pendentes</span><strong>{{ Number(stats.sync_pendente || 0) }}</strong></RouterLink>
        </div>
      </DashboardChartCard>
    </section>
  </div>
</template>
