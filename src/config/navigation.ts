import { appFeatures } from "./projectConfig";

export type MenuSection = "dashboard" | "cadastro" | "sistema" | "ferramentas" | "documentacao";

export interface MenuItemConfig {
  title: string;
  route: string;
  permission?: string;
  feature?: keyof typeof appFeatures;
  section: MenuSection;
  eyebrow?: string;
  description?: string;
}

export const menuItems: MenuItemConfig[] = [
  { title: "Dashboard", route: "/", permission: "dashboard:view", section: "dashboard" },

  { title: "Empresas", route: "/empresas", permission: "empresas:view", section: "cadastro" },
  { title: "Funcionários", route: "/funcionarios", permission: "funcionarios:view", section: "cadastro" },
  { title: "Departamentos", route: "/departamentos", permission: "funcionarios:view", feature: "genericEntities", section: "cadastro" },
  { title: "Funções", route: "/funcoes", permission: "funcionarios:view", feature: "genericEntities", section: "cadastro" },
  { title: "Centros de custo", route: "/centros-custo", permission: "funcionarios:view", feature: "genericEntities", section: "cadastro" },
  { title: "Horários", route: "/horarios", permission: "horarios:view", feature: "genericEntities", section: "cadastro" },
  { title: "Escalas", route: "/escalas", permission: "escalas:view", feature: "genericEntities", section: "cadastro" },
  { title: "Jornadas", route: "/jornadas", permission: "jornadas:view", section: "cadastro" },
  { title: "Regras por contexto", route: "/jornada-contextos", permission: "jornadas:view", feature: "genericEntities", section: "cadastro" },
  { title: "Feriados", route: "/feriados", permission: "feriados:view", section: "cadastro" },
  { title: "Férias", route: "/ferias", permission: "ferias:view", feature: "genericEntities", section: "cadastro" },
  { title: "Equipamentos", route: "/equipamentos", permission: "equipamentos:view", feature: "genericEntities", section: "cadastro" },
  { title: "Eventos", route: "/eventos", permission: "eventos:view", feature: "genericEntities", section: "cadastro" },
  { title: "Justificativas", route: "/justificativas", permission: "justificativas:view", feature: "genericEntities", section: "cadastro" },

  { title: "Usuários", route: "/usuarios", permission: "usuarios:view", section: "sistema" },
  { title: "Perfis de usuários", route: "/perfis", permission: "perfis:view", section: "sistema" },
  { title: "Logs", route: "/logs", permission: "config:view", feature: "logs", section: "sistema" },
  { title: "Parâmetros", route: "/sistema", permission: "config:view", feature: "systemSettings", section: "sistema" },
  { title: "Banco de dados", route: "/sistema/banco", permission: "config:view", feature: "databaseSettings", section: "sistema" },
  { title: "Diagnósticos", route: "/runtime", permission: "config:view", section: "sistema" },

  { title: "API Interna", route: "/api-interna", permission: "config:view", feature: "internalApi", section: "sistema" },
  { title: "Documentação Scalar", route: "/documentacao/scalar", permission: "config:view", feature: "scalarDocs", section: "sistema" },
  { title: "Webhooks", route: "/webhooks", permission: "config:view", feature: "webhookService", section: "sistema" },
  { title: "WebSocket", route: "/websocket", permission: "config:view", feature: "websocketService", section: "sistema" },
  { title: "Importar AFD", route: "/afd", permission: "afd:import", section: "ferramentas" },
  { title: "Batidas", route: "/batidas", permission: "batidas:view", section: "ferramentas" },
  { title: "Batidas em lote", route: "/batidas-lote", permission: "batidas:manage", section: "ferramentas" },
  { title: "Apuração", route: "/apuracao", permission: "apuracao:view", section: "ferramentas" },
  { title: "Tratamento de ponto", route: "/tratamentos", permission: "tratamentos:view", section: "ferramentas" },
  { title: "Banco de horas", route: "/banco-horas", permission: "banco_horas:view", section: "ferramentas" },
  { title: "Fechamento mensal", route: "/fechamentos", permission: "fechamentos:view", section: "ferramentas" },
  { title: "Cartão de ponto", route: "/cartao-ponto", permission: "batidas:view", section: "ferramentas" },
  { title: "Relatórios", route: "/relatorios", permission: "relatorios:export", section: "ferramentas" },
  { title: "Relatório de horas", route: "/relatorios/horas", permission: "relatorios:export", section: "ferramentas" },
  { title: "Relatórios gerados", route: "/relatorios-gerados", permission: "relatorios:export", section: "ferramentas" },
  { title: "Exportação REP", route: "/rep", permission: "relatorios:export", section: "ferramentas" },
  { title: "Conector", route: "/conector-dashboard", permission: "equipamentos:view", section: "ferramentas" },
  { title: "Fila de sincronização", route: "/sync-queue", permission: "sync:view", section: "ferramentas" },
  { title: "Integrações", route: "/integracoes", permission: "config:view", feature: "integrations", section: "ferramentas" },
  { title: "Ficha técnica", route: "/ficha-tecnica", permission: "config:view", feature: "technicalSheet", section: "ferramentas" },

  { title: "Sobre", route: "/sobre", feature: "about", section: "documentacao" },
  { title: "Guia do usuário", route: "/documentacao/guia", feature: "userGuide", section: "documentacao" },
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
