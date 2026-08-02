import { readFile } from "node:fs/promises";

const routerSource = await readFile(new URL("../../src/router/index.ts", import.meta.url), "utf8");
const basePageSource = await readFile(new URL("../../src/components/base/BasePage.vue", import.meta.url), "utf8");
const pageImports = [...routerSource.matchAll(/import\s+(\w+)\s+from\s+"\.\.\/pages\/([^";]+\.vue)";/g)]
  .map((match) => ({ component: match[1], path: match[2] }));

const standaloneLayouts = new Set(["LoginPage", "FirstAccessPage", "PrintPreviewPage"]);
const routedPages = pageImports.filter((page) => !standaloneLayouts.has(page.component));
const nonCanonical = [];

for (const page of routedPages) {
  const source = await readFile(new URL(`../../src/pages/${page.path}`, import.meta.url), "utf8");
  if (!/<(?:BasePage|AppPageTitleBar)\b/.test(source)) {
    nonCanonical.push(`${page.component} (${page.path})`);
  }
}

if (nonCanonical.length) {
  throw new Error(`Views autenticadas sem a TitleBar oficial: ${nonCanonical.join(", ")}`);
}

if (!/findMenuItemByRoute\(normalized\)\?\.icon/.test(basePageSource)) {
  throw new Error("BasePage deve resolver o ícone semântico pela mesma configuração da sidebar.");
}

console.log(`${routedPages.length} views autenticadas validadas com AppPageTitleBar ou BasePage.`);
