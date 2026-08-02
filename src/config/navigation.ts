import { appFeatures } from "./projectConfig";

export type MenuSection =
  | "inicio"
  | "cadastros"
  | "operacao"
  | "relatorios_integracao"
  | "documentacao"
  | "sistema";

export interface MenuItemConfig {
  title: string;
  route: string;
  permission?: string;
  feature?: keyof typeof appFeatures;
  section: MenuSection;
  icon: string;
  eyebrow?: string;
  description?: string;
}

export const menuItems: MenuItemConfig[] = [
  { title: "Dashboard", route: "/", permission: "dashboard:view", section: "inicio", icon: "dashboard" },

  { title: "Empresas", route: "/empresas", permission: "empresas:view", section: "cadastros", icon: "building" },
  { title: "Funcionários", route: "/funcionarios", permission: "funcionarios:view", section: "cadastros", icon: "idBadge" },
  { title: "Departamentos", route: "/departamentos", permission: "funcionarios:view", feature: "genericEntities", section: "cadastros", icon: "department" },
  { title: "Funções", route: "/funcoes", permission: "funcionarios:view", feature: "genericEntities", section: "cadastros", icon: "briefcase" },
  { title: "Centros de custo", route: "/centros-custo", permission: "funcionarios:view", feature: "genericEntities", section: "cadastros", icon: "target" },
  { title: "Horários", route: "/horarios", permission: "horarios:view", feature: "genericEntities", section: "cadastros", icon: "clock" },
  { title: "Escalas", route: "/escalas", permission: "escalas:view", feature: "genericEntities", section: "cadastros", icon: "calendarRange" },
  { title: "Jornadas", route: "/jornadas", permission: "jornadas:view", section: "cadastros", icon: "route" },
  { title: "Regras por contexto", route: "/jornada-contextos", permission: "jornadas:view", feature: "genericEntities", section: "cadastros", icon: "workflow" },
  { title: "Feriados", route: "/feriados", permission: "feriados:view", section: "cadastros", icon: "calendarStar" },
  { title: "Férias", route: "/ferias", permission: "ferias:view", feature: "genericEntities", section: "cadastros", icon: "umbrella" },
  { title: "Equipamentos", route: "/equipamentos", permission: "equipamentos:view", feature: "genericEntities", section: "cadastros", icon: "device" },
  { title: "Eventos", route: "/eventos", permission: "eventos:view", feature: "genericEntities", section: "cadastros", icon: "event" },
  { title: "Justificativas", route: "/justificativas", permission: "justificativas:view", feature: "genericEntities", section: "cadastros", icon: "messageCheck" },

  { title: "Batidas", route: "/batidas", permission: "batidas:view", section: "operacao", icon: "fingerprint" },
  { title: "Cartão de ponto", route: "/cartao-ponto", permission: "batidas:view", section: "operacao", icon: "timeCard" },
  { title: "Tratamento de ponto", route: "/tratamentos", permission: "tratamentos:view", section: "operacao", icon: "clipboardCheck" },
  { title: "Importação AFD", route: "/afd", permission: "afd:import", section: "operacao", icon: "fileImport" },
  { title: "Apuração", route: "/apuracao", permission: "apuracao:view", section: "operacao", icon: "calculator" },
  { title: "Banco de horas", route: "/banco-horas", permission: "banco_horas:view", section: "operacao", icon: "hourBank" },
  { title: "Fechamento mensal", route: "/fechamentos", permission: "fechamentos:view", section: "operacao", icon: "calendarCheck" },
  { title: "Tratamento em lote", route: "/batidas-lote", permission: "batidas:manage", section: "operacao", icon: "layersCheck" },

  { title: "Relatórios", route: "/relatorios", permission: "relatorios:export", section: "relatorios_integracao", icon: "reports" },
  { title: "Relatório de horas", route: "/relatorios/horas", permission: "relatorios:export", section: "relatorios_integracao", icon: "chartClock" },
  { title: "Relatórios gerados", route: "/relatorios-gerados", permission: "relatorios:export", section: "relatorios_integracao", icon: "archive" },
  { title: "Exportação REP", route: "/rep", permission: "relatorios:export", section: "relatorios_integracao", icon: "fileExport" },
  { title: "Ponto Manager Conector", route: "/conector-dashboard", permission: "equipamentos:view", section: "relatorios_integracao", icon: "connector" },
  { title: "Fila de sincronização", route: "/sync-queue", permission: "sync:view", section: "relatorios_integracao", icon: "queue" },
  { title: "Integrações", route: "/integracoes", permission: "config:view", feature: "integrations", section: "relatorios_integracao", icon: "link" },

  { title: "Guia do usuário", route: "/documentacao/guia", feature: "userGuide", section: "documentacao", icon: "book" },
  { title: "Sobre", route: "/sobre", feature: "about", section: "documentacao", icon: "info" },

  { title: "Usuários", route: "/usuarios", permission: "usuarios:view", section: "sistema", icon: "users" },
  { title: "Perfis de usuários", route: "/perfis", permission: "perfis:view", section: "sistema", icon: "shield" },
  { title: "Parâmetros", route: "/sistema", permission: "config:view", feature: "systemSettings", section: "sistema", icon: "settings" },
  { title: "Banco de dados", route: "/sistema/banco", permission: "config:view", feature: "databaseSettings", section: "sistema", icon: "database" },
  { title: "Logs da aplicação", route: "/logs", permission: "config:view", feature: "logs", section: "sistema", icon: "logs" },
  { title: "Diagnósticos", route: "/runtime", permission: "config:view", section: "sistema", icon: "stethoscope" },
  { title: "API interna", route: "/api-interna", permission: "config:view", feature: "internalApi", section: "sistema", icon: "api" },
  { title: "Documentação Scalar", route: "/documentacao/scalar", permission: "config:view", feature: "scalarDocs", section: "sistema", icon: "docs" },
  { title: "Webhooks", route: "/webhooks", permission: "config:view", feature: "webhookService", section: "sistema", icon: "webhook" },
  { title: "WebSocket", route: "/websocket", permission: "config:view", feature: "websocketService", section: "sistema", icon: "websocket" },
  { title: "Ficha técnica", route: "/ficha-tecnica", permission: "config:view", feature: "technicalSheet", section: "sistema", icon: "technical" },
];

export function isFeatureEnabled(feature?: keyof typeof appFeatures): boolean {
  return feature ? appFeatures[feature] !== false : true;
}

export function visibleMenuItems(): MenuItemConfig[] {
  return menuItems.filter((item) => isFeatureEnabled(item.feature));
}


export function findMenuItemByRoute(route: string): MenuItemConfig | undefined {
  const normalize = (value: string) => {
    const clean = (value || "/").replace(/\/+$/, "");
    return clean || "/";
  };

  const target = normalize(route);
  return visibleMenuItems().find((item) => normalize(item.route) === target);
}
