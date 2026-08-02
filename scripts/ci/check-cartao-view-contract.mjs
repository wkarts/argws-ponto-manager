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
];

for (const [pattern, message] of requirements) {
  if (!pattern.test(source)) {
    throw new Error(message);
  }
}

console.log("Cartão de ponto validado: edição e prévia independentes, competência atual padrão e relatório sincronizado.");
