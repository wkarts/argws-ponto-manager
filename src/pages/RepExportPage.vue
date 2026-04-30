<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { comboList, exportRepEmpresaTxt, exportRepFuncionariosTxt, type ComboOption } from "../services/crud";
import { baixarAFD, batidasPorData, batidasPorNSR, testarConector } from "../services/conectorService";
import { useSessionStore } from "../stores/session";

const session = useSessionStore();
const companies = ref<ComboOption[]>([]);
const brand = ref("henry");
const empresaId = ref<number | null>(null);
const message = ref("");
const error = ref("");
const conectorDeviceId = ref("");

function download(fileName: string, content: string) {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 500);
}

async function loadCompanies() {
  const all = await comboList("empresas");
  companies.value = session.isMaster ? all : all.filter((item) => session.user?.company_ids.includes(item.id));
  empresaId.value = session.activeCompanyId ?? companies.value[0]?.id ?? null;
}

async function exportEmpresa() {
  if (!empresaId.value) return;
  error.value = "";
  const result = await exportRepEmpresaTxt(brand.value, empresaId.value);
  download(String(result.file_name), String(result.content || ""));
  message.value = `Cadastro da empresa exportado para ${brand.value.toUpperCase()}.`;
}

async function exportFuncionarios() {
  if (!empresaId.value) return;
  error.value = "";
  const result = await exportRepFuncionariosTxt(brand.value, empresaId.value);
  download(String(result.file_name), String(result.content || ""));
  message.value = `Funcionários exportados para ${brand.value.toUpperCase()} (${result.total} registro(s)).`;
}


async function testar() {
  const res = await testarConector();
  message.value = `Conector respondeu com status ${res}.`;
}

async function coletarData() {
  if (!conectorDeviceId.value) return;
  const res = await batidasPorData({
    device_id: conectorDeviceId.value,
    data_inicio: "2026-01-01",
    data_fim: "2026-01-31",
  });
  console.log(res);
  message.value = "Coleta por data executada. Verifique o console para detalhes.";
}

async function coletarNSR() {
  if (!conectorDeviceId.value) return;
  const res = await batidasPorNSR({
    device_id: conectorDeviceId.value,
    nsr_inicio: 1,
  });
  console.log(res);
  message.value = "Coleta por NSR executada. Verifique o console para detalhes.";
}

async function baixarAfdConector() {
  if (!conectorDeviceId.value) return;
  const data = await baixarAFD(conectorDeviceId.value);
  console.log("AFD bytes:", data);
  message.value = "Download de AFD executado. Verifique o console para detalhes.";
}

watch(() => session.activeCompanyId, loadCompanies);
onMounted(loadCompanies);
</script>

<template>
  <div class="grid page-gap">
    <div class="toolbar">
      <div>
        <h2>Exportação para REP</h2>
        <div class="muted-text">Estrutura extensível para Henry, Evo, Blue, Dimep e novas marcas, com exportação separada de empresa e funcionários em TXT.</div>
      </div>
    </div>
    <div v-if="message" class="alert success">{{ message }}</div>
    <div v-if="error" class="alert error">{{ error }}</div>

    <div class="card grid page-gap">
      <div class="grid columns-3 mobile-columns-1">
        <div class="field">
          <label>Marca / layout</label>
          <select v-model="brand">
            <option value="henry">Henry</option>
            <option value="evo">Evo</option>
            <option value="blue">Blue</option>
            <option value="dimep">Dimep</option>
            <option value="generic">Genérico</option>
          </select>
        </div>
        <div class="field">
          <label>Empresa</label>
          <select v-model="empresaId">
            <option v-for="item in companies" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
        </div>
        <div class="actions align-end">
          <button class="primary" @click="exportEmpresa">Exportar empresa</button>
          <button class="secondary" @click="exportFuncionarios">Exportar funcionários</button>
        </div>
      </div>
      <div class="grid columns-3 mobile-columns-1">
        <div class="field">
          <label>Device ID do Conector</label>
          <input v-model="conectorDeviceId" placeholder="Informe o device_id" />
        </div>
        <div class="actions align-end">
          <button class="secondary" @click="testar">Testar Conector</button>
          <button class="secondary" @click="coletarData">Coletar por Data</button>
          <button class="secondary" @click="coletarNSR">Coletar por NSR</button>
          <button class="secondary" @click="baixarAfdConector">Baixar AFD</button>
        </div>
      </div>
      <div class="muted-text">A arquitetura foi preparada por adaptadores de marca, facilitando expansão futura sem acoplamento rígido ao cadastro de funcionários.</div>
    </div>
  </div>
</template>
