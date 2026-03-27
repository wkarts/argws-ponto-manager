import { invoke } from "@tauri-apps/api/core";

export type AppLogLevel = "debug" | "info" | "warning" | "error";

export interface AppLogPayload {
  level?: AppLogLevel;
  category: string;
  message: string;
  source?: string;
  route?: string;
  details?: unknown;
}

export async function writeAppLog(payload: AppLogPayload): Promise<void> {
  const normalized = {
    level: payload.level ?? "info",
    category: payload.category,
    message: payload.message,
    source: payload.source ?? "frontend",
    route: payload.route ?? window.location.hash,
    details: payload.details ?? null,
  };
  try {
    await invoke<boolean>("app_log_write", { payload: normalized });
  } catch (error) {
    console.error("Falha ao gravar log da aplicação", normalized, error);
  }
}

export function logAppError(category: string, message: string, details?: unknown) {
  void writeAppLog({ level: "error", category, message, details, source: "frontend" });
}

export function logAppInfo(category: string, message: string, details?: unknown) {
  void writeAppLog({ level: "info", category, message, details, source: "frontend" });
}

export function logAppWarning(category: string, message: string, details?: unknown) {
  void writeAppLog({ level: "warning", category, message, details, source: "frontend" });
}
