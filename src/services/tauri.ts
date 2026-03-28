import { invoke } from "@tauri-apps/api/core";

function toCamelCase(key: string): string {
  return key.replace(/_([a-z])/g, (_, char: string) => char.toUpperCase());
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return Object.prototype.toString.call(value) === "[object Object]";
}

function withTauriArgAliases<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((item) => withTauriArgAliases(item)) as T;
  }

  if (!isPlainObject(value)) {
    return value;
  }

  const result: Record<string, unknown> = {};
  for (const [key, raw] of Object.entries(value)) {
    const normalized = withTauriArgAliases(raw);
    result[key] = normalized;

    const camelKey = toCamelCase(key);
    if (camelKey !== key && !(camelKey in result)) {
      result[camelKey] = normalized;
    }
  }

  return result as T;
}

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const normalizedArgs = withTauriArgAliases(args);
  try {
    return await invoke<T>(command, normalizedArgs);
  } catch (error) {
    if (command !== "app_log_write") {
      void import("./logger").then(({ logAppError }) => {
        logAppError("invoke", `Falha ao executar comando ${command}.`, {
          command,
          args: normalizedArgs,
          error: error instanceof Error ? error.message : String(error),
        });
      });
    }
    throw error;
  }
}
