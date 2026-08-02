import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../../src/pages/CartaoPontoPage.vue", import.meta.url), "utf8");
const styles = await readFile(new URL("../../src/styles.css", import.meta.url), "utf8");

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
    /@page\{size:A4 \$\{orientation\};margin:\$\{margin\}\}[\s\S]*?html,body\{width:100%;background:#fff\}/,
    "O relatório deve preservar o CSS de impressão contínuo compartilhado pela prévia e pela impressão.",
  ],
  [
    /cartao_ponto:\s*`[\s\S]*?<th>Dia<\/th><th>Ent\.1<\/th><th>Saí\.1<\/th><th>Ent\.2<\/th><th>Saí\.2<\/th><th>Ent\.3<\/th><th>Saí\.3<\/th><th>Normais<\/th><th>Faltas<\/th><th>Extras<\/th><th>Observações<\/th>[\s\S]*?r\.ent1[\s\S]*?r\.sai1[\s\S]*?r\.ent2[\s\S]*?r\.sai2[\s\S]*?r\.ent3[\s\S]*?r\.sai3[\s\S]*?r\.normal[\s\S]*?r\.falta[\s\S]*?r\.extra[\s\S]*?r\.ocorrencias[\s\S]*?<td colspan="7">TOTAIS<\/td>[\s\S]*?totals\.normal[\s\S]*?totals\.falta[\s\S]*?totals\.extra/,
    "O relatório padrão deve preservar a grade clássica da versão 1.21.3/1.22.0, com seis marcações, normais, faltas, extras, observações e totais.",
  ],
  [
    /const isClassicCard = modelo === "cartao_ponto"[\s\S]*?isClassicCard \? "12px"[\s\S]*?isClassicCard \? "4px 6px"[\s\S]*?isClassicCard \? "24px"[\s\S]*?isClassicCard \? "32px"[\s\S]*?isClassicCard \? "14px"/,
    "O relatório padrão deve manter as proporções tipográficas e os espaçamentos do cartão clássico na prévia e na impressão.",
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
    /<style>\$\{cartaoPrintCss\(filtros\.modeloRelatorio\)\}<\/style>/,
    "O documento real deve incorporar o CSS clássico compartilhado pela prévia e pela impressão.",
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
];
for (const [pattern, message] of styleRequirements) {
  if (!pattern.test(styles)) throw new Error(message);
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

console.log("Cartão de ponto validado: contrato de CSS clássico compatível com WebView2, iframe sem scripts, modos na Title Bar e relatório sincronizado.");
