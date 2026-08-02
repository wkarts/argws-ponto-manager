import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../../src/pages/CartaoPontoPage.vue", import.meta.url), "utf8");
const styles = await readFile(new URL("../../src/styles.css", import.meta.url), "utf8");
const portraitReportCss = await readFile(new URL("../../src/print/cartao-report-portrait.css", import.meta.url), "utf8");
const landscapeReportCss = await readFile(new URL("../../src/print/cartao-report-landscape.css", import.meta.url), "utf8");

const requirements = [
  [
    /const activeView = ref<"edicao" \| "previsualizacao">\("edicao"\)/,
    "O Cartão de ponto deve abrir no modo de edição.",
  ],
  [
    /modoPeriodo:\s*"competencia"\s+as\s+"intervalo"\s*\|\s*"competencia"/,
    "O filtro inicial deve usar a competência atual.",
  ],
  [
    /const competenciaAtual = getCompetenciaRange\(hoje\.getFullYear\(\), hoje\.getMonth\(\) \+ 1\)/,
    "A competência inicial deve derivar do mês e ano correntes.",
  ],
  [
    /id="cartao-panel-preview"[\s\S]*?:srcdoc="reportHtml"/,
    "A guia de pré-visualização deve renderizar o mesmo HTML usado no relatório.",
  ],
  [
    /<AppPageTitleBar[^>]*class="cartao-page-titlebar"[\s\S]*?<template #actions>[\s\S]*?<nav class="cartao-view-tabs"/,
    "Os controles Editar e Pré-visualizar devem estar integrados às ações da Title Bar.",
  ],
  [
    /Editar cartão[\s\S]*?Pré-visualizar/,
    "A Title Bar deve oferecer alternância compacta entre edição e pré-visualização.",
  ],
  [
    /v-if="activeView === 'previsualizacao'"[\s\S]*?id="cartao-panel-preview"/,
    "A pré-visualização deve possuir um painel independente.",
  ],
  [
    /v-else[\s\S]*?id="cartao-panel-edicao"[\s\S]*?class="cartao-vb6-shell"/,
    "A grade de edição deve ficar fora da guia de pré-visualização.",
  ],
  [
    /watch\(\(\) => filtros\.modeloRelatorio,[\s\S]*?reportHtml\.value = buildCartaoHtml\(\)/,
    "A prévia deve acompanhar o modelo de relatório selecionado.",
  ],
  [
    /function isCartaoModeloPaisagem[\s\S]*?folha_interjornada[\s\S]*?folha_com_he[\s\S]*?folha_completa/,
    "Os modelos com mais colunas devem preservar a orientação A4 paisagem.",
  ],
  [
    /cartao_ponto:\s*`[\s\S]*?<th>Dia<\/th><th>Ent\.1<\/th><th>Saí\.1<\/th><th>Ent\.2<\/th><th>Saí\.2<\/th><th>Ent\.3<\/th><th>Saí\.3<\/th><th>Normais<\/th><th>Faltas<\/th><th>Extras<\/th><th>Observações<\/th>[\s\S]*?r\.ent1[\s\S]*?r\.sai1[\s\S]*?r\.ent2[\s\S]*?r\.sai2[\s\S]*?r\.ent3[\s\S]*?r\.sai3[\s\S]*?r\.normal[\s\S]*?r\.falta[\s\S]*?r\.extra[\s\S]*?r\.ocorrencias[\s\S]*?<td colspan="7">TOTAIS<\/td>[\s\S]*?totals\.normal[\s\S]*?totals\.falta[\s\S]*?totals\.extra/,
    "O relatório padrão deve preservar a grade clássica da versão 1.21.3/1.22.0, com seis marcações, normais, faltas, extras, observações e totais.",
  ],
  [
    /import cartaoLandscapeCss from "\.\.\/print\/cartao-report-landscape\.css\?raw"[\s\S]*?import cartaoLandscapeCssUrl from "\.\.\/print\/cartao-report-landscape\.css\?url"[\s\S]*?import cartaoPortraitCss from "\.\.\/print\/cartao-report-portrait\.css\?raw"[\s\S]*?import cartaoPortraitCssUrl from "\.\.\/print\/cartao-report-portrait\.css\?url"/,
    "O relatório deve carregar o mesmo CSS como conteúdo de impressão e como stylesheet empacotado para o WebView2.",
  ],
  [
    /function cartaoPrintCss\(modelo = filtros\.modeloRelatorio\): string \{[\s\S]*?return isCartaoModeloPaisagem\(modelo\) \? cartaoLandscapeCss : cartaoPortraitCss;[\s\S]*?function cartaoPrintStylesheetUrl\(modelo = filtros\.modeloRelatorio\): string \{[\s\S]*?return isCartaoModeloPaisagem\(modelo\) \? cartaoLandscapeCssUrl : cartaoPortraitCssUrl;/,
    "O CSS da prévia e da impressão deve escolher a mesma orientação a partir do modelo selecionado.",
  ],
  [
    /<body>\s*<div class="head">[\s\S]*?<table>[\s\S]*?tableByModel\[filtros\.modeloRelatorio\][\s\S]*?<div class="sign">/,
    "A prévia deve usar o documento contínuo original, com cabeçalho, tabela, totais e assinaturas.",
  ],
  [
    /\.card-page\{page-break-after:always;break-after:page\}[\s\S]*?\.card-page:last-child\{page-break-after:auto;break-after:auto\}/,
    "A impressão em lote deve separar colaboradores sem paginar manualmente as linhas do cartão.",
  ],
  [
    /<iframe[\s\S]*?:srcdoc="reportHtml"[\s\S]*?sandbox="allow-same-origin"[\s\S]*?referrerpolicy="no-referrer"/,
    "O iframe deve preservar a origem necessária ao CSS no WebView2 sem liberar scripts.",
  ],
  [
    /<link rel="stylesheet" href="\$\{cartaoPrintStylesheetUrl\(filtros\.modeloRelatorio\)\}">[\s\S]*?<style>\$\{cartaoPrintCss\(filtros\.modeloRelatorio\)\}<\/style>/,
    "O documento real deve carregar o CSS empacotado no WebView2 e manter o mesmo conteúdo inline para impressão e exportação.",
  ],
  [
    /<div class="cartao-sticky-header">[\s\S]*?<AppPageTitleBar[\s\S]*?<BaseFilterBar/,
    "A Title Bar e os filtros devem compartilhar o cabeçalho fixo do cartão.",
  ],
  [
    /<tr v-for="row in dailyGridRows"[\s\S]*?@click="selectDay\(row\.isoDate\)"/,
    "Cada linha da grade deve continuar selecionável mesmo quando contém marcações protegidas.",
  ],
];

for (const [pattern, message] of requirements) {
  if (!pattern.test(source)) {
    throw new Error(message);
  }
}

if (/previewExpanded/.test(source)) {
  throw new Error("A prévia não deve depender de um painel reduzido que esconda o formato real da página.");
}

for (const forbidden of ["rowsPerPageByModel", "data-page-count", "class=\"report-page", "zoom:.82", "zoom:.31"]) {
  if (source.includes(forbidden)) {
    throw new Error(`A prévia não pode recriar a paginação manual removida da versão 1.26.2: ${forbidden}`);
  }
}

if (source.includes("<th>Marcações do dia</th><th>Total trabalhado</th><th>Jornada esperada</th><th>Saldo do dia</th>")) {
  throw new Error("O modelo padrão não pode voltar ao cabeçalho sintético introduzido na versão 1.23.0.");
}

if (/sandbox=""|sandbox="[^"]*allow-scripts/.test(source)) {
  throw new Error("A prévia não pode usar origem opaca nem liberar scripts no iframe do relatório.");
}

const styleRequirements = [
  [/\.cartao-sticky-header\s*\{[\s\S]*?position:\s*sticky[\s\S]*?top:\s*0/, "O cabeçalho do cartão deve permanecer fixo em telas amplas."],
  [/\.cartao-vb6-grid-panel\s*\{[\s\S]*?overflow:\s*auto/, "A rolagem deve permanecer concentrada na grade operacional."],
  [/\.cartao-vb6-side \.side-content\s*\{[\s\S]*?grid-template-rows:\s*auto minmax\(0, 1fr\)[\s\S]*?overflow:\s*hidden/, "As abas laterais devem permanecer fixas enquanto o conteúdo do painel rola."],
  [/\.report-frame\s*\{\s*width:\s*100%;\s*min-height:\s*70vh;\s*border:\s*1px solid #d1d5db;\s*border-radius:\s*12px;\s*background:\s*white;\s*\}/, "O iframe deve preservar o enquadramento visual original da versão 1.23.4."],
  [/\.cartao-preview-card\.preview-only\s*\{[\s\S]*?padding:\s*10px !important;[\s\S]*?overflow:\s*auto;[\s\S]*?background:\s*var\(--card-bg\);/, "A guia de pré-visualização deve manter o cartão branco e o respiro do CSS histórico."],
];
for (const [pattern, message] of styleRequirements) {
  if (!pattern.test(styles)) throw new Error(message);
}

const reportCssRequirements = [
  [portraitReportCss, "portrait", "9px", "8.2px", "2px 3px", "15px", "8.5px", "12px", "7.5px", "120px"],
  [landscapeReportCss, "landscape", "8.5px", "7.4px", "1.6px 2.4px", "14px", "8px", "10px", "7px", "112px"],
];
for (const [css, orientation, bodySize, tableSize, padding, titleSize, metaSize, signatureMargin, legendSize, logoWidth] of reportCssRequirements) {
  const expectedFragments = [
    `@page{size:A4 ${orientation};margin:6mm}`,
    `body{font-family:Consolas,monospace;margin:0;color:#111;font-size:${bodySize}}`,
    `.head{display:grid;grid-template-columns:1fr auto;gap:6px;align-items:end;border-bottom:1px solid #333;padding-bottom:3px}`,
    `h1{margin:0;font-size:${titleSize};line-height:1.1}`,
    `.meta{font-size:${metaSize};line-height:1.15}`,
    `table{width:100%;border-collapse:collapse;font-size:${tableSize};margin-top:4px;table-layout:fixed}`,
    `th,td{border:1px solid #808080;padding:${padding};text-align:left;vertical-align:top;word-break:break-word;line-height:1.12}`,
    `.sign{margin-top:${signatureMargin};display:grid;grid-template-columns:1fr 1fr;gap:18px;text-align:center}`,
    `.legend{font-size:${legendSize};margin-top:4px}`,
    `svg{max-width:${logoWidth};height:auto}`,
  ];
  for (const fragment of expectedFragments) {
    if (!css.includes(fragment)) {
      throw new Error(`O CSS ${orientation} divergiu do contrato visual da versão 1.23.4: ${fragment}`);
    }
  }
}

for (const forbidden of ["const isClassicCard", "height: 76vh", "background: #e9eef5", ".cartao-preview-card.expanded"]) {
  if (source.includes(forbidden) || styles.includes(forbidden)) {
    throw new Error(`A prévia não pode reintroduzir a ampliação artificial removida da versão 1.26.5: ${forbidden}`);
  }
}

const documentActions = source.match(/<div class="cartao-document-actions">([\s\S]*?)<\/div>/)?.[1] || "";
for (const label of ["Atualizar", "Exportar HTML", "Exportar Excel", "Imprimir competência", "Imprimir / Salvar PDF"]) {
  if (!documentActions.includes(label)) {
    throw new Error(`A ação ${label} deve permanecer visível nos dois modos do cartão.`);
  }
}
if (/v-if="activeView/.test(documentActions)) {
  throw new Error("As ações do documento não podem desaparecer ao alternar entre edição e pré-visualização.");
}

console.log("Cartão de ponto validado: CSS da versão 1.23.4, iframe compatível com WebView2, modos na Title Bar e relatório sincronizado.");
