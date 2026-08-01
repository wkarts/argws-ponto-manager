#!/usr/bin/env python3
"""Valida estrutura, configuração, licenciamento e gates de um projeto Tauri/Rust."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path


REQUIRED_FILES = (
    "package.json",
    "package-lock.json",
    "src/config/projectConfig.ts",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "public/manifest.webmanifest",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    ".github/workflows/cloudpanel-linux-release.yml",
    "scripts/release/collect-release-assets.mjs",
    "scripts/release/upload-release-assets.mjs",
    "scripts/windows/build-windows.ps1",
    "deploy/cloudpanel/.env.public.example",
    "deploy/cloudpanel/ports.sh",
)

REQUIRED_NPM_SCRIPTS = (
    "ci:version",
    "typecheck",
    "build:web",
    "fmt:rust:check",
    "lint:rust",
    "test:rust",
    "validate:web",
    "validate:rust",
    "validate:all",
    "release:ci",
    "release:dry",
    "release:collect",
    "release:upload",
    "build:windows:short-target",
    "build:windows:project-target",
    "build:windows:ci-fast",
)

REQUIRED_CI_MARKERS = (
    "pull_request:",
    "workflow_dispatch:",
    "actionlint",
    "npm ci",
    "npm run ci:version",
    "npm run typecheck",
    "npm run build:web",
    "cargo fmt",
    "cargo check",
    "cargo clippy",
    "cargo test",
    "--locked",
    "Validate PowerShell syntax",
    "deploy/cloudpanel/ports.sh",
)

REQUIRED_RELEASE_MARKERS = (
    "workflow_run:",
    "workflow_dispatch:",
    "contents: write",
    "cancel-in-progress: false",
    "tauri-apps/tauri-action@v1",
    "actions/upload-artifact@v7",
    "actions/download-artifact@v8",
    "x86_64-pc-windows-msvc",
    "i686-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "collect-release-assets.mjs",
    "upload-release-assets.mjs",
    "CARGO_TARGET_DIR",
    "runner.temp }}/tauri-target",
    "SHA256SUMS.txt",
    "--draft=false",
)

TEXT_SUFFIXES = {
    ".css",
    ".html",
    ".js",
    ".json",
    ".jsx",
    ".md",
    ".mjs",
    ".rs",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".vue",
    ".yaml",
    ".yml",
}

SKIP_DIRECTORIES = {
    ".git",
    "dist",
    "node_modules",
    "release",
    "target",
}

CONFLICT_PATTERN = re.compile(
    r"^\s*(?:<<<<<<<\s+\S.*|=======|>>>>>>>\s+\S.*)\s*$"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Valida projeto Tauri 2 + Rust + Vue/TypeScript e seus workflows."
    )
    parser.add_argument("--project", required=True, type=Path)
    parser.add_argument(
        "--mode",
        choices=("static", "local", "ci"),
        default="static",
        help="static não executa builds; local/ci executam a matriz completa.",
    )
    parser.add_argument(
        "--allow-licensing",
        action="store_true",
        help="Aceita licenciamento ativo somente quando houve solicitação explícita.",
    )
    parser.add_argument(
        "--skip-npm-ci",
        action="store_true",
        help="Não reinstala node_modules nos modos local/ci.",
    )
    return parser.parse_args()


def read_json(path: Path, findings: list[str]) -> dict | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        findings.append(f"JSON inválido em {path}: {exc}")
        return None
    if not isinstance(value, dict):
        findings.append(f"JSON raiz deve ser objeto em {path}")
        return None
    return value


def read_toml(path: Path, findings: list[str]) -> dict | None:
    try:
        with path.open("rb") as stream:
            value = tomllib.load(stream)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        findings.append(f"TOML inválido em {path}: {exc}")
        return None
    return value


def iter_text_files(project: Path):
    for path in project.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(project)
        if any(part in SKIP_DIRECTORIES for part in relative.parts):
            continue
        if path.suffix.lower() in TEXT_SUFFIXES or path.name in {
            ".env",
            ".env.example",
            "VERSION",
        }:
            yield path


def scan_conflicts(project: Path, findings: list[str]) -> None:
    for path in iter_text_files(project):
        try:
            content = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for line_number, line in enumerate(content.splitlines(), 1):
            if CONFLICT_PATTERN.fullmatch(line):
                findings.append(
                    f"marcador de conflito em {path.relative_to(project)}:{line_number}"
                )


def extract_boolean(content: str, key: str) -> bool | None:
    match = re.search(rf"(?m)^\s*{re.escape(key)}\s*:\s*(true|false)\s*,?", content)
    if not match:
        return None
    return match.group(1) == "true"


def validate_licensing(
    project: Path, findings: list[str], *, allow_licensing: bool
) -> None:
    checks = (
        ("src/config/projectConfig.ts", "licensing", False),
        ("src/config/projectConfig.ts", "licensingStatus", False),
        ("src-tauri/src/core/mod.rs", "licensing", False),
    )
    values: list[tuple[str, bool]] = []
    for relative, key, _expected in checks:
        path = project / relative
        if not path.is_file():
            continue
        value = extract_boolean(path.read_text(encoding="utf-8"), key)
        if value is None:
            findings.append(f"flag {key!r} não encontrada em {relative}")
        else:
            values.append((f"{relative}:{key}", value))

    routes_path = project / "src-tauri" / "src" / "internal_api" / "routes.rs"
    if routes_path.is_file():
        routes = routes_path.read_text(encoding="utf-8")
        match = re.search(r'"licensing"\s*:\s*(true|false)', routes)
        if match:
            values.append(
                ("src-tauri/src/internal_api/routes.rs:licensing", match.group(1) == "true")
            )
        else:
            findings.append("metadado licensing não encontrado na API interna")

    commands_path = project / "src-tauri" / "src" / "commands" / "licensing.rs"
    if commands_path.is_file():
        commands = commands_path.read_text(encoding="utf-8")
        feature_match = re.search(
            r"const LICENSING_FEATURE_ENABLED:\s*bool\s*=\s*(true|false)\s*;",
            commands,
        )
        if feature_match:
            values.append(
                (
                    "src-tauri/src/commands/licensing.rs:LICENSING_FEATURE_ENABLED",
                    feature_match.group(1) == "true",
                )
            )
        else:
            findings.append("gate LICENSING_FEATURE_ENABLED ausente no backend Rust")

        disabled_match = re.search(
            r'"licensing_disabled"\.to_string\(\),\s*Value::from\((true|false)\)',
            commands,
        )
        if not disabled_match:
            findings.append("default licensing_disabled ausente no backend Rust")
        elif not allow_licensing and disabled_match.group(1) != "true":
            findings.append(
                "licenciamento deve iniciar desabilitado: "
                "src-tauri/src/commands/licensing.rs:licensing_disabled"
            )

    if not allow_licensing:
        for label, enabled in values:
            if enabled:
                findings.append(f"licenciamento deve permanecer desligado: {label}")


def validate_versions(
    project: Path,
    package: dict | None,
    cargo: dict | None,
    tauri: dict | None,
    findings: list[str],
) -> None:
    version_path = project / "VERSION"
    version_file = (
        version_path.read_text(encoding="utf-8").strip() if version_path.is_file() else None
    )
    versions = {
        "package.json": package.get("version") if package else None,
        "src-tauri/Cargo.toml": cargo.get("package", {}).get("version") if cargo else None,
        "src-tauri/tauri.conf.json": tauri.get("version") if tauri else None,
        "VERSION": version_file,
    }
    present = {str(value) for value in versions.values() if value is not None}
    if len(present) > 1:
        details = ", ".join(f"{name}={value!r}" for name, value in versions.items())
        findings.append(f"versões divergentes: {details}")
    if not present:
        findings.append("versão do projeto não pôde ser determinada")


def validate_package(package: dict | None, findings: list[str]) -> None:
    if package is None:
        return
    scripts = package.get("scripts")
    if not isinstance(scripts, dict):
        findings.append("package.json não possui objeto scripts")
        return
    for script in REQUIRED_NPM_SCRIPTS:
        if not isinstance(scripts.get(script), str) or not scripts[script].strip():
            findings.append(f"script npm obrigatório ausente: {script}")


def validate_workflows(project: Path, findings: list[str]) -> None:
    workflows_dir = project / ".github" / "workflows"
    if not workflows_dir.is_dir():
        findings.append("diretório .github/workflows ausente")
        return

    workflows = sorted(
        path
        for path in workflows_dir.iterdir()
        if path.is_file() and path.suffix.lower() in {".yml", ".yaml"}
    )
    if not workflows:
        findings.append("nenhum workflow GitHub Actions encontrado")
        return

    for path in workflows:
        content = path.read_text(encoding="utf-8")
        relative = path.relative_to(project)
        if "\t" in content:
            findings.append(f"tabulação inválida em workflow: {relative}")
        for required in ("name:", "on:", "jobs:"):
            if not re.search(rf"(?m)^{re.escape(required)}", content):
                findings.append(f"{relative} não contém chave raiz {required}")

    ci_workflow = project / ".github" / "workflows" / "ci.yml"
    if ci_workflow.is_file():
        content = ci_workflow.read_text(encoding="utf-8")
        for marker in REQUIRED_CI_MARKERS:
            if marker not in content:
                findings.append(f"workflow de CI não contém: {marker}")

    release_workflow = project / ".github" / "workflows" / "release.yml"
    if not release_workflow.is_file():
        return
    content = release_workflow.read_text(encoding="utf-8")
    for marker in REQUIRED_RELEASE_MARKERS:
        if marker not in content:
            findings.append(f"workflow de release não contém: {marker}")


def validate_required_files(project: Path, findings: list[str]) -> None:
    for relative in REQUIRED_FILES:
        path = project / relative
        if not path.is_file():
            findings.append(f"arquivo obrigatório ausente: {relative}")
        elif path.stat().st_size == 0:
            findings.append(f"arquivo obrigatório vazio: {relative}")


def run_command(project: Path, command: list[str]) -> bool:
    print(f"\n$ {' '.join(command)}")
    completed = subprocess.run(command, cwd=project, check=False)
    if completed.returncode != 0:
        print(f"FALHA: comando retornou {completed.returncode}", file=sys.stderr)
        return False
    return True


def run_full_matrix(project: Path, *, skip_npm_ci: bool) -> int:
    required_tools = ("npm", "cargo")
    missing = [tool for tool in required_tools if shutil.which(tool) is None]
    if missing:
        print(f"FALHA: ferramentas ausentes: {', '.join(missing)}", file=sys.stderr)
        return 2

    commands: list[list[str]] = []
    if not skip_npm_ci:
        commands.append(["npm", "ci"])
    commands.extend(
        [
            ["npm", "run", "ci:version"],
            ["npm", "run", "typecheck"],
            ["npm", "run", "build:web"],
            [
                "cargo",
                "generate-lockfile",
                "--manifest-path",
                "src-tauri/Cargo.toml",
            ],
            [
                "cargo",
                "fmt",
                "--manifest-path",
                "src-tauri/Cargo.toml",
                "--all",
                "--check",
            ],
            [
                "cargo",
                "check",
                "--manifest-path",
                "src-tauri/Cargo.toml",
                "--locked",
                "--all-targets",
                "--all-features",
            ],
            [
                "cargo",
                "clippy",
                "--manifest-path",
                "src-tauri/Cargo.toml",
                "--locked",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            [
                "cargo",
                "test",
                "--manifest-path",
                "src-tauri/Cargo.toml",
                "--locked",
                "--all-targets",
                "--all-features",
            ],
        ]
    )
    for command in commands:
        if not run_command(project, command):
            return 1
    return 0


def main() -> int:
    args = parse_args()
    project = args.project.expanduser().resolve()
    if not project.is_dir():
        print(f"ERRO: projeto não encontrado: {project}", file=sys.stderr)
        return 2

    findings: list[str] = []
    validate_required_files(project, findings)
    scan_conflicts(project, findings)

    package = read_json(project / "package.json", findings) if (project / "package.json").is_file() else None
    read_json(project / "package-lock.json", findings) if (project / "package-lock.json").is_file() else None
    tauri = (
        read_json(project / "src-tauri" / "tauri.conf.json", findings)
        if (project / "src-tauri" / "tauri.conf.json").is_file()
        else None
    )
    read_json(project / "public" / "manifest.webmanifest", findings) if (
        project / "public" / "manifest.webmanifest"
    ).is_file() else None
    cargo = (
        read_toml(project / "src-tauri" / "Cargo.toml", findings)
        if (project / "src-tauri" / "Cargo.toml").is_file()
        else None
    )

    validate_package(package, findings)
    validate_versions(project, package, cargo, tauri, findings)
    is_template_source = (project / ".tauri-template-source").is_file()
    if not is_template_source:
        validate_licensing(project, findings, allow_licensing=args.allow_licensing)
    validate_workflows(project, findings)

    if findings:
        print(f"FALHA: {len(findings)} problema(s) estrutural(is) encontrado(s).")
        for finding in findings:
            print(f"- {finding}")
        return 1

    print("OK: estrutura, configurações, licenciamento e workflows validados.")
    if args.mode == "static":
        return 0
    return run_full_matrix(project, skip_npm_ci=args.skip_npm_ci)


if __name__ == "__main__":
    raise SystemExit(main())
