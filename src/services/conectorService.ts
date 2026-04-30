import { invoke } from "@tauri-apps/api/core";

export const testarConector = () => invoke<string>("conector_testar");

export const batidasPorData = (payload: {
  device_id: string;
  data_inicio: string;
  data_fim: string;
}) => invoke("conector_batidas_por_data", payload);

export const batidasPorNSR = (payload: {
  device_id: string;
  nsr_inicio: number;
  nsr_fim?: number;
}) => invoke("conector_batidas_por_nsr", payload);

export const baixarAFD = (device_id: string) => invoke("conector_baixar_afd", { device_id });
