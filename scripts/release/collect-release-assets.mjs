#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const ALLOWED_SUFFIXES = [
  '.AppImage',
  '.deb',
  '.dmg',
  '.exe',
  '.msi',
  '.rpm',
  '.sig',
  '.tar.gz',
  '.zip',
];

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (!current.startsWith('--')) {
      throw new Error(`Argumento inválido: ${current}`);
    }
    const key = current.slice(2);
    const value = argv[index + 1];
    if (!value || value.startsWith('--')) {
      throw new Error(`Valor ausente para --${key}`);
    }
    options[key] = value;
    index += 1;
  }
  return options;
}

function sanitize(value) {
  return value
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^A-Za-z0-9._+-]+/g, '-')
    .replace(/-{2,}/g, '-')
    .replace(/^-+|-+$/g, '');
}

function walk(directory) {
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(absolute));
    } else if (entry.isFile()) {
      files.push(absolute);
    }
  }
  return files;
}

function sha256(file) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash('sha256');
    const stream = fs.createReadStream(file);
    stream.on('error', reject);
    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('end', () => resolve(hash.digest('hex')));
  });
}

function isReleaseAsset(file) {
  if (file.endsWith('.sha256')) {
    return false;
  }
  return ALLOWED_SUFFIXES.some((suffix) => file.endsWith(suffix));
}

function readProjectIdentity() {
  const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  const version = fs.readFileSync('VERSION', 'utf8').trim();
  return {
    product: sanitize(packageJson.name),
    version,
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const identity = readProjectIdentity();
  const input = path.resolve(args.input ?? 'artifacts');
  const output = path.resolve(args.output ?? 'release-assets');
  const product = sanitize(args.product ?? identity.product);
  const version = sanitize(args.version ?? identity.version);

  if (!fs.existsSync(input) || !fs.statSync(input).isDirectory()) {
    throw new Error(`Diretório de entrada ausente: ${input}`);
  }
  if (!product || !version) {
    throw new Error('Produto e versão são obrigatórios para nomear os artefatos.');
  }

  fs.mkdirSync(output, { recursive: true });
  if (fs.readdirSync(output).length > 0) {
    throw new Error(`Diretório de saída deve estar vazio: ${output}`);
  }

  const selected = walk(input).filter(isReleaseAsset).sort();
  if (selected.length === 0) {
    throw new Error(`Nenhum artefato distribuível encontrado em ${input}`);
  }

  const names = new Set();
  const assets = [];

  for (const source of selected) {
    const relative = path.relative(input, source);
    if (relative.startsWith('..') || path.isAbsolute(relative)) {
      throw new Error(`Artefato fora do diretório de entrada: ${source}`);
    }

    const [artifactDirectory = 'artifact'] = relative.split(path.sep);
    const context = sanitize(artifactDirectory.replace(/^release-/, ''));
    const originalName = sanitize(path.basename(source));
    const outputName = `${product}-v${version}-${context}-${originalName}`;

    if (names.has(outputName)) {
      throw new Error(`Colisão de nome de artefato: ${outputName}`);
    }
    names.add(outputName);

    const destination = path.join(output, outputName);
    fs.copyFileSync(source, destination, fs.constants.COPYFILE_EXCL);

    const stat = fs.statSync(destination);
    assets.push({
      name: outputName,
      source: relative.split(path.sep).join('/'),
      size: stat.size,
      sha256: await sha256(destination),
    });
  }

  const checksumLines = assets.map((asset) => `${asset.sha256}  ${asset.name}`);
  fs.writeFileSync(
    path.join(output, 'SHA256SUMS.txt'),
    `${checksumLines.join('\n')}\n`,
    'utf8',
  );

  fs.writeFileSync(
    path.join(output, 'RELEASE-MANIFEST.json'),
    `${JSON.stringify({
      product,
      version: args.version ?? identity.version,
      generatedAt: new Date().toISOString(),
      assets,
    }, null, 2)}\n`,
    'utf8',
  );

  console.log(`Artefatos coletados: ${assets.length}`);
  for (const asset of assets) {
    console.log(`${asset.sha256}  ${asset.name}  ${asset.size} bytes`);
  }
}

try {
  await main();
} catch (error) {
  console.error(`Falha ao coletar artefatos: ${error.message}`);
  process.exit(1);
}
