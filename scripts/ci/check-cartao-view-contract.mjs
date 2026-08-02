import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../../src/pages/CartaoPontoPage.vue", import.meta.url), "utf8");

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
    /@page\{size:A4 \$\{orientation\};margin:\$\{margin\}\}[\s\S]*?@media screen[\s\S]*?\.report-page\{width:\$\{pageWidth\};min-height:\$\{pageHeight\};padding:\$\{margin\}/,
    "A prévia deve materializar a mesma página A4, orientação e margens usadas na impressão.",
  ],
  [
    /\.report-page\{width:auto;min-height:0;margin:0;padding:0[\s\S]*?break-after:page/,
    "A estrutura de páginas da prévia deve controlar as mesmas quebras da impressão.",
  ],
  [
    /const rowsPerPageByModel:[\s\S]*?folha_completa:\s*25[\s\S]*?data-page-count="\$\{pageRows\.length\}"/,
    "Cada modelo deve dividir intervalos extensos em páginas reais conforme sua densidade de colunas.",
  ],
  [
    /<iframe[\s\S]*?:srcdoc="reportHtml"[\s\S]*?sandbox[\s\S]*?referrerpolicy="no-referrer"/,
    "O HTML real do relatório deve ser isolado com sandbox dentro da prévia.",
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

const documentActions = source.match(/<div class="cartao-document-actions">([\s\S]*?)<\/div>/)?.[1] || "";
for (const label of ["Atualizar", "Exportar HTML", "Exportar Excel", "Imprimir competência", "Imprimir / Salvar PDF"]) {
  if (!documentActions.includes(label)) {
    throw new Error(`A ação ${label} deve permanecer visível nos dois modos do cartão.`);
  }
}
if (/v-if="activeView/.test(documentActions)) {
  throw new Error("As ações do documento não podem desaparecer ao alternar entre edição e pré-visualização.");
}

console.log("Cartão de ponto validado: modos na Title Bar, página A4 responsiva, paginação real, competência atual e relatório sincronizado.");
