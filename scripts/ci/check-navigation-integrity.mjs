import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../../src/config/navigation.ts", import.meta.url), "utf8");
const entries = [...source.matchAll(
  /\{ title: "([^"]+)", route: "([^"]+)"[^}]+section: "([^"]+)", icon: "([^"]+)"/g,
)].map((match) => ({ title: match[1], route: match[2], section: match[3], icon: match[4] }));

if (entries.length < 40) {
  throw new Error(`Inventário lateral incompleto: apenas ${entries.length} itens foram encontrados.`);
}

const missingIcon = entries.filter((item) => !item.icon.trim());
if (missingIcon.length) {
  throw new Error(`Itens sem ícone: ${missingIcon.map((item) => item.title).join(", ")}`);
}

const iconOwners = new Map();
for (const item of entries) {
  const owners = iconOwners.get(item.icon) || [];
  owners.push(item.title);
  iconOwners.set(item.icon, owners);
}
const repeatedIcons = [...iconOwners.entries()].filter(([, owners]) => owners.length > 1);
if (repeatedIcons.length) {
  throw new Error(
    `Ícones semânticos repetidos: ${repeatedIcons.map(([icon, owners]) => `${icon} (${owners.join(", ")})`).join("; ")}`,
  );
}

const expectedOperationalOrder = [
  "/batidas",
  "/cartao-ponto",
  "/tratamentos",
  "/afd",
  "/apuracao",
  "/banco-horas",
  "/fechamentos",
  "/batidas-lote",
];
const operationalOrder = entries
  .filter((item) => item.section === "operacao")
  .map((item) => item.route);
if (JSON.stringify(operationalOrder) !== JSON.stringify(expectedOperationalOrder)) {
  throw new Error(`Ordem operacional incompatível com a 1.23.x: ${operationalOrder.join(" -> ")}`);
}

console.log(`${entries.length} itens de menu validados com ícones exclusivos e ordem operacional compatível.`);
