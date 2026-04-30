import { invokeCommand } from "./tauri";

export const testarConector = (equipamento_id?: number) =>
  invokeCommand<string>("conector_testar", equipamento_id ? { equipamento_id } : {});

export const coletarBatidasConector = (payload: {
  equipamento_id: number;
  completo?: boolean;
  nsr_inicio?: number;
  nsr_fim?: number;
  data_inicio?: string;
  data_fim?: string;
}) => invokeCommand("conector_coletar_batidas", payload);

export const baixarAfdConector = (equipamento_id: number) =>
  invokeCommand("conector_baixar_afd", { equipamento_id });

export const dashboardConector = () => invokeCommand("conector_dashboard");

export const carregarConfiguracaoConector = () =>
  invokeCommand<Record<string, unknown>>("conector_configuracao_carregar");

export const salvarConfiguracaoConector = (payload: {
  base_url: string;
  api_token?: string | null;
  timeout_secs?: number | null;
}) => invokeCommand<Record<string, unknown>>("conector_configuracao_salvar", payload);

export const importarAfdConector = (payload: {
  empresa_id?: number | null;
  equipamento_id: number;
  mode?: string | null;
  completo?: boolean | null;
  nsr_inicio?: number | null;
  nsr_fim?: number | null;
  data_inicio?: string | null;
  data_fim?: string | null;
}) => invokeCommand<Record<string, unknown>>("conector_importar_afd", { args: payload });
