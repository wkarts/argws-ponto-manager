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
import LoginPage from "../pages/LoginPage.vue";
import PunchesPage from "../pages/PunchesPage.vue";
import ApuracaoPage from "../pages/ApuracaoPage.vue";
import SyncQueuePage from "../pages/SyncQueuePage.vue";
import SystemPage from "../pages/SystemPage.vue";
import LicensingPage from "../pages/LicensingPage.vue";
import ReportsCenterPage from "../pages/ReportsCenterPage.vue";
import RepExportPage from "../pages/RepExportPage.vue";
import PunchBatchPage from "../pages/PunchBatchPage.vue";
import AboutPage from "../pages/AboutPage.vue";
import { entityConfigs } from "../config/entities";
import { useSessionStore } from "../stores/session";

const permissionByEntity: Record<string, string> = {
  departamentos: "funcionarios:view",
  funcoes: "funcionarios:view",
  centro_custos: "funcionarios:view",
  horarios: "horarios:view",
  escalas: "escalas:view",
  equipamentos: "equipamentos:view",
  eventos: "eventos:view",
  justificativas: "justificativas:view"
};

const genericEntityRoutes: RouteRecordRaw[] = Object.values(entityConfigs)
  .filter((entity) => !["empresas", "funcionarios", "usuarios"].includes(entity.key))
  .map((entity) => ({
    path: entity.route,
    component: EntityPage,
    props: { entityKey: entity.key },
    meta: { permission: permissionByEntity[entity.key] }
  }));

const routes: RouteRecordRaw[] = [
  { path: "/login", component: LoginPage },
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
      { path: "afd", component: AfdImportPage, meta: { permission: "afd:import" } },
      { path: "banco-horas", component: BancoHorasPage, meta: { permission: "banco_horas:view" } },
      { path: "tratamentos", component: TratamentosPage, meta: { permission: "tratamentos:view" } },
      { path: "fechamentos", component: FechamentoMensalPage, meta: { permission: "fechamentos:view" } },
      ...genericEntityRoutes,
      { path: "batidas", component: PunchesPage, meta: { permission: "batidas:view" } },
      { path: "apuracao", component: ApuracaoPage, meta: { permission: "apuracao:view" } },
      { path: "sync-queue", component: SyncQueuePage, meta: { permission: "sync:view" } },
      { path: "sistema", component: SystemPage, meta: { permission: "config:view" } },
      { path: "licenciamento", component: LicensingPage, meta: { permission: "config:view" } },
      { path: "relatorios", component: ReportsCenterPage, meta: { permission: "relatorios:export" } },
      { path: "rep", component: RepExportPage, meta: { permission: "relatorios:export" } },
      { path: "batidas-lote", component: PunchBatchPage, meta: { permission: "batidas:manage" } },
      { path: "sobre", component: AboutPage }
    ]
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes
});

router.beforeEach(async (to) => {
  const session = useSessionStore();
  if (!session.initialized) {
    await session.restore();
  }

  if (to.path !== "/login" && !session.isAuthenticated) {
    return "/login";
  }
  if (to.path === "/login" && session.isAuthenticated) {
    return "/";
  }

  const requiredPermission = to.meta?.permission as string | undefined;
  if (requiredPermission && !session.can(requiredPermission)) {
    return "/";
  }

  return true;
});

export default router;
