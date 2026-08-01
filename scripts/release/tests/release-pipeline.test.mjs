import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');

function temporaryDirectory(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

test('prepare-release sincroniza e valida o Cargo.lock antes da tag', () => {
  const fixture = temporaryDirectory('ponto-prepare-release-');
  const cargoLog = path.join(fixture, 'cargo.log');
  const fakeCargo = path.join(fixture, 'fake-cargo.sh');
  fs.writeFileSync(fakeCargo, '#!/usr/bin/env bash\nprintf "%s\\n" "$*" >> "${FAKE_CARGO_LOG}"\n', 'utf8');
  fs.chmodSync(fakeCargo, 0o755);

  fs.mkdirSync(path.join(fixture, 'src-tauri'), { recursive: true });
  fs.mkdirSync(path.join(fixture, 'src', 'config'), { recursive: true });
  fs.writeFileSync(path.join(fixture, 'VERSION'), '1.0.0\n');
  writeJson(path.join(fixture, 'package.json'), { name: 'argws-ponto-manager', version: '1.0.0' });
  writeJson(path.join(fixture, 'package-lock.json'), {
    name: 'argws-ponto-manager',
    version: '1.0.0',
    packages: { '': { name: 'argws-ponto-manager', version: '1.0.0' } },
  });
  writeJson(path.join(fixture, 'src-tauri', 'tauri.conf.json'), { productName: 'Ponto Manager', version: '1.0.0' });
  fs.writeFileSync(
    path.join(fixture, 'src-tauri', 'Cargo.toml'),
    '[package]\nname = "argws_ponto_manager"\nversion = "1.0.0"\nedition = "2021"\n',
  );
  fs.writeFileSync(
    path.join(fixture, 'src-tauri', 'Cargo.lock'),
    'version = 4\n\n[[package]]\nname = "argws_ponto_manager"\nversion = "1.0.0"\n',
  );
  fs.writeFileSync(
    path.join(fixture, 'src', 'config', 'projectConfig.ts'),
    'export const projectConfig = { version: "1.0.0" };\n',
  );

  const result = spawnSync(
    process.execPath,
    [path.join(repositoryRoot, 'scripts', 'release', 'prepare-release.mjs'), '2.0.0'],
    {
      cwd: fixture,
      encoding: 'utf8',
      env: { ...process.env, CARGO: fakeCargo, FAKE_CARGO_LOG: cargoLog },
    },
  );

  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(fs.readFileSync(path.join(fixture, 'src-tauri', 'Cargo.lock'), 'utf8'), /version = "2\.0\.0"/);
  assert.equal(JSON.parse(fs.readFileSync(path.join(fixture, 'package.json'))).version, '2.0.0');
  assert.match(fs.readFileSync(cargoLog, 'utf8'), /metadata --manifest-path src-tauri\/Cargo\.toml --locked --format-version 1/);
});

test('coletor publica artefatos aprovados e registra matriz parcial', () => {
  const fixture = temporaryDirectory('ponto-collect-release-');
  const input = path.join(fixture, 'artifacts');
  const output = path.join(fixture, 'release-assets');
  fs.mkdirSync(path.join(input, 'release-desktop-windows-x64'), { recursive: true });
  fs.writeFileSync(path.join(input, 'release-desktop-windows-x64', 'PontoManager.exe'), 'binary');

  writeJson(
    path.join(input, 'release-status-desktop-windows-x64', 'desktop-windows-x64.status.json'),
    {
      id: 'desktop-windows-x64',
      label: 'Windows x64',
      status: 'success',
      artifact: 'release-desktop-windows-x64',
      runUrl: 'https://github.example/actions/runs/1',
    },
  );
  writeJson(
    path.join(input, 'release-status-cloudpanel-linux-x86', 'cloudpanel-linux-x86.status.json'),
    {
      id: 'cloudpanel-linux-x86',
      label: 'CloudPanel Linux x86',
      status: 'failure',
      artifact: 'release-cloudpanel-linux-x86',
      runUrl: 'https://github.example/actions/runs/1',
      error: 'Dependência i386 indisponível.',
    },
  );
  writeJson(path.join(fixture, 'package.json'), { name: 'argws-ponto-manager' });
  fs.writeFileSync(path.join(fixture, 'VERSION'), '1.24.1\n');

  const result = spawnSync(
    process.execPath,
    [
      path.join(repositoryRoot, 'scripts', 'release', 'collect-release-assets.mjs'),
      '--input', input,
      '--output', output,
      '--version', '1.24.1',
    ],
    { cwd: fixture, encoding: 'utf8' },
  );

  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const manifest = JSON.parse(fs.readFileSync(path.join(output, 'RELEASE-MANIFEST.json')));
  assert.equal(manifest.assets.length, 1);
  assert.equal(manifest.buildMatrix.outcome, 'partial');
  assert.equal(manifest.buildMatrix.succeeded, 1);
  assert.equal(manifest.buildMatrix.failed, 8);
  const report = fs.readFileSync(path.join(output, 'RELEASE-STATUS.md'), 'utf8');
  assert.match(report, /Release parcial/);
  assert.match(report, /CloudPanel Linux x86/);
  assert.match(report, /Dependência i386 indisponível/);
});

test('gravador de status preserva falha e URL da execução', () => {
  const fixture = temporaryDirectory('ponto-build-status-');
  const output = path.join(fixture, 'cloudpanel-linux-x86.status.json');
  const result = spawnSync(
    process.execPath,
    [
      path.join(repositoryRoot, 'scripts', 'release', 'write-build-status.mjs'),
      '--output', output,
      '--id', 'cloudpanel-linux-x86',
      '--label', 'CloudPanel Linux x86',
      '--platform', 'cloudpanel-linux',
      '--arch', 'x86',
      '--target', 'i686-unknown-linux-gnu',
      '--artifact', 'release-cloudpanel-linux-x86',
    ],
    {
      encoding: 'utf8',
      env: {
        ...process.env,
        BUILD_STATUS: 'failure',
        RELEASE_RUN_URL: 'https://github.example/actions/runs/2',
      },
    },
  );

  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const status = JSON.parse(fs.readFileSync(output));
  assert.equal(status.success, false);
  assert.equal(status.status, 'failure');
  assert.match(status.error, /actions\/runs\/2/);
});

test('CloudPanel isola Tauri/GTK e compila x64 e x86 no CI', () => {
  const buildScript = fs.readFileSync(
    path.join(repositoryRoot, 'scripts', 'linux', 'build-cloudpanel-release.sh'),
    'utf8',
  );
  const dependencyScript = fs.readFileSync(
    path.join(repositoryRoot, 'scripts', 'linux', 'install-cloudpanel-build-deps.sh'),
    'utf8',
  );
  const cargoManifest = fs.readFileSync(
    path.join(repositoryRoot, 'src-tauri', 'Cargo.toml'),
    'utf8',
  );
  const buildSource = fs.readFileSync(
    path.join(repositoryRoot, 'src-tauri', 'build.rs'),
    'utf8',
  );
  const librarySource = fs.readFileSync(
    path.join(repositoryRoot, 'src-tauri', 'src', 'lib.rs'),
    'utf8',
  );
  const authCommands = fs.readFileSync(
    path.join(repositoryRoot, 'src-tauri', 'src', 'commands', 'auth.rs'),
    'utf8',
  );
  const entityCommands = fs.readFileSync(
    path.join(repositoryRoot, 'src-tauri', 'src', 'commands', 'entities.rs'),
    'utf8',
  );
  const ciWorkflow = fs.readFileSync(
    path.join(repositoryRoot, '.github', 'workflows', 'ci.yml'),
    'utf8',
  );

  assert.match(cargoManifest, /desktop = \["dep:tauri", "dep:tauri-build"/);
  assert.match(cargoManifest, /tauri = \{[^}]*optional = true[^}]*\}/);
  assert.match(cargoManifest, /tauri-build = \{[^}]*optional = true[^}]*\}/);
  assert.match(buildSource, /#\[cfg\(feature = "desktop"\)\]\n    tauri_build::build\(\);/);
  assert.match(buildScript, /--no-default-features/);
  assert.match(buildScript, /cargo tree/);
  assert.match(buildScript, /--edges normal,build/);
  assert.match(buildScript, /forbidden_headless_crates/);
  assert.match(buildScript, /tauri-build/);
  assert.match(buildScript, /tauri-runtime-wry/);
  assert.match(buildScript, /pkg-config --exists openssl/);
  assert.match(librarySource, /#\[cfg\(feature = "desktop"\)\]\n    pub mod access;/);
  assert.doesNotMatch(librarySource, /#\[cfg\(feature = "desktop"\)\]\n    pub mod auth;/);
  assert.doesNotMatch(librarySource, /#\[cfg\(feature = "desktop"\)\]\n    pub mod entities;/);
  assert.match(authCommands, /#\[cfg\(feature = "desktop"\)\]\n#\[tauri::command\]\npub fn auth_login/);
  assert.match(entityCommands, /#\[cfg\(feature = "desktop"\)\]\n#\[tauri::command\]\npub fn entity_list/);
  assert.match(ciWorkflow, /cloudpanel-headless:/);
  assert.match(ciWorkflow, /Build and package CloudPanel headless/);
  assert.match(ciWorkflow, /BUILD_WEB: "false"/);
  assert.doesNotMatch(dependencyScript, /libayatana-appindicator3-dev:i386/);
  assert.doesNotMatch(dependencyScript, /libxdo-dev:i386/);
  assert.doesNotMatch(dependencyScript, /libwebkit2gtk-4\.1-dev:i386/);
});
