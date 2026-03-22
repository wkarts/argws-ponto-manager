export type EntityFieldType =
  | "text"
  | "number"
  | "date"
  | "textarea"
  | "checkbox"
  | "select"
  | "password"
  | "time"
  | "email"
  | "tel";

export interface EntityField {
  key: string;
  label: string;
  type?: EntityFieldType;
  required?: boolean;
  relationEntity?: string;
  placeholder?: string;
}

export interface EntityConfig {
  key: string;
  title: string;
  route: string;
  columns: string[];
  fields: EntityField[];
}

export const entityConfigs: Record<string, EntityConfig> = {
  empresas: {
    key: "empresas",
    title: "Empresas usuárias",
    route: "/empresas",
    columns: ["id", "nome", "documento", "cidade", "estado", "ativo"],
    fields: [
      { key: "nome", label: "Nome", required: true },
      { key: "documento", label: "Documento" },
      { key: "telefone", label: "Telefone", type: "tel" },
      { key: "email", label: "E-mail", type: "email" },
      { key: "endereco", label: "Endereço" },
      { key: "bairro", label: "Bairro" },
      { key: "cidade", label: "Cidade" },
      { key: "estado", label: "Estado" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  usuarios: {
    key: "usuarios",
    title: "Usuários",
    route: "/usuarios",
    columns: ["id", "nome", "login", "administrador", "ativo"],
    fields: [
      { key: "nome", label: "Nome", required: true },
      { key: "login", label: "Login", required: true },
      { key: "senha", label: "Senha", type: "password" },
      { key: "administrador", label: "Administrador", type: "checkbox" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  departamentos: {
    key: "departamentos",
    title: "Departamentos",
    route: "/departamentos",
    columns: ["id", "descricao", "ativo"],
    fields: [
      { key: "descricao", label: "Descrição", required: true },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  funcoes: {
    key: "funcoes",
    title: "Funções",
    route: "/funcoes",
    columns: ["id", "descricao", "ativo"],
    fields: [
      { key: "descricao", label: "Descrição", required: true },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  centro_custos: {
    key: "centro_custos",
    title: "Centros de custo",
    route: "/centros-custo",
    columns: ["id", "codigo", "descricao", "ativo"],
    fields: [
      { key: "codigo", label: "Código" },
      { key: "descricao", label: "Descrição", required: true },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  horarios: {
    key: "horarios",
    title: "Horários",
    route: "/horarios",
    columns: ["id", "numero", "descricao", "carga_horaria_minutos", "ativo"],
    fields: [
      { key: "numero", label: "Número", type: "number" },
      { key: "descricao", label: "Descrição", required: true },
      { key: "entrada_1", label: "Entrada 1", type: "time" },
      { key: "saida_1", label: "Saída 1", type: "time" },
      { key: "entrada_2", label: "Entrada 2", type: "time" },
      { key: "saida_2", label: "Saída 2", type: "time" },
      { key: "carga_horaria_minutos", label: "Carga diária (min)", type: "number" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  escalas: {
    key: "escalas",
    title: "Escalas",
    route: "/escalas",
    columns: ["id", "descricao", "horario_id", "dias_ativos", "tolerancia_minutos", "ativo"],
    fields: [
      { key: "descricao", label: "Descrição", required: true },
      { key: "horario_id", label: "Horário", type: "select", relationEntity: "horarios" },
      { key: "dias_ativos", label: "Dias ativos", placeholder: "1,2,3,4,5" },
      { key: "tolerancia_minutos", label: "Tolerância (min)", type: "number" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  equipamentos: {
    key: "equipamentos",
    title: "Equipamentos",
    route: "/equipamentos",
    columns: ["id", "codigo", "descricao", "modelo", "ip", "porta", "ativo"],
    fields: [
      { key: "empresa_id", label: "Empresa", type: "select", relationEntity: "empresas" },
      { key: "codigo", label: "Código" },
      { key: "descricao", label: "Descrição", required: true },
      { key: "modelo", label: "Modelo" },
      { key: "ip", label: "IP" },
      { key: "porta", label: "Porta", type: "number" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  eventos: {
    key: "eventos",
    title: "Eventos",
    route: "/eventos",
    columns: ["id", "codigo", "descricao", "tipo", "ativo"],
    fields: [
      { key: "codigo", label: "Código" },
      { key: "descricao", label: "Descrição", required: true },
      { key: "tipo", label: "Tipo" },
      { key: "impacta_banco_horas", label: "Impacta banco de horas", type: "checkbox" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  justificativas: {
    key: "justificativas",
    title: "Justificativas",
    route: "/justificativas",
    columns: ["id", "descricao", "abono", "ativo"],
    fields: [
      { key: "descricao", label: "Descrição", required: true },
      { key: "abono", label: "Abono", type: "checkbox" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  },
  funcionarios: {
    key: "funcionarios",
    title: "Funcionários",
    route: "/funcionarios",
    columns: [
      "id",
      "matricula",
      "nome",
      "empresa_id",
      "departamento_id",
      "funcao_id",
      "horario_id",
      "ativo"
    ],
    fields: [
      { key: "empresa_id", label: "Empresa", type: "select", relationEntity: "empresas" },
      { key: "matricula", label: "Matrícula" },
      { key: "nome", label: "Nome", required: true },
      { key: "documento", label: "Documento" },
      { key: "pis", label: "PIS" },
      { key: "email", label: "E-mail", type: "email" },
      { key: "telefone", label: "Telefone", type: "tel" },
      { key: "departamento_id", label: "Departamento", type: "select", relationEntity: "departamentos" },
      { key: "funcao_id", label: "Função", type: "select", relationEntity: "funcoes" },
      { key: "centro_custo_id", label: "Centro de custo", type: "select", relationEntity: "centro_custos" },
      { key: "horario_id", label: "Horário", type: "select", relationEntity: "horarios" },
      { key: "escala_id", label: "Escala", type: "select", relationEntity: "escalas" },
      { key: "data_admissao", label: "Data de admissão", type: "date" },
      { key: "ativo", label: "Ativo", type: "checkbox" }
    ]
  }
};
