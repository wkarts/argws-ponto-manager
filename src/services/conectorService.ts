import { invoke } from "@tauri-apps/api/core";

export const testarConector = () => invoke<string>("conector_testar");

export const coletarBatidasConector = (payload: {
  equipamento_id: number;
  completo?: boolean;
  nsr_inicio?: number;
  nsr_fim?: number;
  data_inicio?: string;
  data_fim?: string;
}) => invoke("conector_coletar_batidas", payload);

export const baixarAfdConector = (equipamento_id: number) =>
  invoke("conector_baixar_afd", { equipamento_id });
