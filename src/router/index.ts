import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";
import AppLayout from "../layouts/AppLayout.vue";
import DashboardPage from "../pages/DashboardPage.vue";
import EntityPage from "../pages/EntityPage.vue";
import EmpresaPage from "../pages/EmpresaPage.vue";
import FuncionarioPage from "../pages/FuncionarioPage.vue";
import JornadaPage from "../pages/JornadaPage.vue";
import AfdImportPage from "../pages/AfdImportPage.vue";
import BancoHorasPage from "../pages/BancoHorasPage.vue";
import TratamentosPage from "../pages/TratamentosPage.vue";
import FechamentoMensalPage from "../pages/FechamentoMensalPage.vue";
import UsuarioPage from "../pages/UsuarioPage.vue";
import PerfilPage from "../pages/PerfilPage.vue";
import FeriadoPage from "../pages/FeriadoPage.vue";
import LoginPage from "../pages/LoginPage.vue";
import FirstAccessPage from "../pages/FirstAccessPage.vue";
import PunchesPage from "../pages/PunchesPage.vue";
import ApuracaoPage from "../pages/ApuracaoPage.vue";
import SyncQueuePage from "../pages/SyncQueuePage.vue";
import SystemPage from "../pages/SystemPage.vue";
import LicensingPage from "../pages/LicensingPage.vue";
import AboutPage from "../pages/AboutPage.vue";
import AppLogsPage from "../pages/AppLogsPage.vue";
import UserGuidePage from "../pages/UserGuidePage.vue";
import ReportsCenterPage from "../pages/ReportsCenterPage.vue";
import RelatorioHorasPage from "../pages/RelatorioHorasPage.vue";
import GeneratedReportsPage from "../pages/GeneratedReportsPage.vue";
import RepExportPage from "../pages/RepExportPage.vue";
import ConectorDashboardPage from "../pages/ConectorDashboardPage.vue";
import PunchBatchPage from "../pages/PunchBatchPage.vue";
import CartaoPontoPage from "../pages/CartaoPontoPage.vue";
import TechnicalSheetPage from "../pages/optional/TechnicalSheetPage.vue";
import SyncPage from "../pages/optional/SyncPage.vue";
import InternalApiPage from "../pages/optional/InternalApiPage.vue";
import ScalarDocsPage from "../pages/optional/ScalarDocsPage.vue";
import WebhookServicePage from "../pages/optional/WebhookServicePage.vue";
import WebSocketServicePage from "../pages/optional/WebSocketServicePage.vue";
import DatabasePage from "../pages/optional/DatabasePage.vue";
import IntegrationPage from "../pages/optional/IntegrationPage.vue";
import PrintPreviewPage from "../pages/PrintPreviewPage.vue";
import RuntimeDiagnosticsPage from "../pages/RuntimeDiagnosticsPage.vue";
import { entityConfigs } from "../config/entities";
import { appFeatures } from "../config/projectConfig";
import { isFeatureEnabled } from "../config/navigation";
import { useSessionStore } from "../stores/session";
import { logAppError, logAppInfo, logAppWarning } from "../services/logger";

const permissionByEntity: Record<string, string> = {
  departamentos: "funcionarios:view",
  funcoes: "funcionarios:view",
  centro_custos: "funcionarios:view",
  horarios: "horarios:view",
  escalas: "escalas:view",
  feriados: "feriados:view",
  ferias_colaboradores: "ferias:view",
  jornada_contextos_regras: "jornadas:view",
  equipamentos: "equipamentos:view",
  eventos: "eventos:view",
  justificativas: "justificativas:view",
};

const genericEntityRoutes: RouteRecordRaw[] = Object.values(entityConfigs)
  .filter((entity) => !["empresas", "funcionarios", "usuarios", "feriados"].includes(entity.key))
  .map((entity) => ({
    path: entity.route.replace(/^\//, ""),
    component: EntityPage,
    props: { entityKey: entity.key },
    meta: { permission: permissionByEntity[entity.key], feature: "genericEntities" },
  }));

const routes: RouteRecordRaw[] = [
  { path: "/login", component: LoginPage },
  { path: "/primeiro-acesso", component: FirstAccessPage },
  { path: "/print-preview", component: PrintPreviewPage },
  {
    path: "/",
    component: AppLayout,
    children: [
      { path: "", component: DashboardPage, meta: { permission: "dashboard:view" } },
      { path: "empresas", component: EmpresaPage, meta: { permission: "empresas:view" } },
      { path: "funcionarios", component: FuncionarioPage, meta: { permission: "funcionarios:view" } },
      { path: "usuarios", component: UsuarioPage, meta: { permission: "usuarios:view" } },
      { path: "perfis", component: PerfilPage, meta: { permission: "perfis:view" } },
      { path: "jornadas", component: JornadaPage, meta: { permission: "jornadas:view" } },
      { path: "feriados", component: FeriadoPage, meta: { permission: "feriados:view" } },
      { path: "afd", component: AfdImportPage, meta: { permission: "afd:import" } },
      { path: "banco-horas", component: BancoHorasPage, meta: { permission: "banco_horas:view" } },
      { path: "tratamentos", component: TratamentosPage, meta: { permission: "tratamentos:view" } },
      { path: "fechamentos", component: FechamentoMensalPage, meta: { permission: "fechamentos:view" } },
      ...genericEntityRoutes,
      { path: "batidas", component: PunchesPage, meta: { permission: "batidas:view" } },
      { path: "batidas-lote", component: PunchBatchPage, meta: { permission: "batidas:manage" } },
      { path: "cartao-ponto", component: CartaoPontoPage, meta: { permission: "batidas:view" } },
      { path: "apuracao", component: ApuracaoPage, meta: { permission: "apuracao:view" } },
      { path: "sync-queue", component: SyncQueuePage, meta: { permission: "sync:view" } },
      { path: "relatorios", component: ReportsCenterPage, meta: { permission: "relatorios:export" } },
      { path: "relatorios/horas", component: RelatorioHorasPage, meta: { permission: "relatorios:export" } },
      { path: "relatorios-gerados", component: GeneratedReportsPage, meta: { permission: "relatorios:export" } },
      { path: "rep", component: RepExportPage, meta: { permission: "relatorios:export" } },
      { path: "conector-dashboard", component: ConectorDashboardPage, meta: { permission: "equipamentos:view" } },
      { path: "conector-config", redirect: "/conector-dashboard" },
      { path: "sistema", component: SystemPage, meta: { permission: "config:view", feature: "systemSettings" } },
      { path: "sistema/banco", component: DatabasePage, meta: { permission: "config:view", feature: "databaseSettings" } },
      { path: "licenciamento", component: LicensingPage, meta: { permission: "config:view", feature: "licensing" } },
      { path: "sobre", component: AboutPage, meta: { feature: "about" } },
      { path: "logs", component: AppLogsPage, meta: { permission: "config:view", feature: "logs" } },
      { path: "runtime", component: RuntimeDiagnosticsPage, meta: { permission: "config:view" } },
      { path: "documentacao/guia", component: UserGuidePage, meta: { feature: "userGuide" } },
      { path: "ficha-tecnica", component: TechnicalSheetPage, meta: { permission: "config:view", feature: "technicalSheet" } },
      { path: "sincronizacao", component: SyncPage, meta: { permission: "config:view", feature: "sync" } },
      { path: "api-interna", component: InternalApiPage, meta: { permission: "config:view", feature: "internalApi" } },
      { path: "documentacao/scalar", component: ScalarDocsPage, meta: { permission: "config:view", feature: "scalarDocs" } },
      { path: "webhooks", component: WebhookServicePage, meta: { permission: "config:view", feature: "webhookService" } },
      { path: "websocket", component: WebSocketServicePage, meta: { permission: "config:view", feature: "websocketService" } },
      { path: "integracoes", component: IntegrationPage, meta: { permission: "config:view", feature: "integrations" } },
    ]
  }
];

const router = createRouter({ history: createWebHashHistory(), routes });

router.beforeEach(async (to) => {
  const session = useSessionStore();
  if (!session.initialized) {
    try {
      await session.restore();
    } catch (error) {
      logAppError("router", "Falha ao restaurar sessão durante navegação.", {
        to: to.fullPath,
        error: error instanceof Error ? error.message : String(error),
      });
      session.clearAuthState();
    }
  }

  if (to.path !== "/login" && !session.isAuthenticated) {
    logAppWarning("router", "Navegação bloqueada por ausência de autenticação.", { to: to.fullPath });
    return "/login";
  }
  if (session.isAuthenticated && session.user?.senha_provisoria && to.path !== "/primeiro-acesso") {
    logAppWarning("auth", "Acesso restrito até a troca da senha temporária.", { to: to.fullPath });
    return "/primeiro-acesso";
  }
  if (session.isAuthenticated && !session.user?.senha_provisoria && to.path === "/primeiro-acesso") {
    return "/";
  }
  if (to.path === "/login" && session.isAuthenticated) return "/";

  const requiredFeature = to.meta?.feature as keyof typeof appFeatures | undefined;
  if (!isFeatureEnabled(requiredFeature)) {
    logAppWarning("router", "Rota bloqueada porque o módulo está desativado.", { to: to.fullPath, feature: requiredFeature });
    return "/";
  }

  const requiredPermission = to.meta?.permission as string | undefined;
  if (requiredPermission && !session.can(requiredPermission)) {
    logAppWarning("router", "Navegação bloqueada por permissão insuficiente.", {
      to: to.fullPath,
      permission: requiredPermission,
      user: session.user?.login,
    });
    return "/";
  }

  return true;
});

router.afterEach((to) => {
  const session = useSessionStore();
  logAppInfo("navigation", "Rota carregada.", {
    to: to.fullPath,
    authenticated: session.isAuthenticated,
    user: session.user?.login ?? null,
  });
});

router.onError((error) => {
  logAppError("router", "Erro interno de roteamento.", {
    error: error instanceof Error ? error.message : String(error),
  });
});

export default router;
