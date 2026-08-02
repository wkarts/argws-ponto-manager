import { readFile } from "node:fs/promises";

const componentSource = await readFile(new URL("../../src/components/base/BaseFilterBar.vue", import.meta.url), "utf8");

const componentRequirements = [
  [/container:\s*filterbar\s*\/\s*inline-size/, "A barra deve responder à largura útil do próprio contêiner."],
  [/grid-template-columns:\s*repeat\(12,\s*minmax\(0,\s*1fr\)\)/, "A grade compartilhada de filtros deve usar doze colunas proporcionais."],
  [/@container\s+filterbar\s*\(max-width:\s*980px\)/, "A barra deve possuir adaptação para larguras médias."],
  [/@container\s+filterbar\s*\(max-width:\s*500px\)/, "A barra deve possuir adaptação para larguras pequenas."],
  [/filter-field--date/, "A barra deve oferecer proporção compacta para datas."],
  [/filter-field--status/, "A barra deve oferecer proporção compacta para status."],
  [/filter-field--search/, "A barra deve oferecer proporção flexível para buscas."],
  [/collapsible\?:\s*boolean/, "A barra deve permitir recolher filtros extensos."],
  [/aria-expanded/, "O recolhimento deve expor seu estado para tecnologias assistivas."],
  [/aria-controls/, "O controle de recolhimento deve apontar para o conteúdo controlado."],
  [/<slot name="actions"\s*\/>/, "A barra deve centralizar as ações de aplicar e limpar."],
];

for (const [pattern, message] of componentRequirements) {
  if (!pattern.test(componentSource)) throw new Error(message);
}

const filterPages = [
  "AppLogsPage.vue",
  "ApuracaoPage.vue",
  "BancoHorasPage.vue",
  "CartaoPontoPage.vue",
  "ConectorDashboardPage.vue",
  "EmpresaPage.vue",
  "EntityPage.vue",
  "FechamentoMensalPage.vue",
  "FeriadoPage.vue",
  "FuncionarioPage.vue",
  "GeneratedReportsPage.vue",
  "PerfilPage.vue",
  "PunchBatchPage.vue",
  "PunchesPage.vue",
  "RelatorioHorasPage.vue",
  "ReportsCenterPage.vue",
  "TratamentosPage.vue",
  "UsuarioPage.vue",
];

const invalidPages = [];
for (const page of filterPages) {
  const source = await readFile(new URL(`../../src/pages/${page}`, import.meta.url), "utf8");
  const requirements = [
    /import BaseFilterBar from "\.\.\/components\/base\/BaseFilterBar\.vue";/,
    /<BaseFilterBar\b/,
    /<template #actions>/,
    /Limpar filtros/,
  ];
  if (requirements.some((pattern) => !pattern.test(source))) invalidPages.push(page);
}

if (invalidPages.length) {
  throw new Error(`Views sem o contrato completo da barra de filtros: ${invalidPages.join(", ")}`);
}

const titlebarFilterPages = ["EntityPage.vue", "FeriadoPage.vue"];
for (const page of titlebarFilterPages) {
  const source = await readFile(new URL(`../../src/pages/${page}`, import.meta.url), "utf8");
  const actionsBlock = source.match(/<template #actions>([\s\S]*?)<\/template>/)?.[1] || "";
  if (/<input\b|<select\b/.test(actionsBlock)) {
    throw new Error(`${page} ainda mantém campos de filtro dentro da TitleBar.`);
  }
}

console.log(`${filterPages.length} views validadas com BaseFilterBar responsiva, ações consistentes e limpeza de filtros.`);
