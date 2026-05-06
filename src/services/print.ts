import { invokeCommand } from "./tauri";

export interface ExternalPrintOptions {
  fileName: string;
}

export async function printHtmlExternally(html: string, options: ExternalPrintOptions): Promise<string | null> {
  const content = String(html || "").trim();
  if (!content) return null;

  const response = await invokeCommand<{ path?: string }>("app_print_html", {
    payload: {
      html: content,
      fileName: options.fileName,
    },
  });

  return response.path || null;
}
