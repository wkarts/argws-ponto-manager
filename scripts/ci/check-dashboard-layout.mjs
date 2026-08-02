import { readFile } from "node:fs/promises";

const [dashboardSource, stylesSource] = await Promise.all([
  readFile(new URL("../../src/pages/DashboardPage.vue", import.meta.url), "utf8"),
  readFile(new URL("../../src/styles.css", import.meta.url), "utf8"),
]);

const sharedGridUsages = dashboardSource.match(/dashboard-card-grid/g) || [];
if (sharedGridUsages.length !== 2) {
  throw new Error(
    `A grade proporcional deve ser aplicada aos dois grupos de três cards; encontrados ${sharedGridUsages.length} usos.`,
  );
}

const responsiveGridRule = /\.dashboard-page\s+\.dashboard-card-grid\s*\{[^}]*repeat\(auto-fit,\s*minmax\(min\(100%,\s*320px\),\s*1fr\)\)/s;
if (!responsiveGridRule.test(stylesSource)) {
  throw new Error("A dashboard não possui a grade auto-fit baseada na largura útil do contêiner.");
}

const proportionalCardRule = /\.dashboard-page\s+\.dashboard-card-grid\s*>\s*\.card[^{]*\{[^}]*grid-column:\s*auto\s*!important[^}]*min-height:\s*190px\s*!important[^}]*height:\s*100%/s;
if (!proportionalCardRule.test(stylesSource)) {
  throw new Error("Os seis cards não compartilham o mesmo contrato de largura, altura e ocupação da grade.");
}

console.log("Grade da dashboard validada: três, duas ou uma coluna conforme a largura útil, sem espaços residuais.");
