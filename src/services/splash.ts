import { reactive, readonly } from "vue";

export type SplashTone = "success" | "error" | "info";

export interface SplashMessage {
  id: number;
  tone: SplashTone;
  text: string;
}

const splashStateInternal = reactive({
  messages: [] as SplashMessage[],
});

let splashSequence = 0;

function pushSplash(text: string, tone: SplashTone, durationMs = 4200): number {
  const id = ++splashSequence;
  splashStateInternal.messages.push({ id, tone, text });
  window.setTimeout(() => dismissSplash(id), Math.max(1200, durationMs));
  return id;
}

export function dismissSplash(id: number) {
  const index = splashStateInternal.messages.findIndex((item) => item.id === id);
  if (index >= 0) splashStateInternal.messages.splice(index, 1);
}

export function showSplashSuccess(text: string, durationMs?: number): number {
  return pushSplash(text, "success", durationMs);
}

export function showSplashError(text: string, durationMs = 5600): number {
  return pushSplash(text, "error", durationMs);
}

export function showSplashInfo(text: string, durationMs?: number): number {
  return pushSplash(text, "info", durationMs);
}

export const splashState = readonly(splashStateInternal);
