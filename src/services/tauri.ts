import { invoke } from "@tauri-apps/api/core";

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    if (command !== "app_log_write") {
      void import("./logger").then(({ logAppError }) => {
        logAppError("invoke", `Falha ao executar comando ${command}.`, {
          command,
          args,
          error: error instanceof Error ? error.message : String(error),
        });
      });
    }
    throw error;
  }
}
