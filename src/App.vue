<script setup lang="ts">
import { onErrorCaptured, ref } from "vue";
import { logAppError } from "./services/logger";
import AppSplash from "./components/AppSplash.vue";

const fatalError = ref("");

function resolveComponentName(instance: unknown): string | null {
  const raw = instance as { type?: { name?: string }; $options?: { name?: string } } | null;
  return raw?.type?.name ?? raw?.$options?.name ?? null;
}

onErrorCaptured((error, instance, info) => {
  fatalError.value = error instanceof Error ? error.message : "Falha inesperada na interface.";
  logAppError("vue", "Erro capturado no componente raiz.", {
    info,
    component: resolveComponentName(instance),
    error: fatalError.value,
  });
  return false;
});
</script>

<template>
  <AppSplash />
  <div v-if="fatalError" class="fatal-screen">
    <div class="fatal-card">
      <h2>Erro ao renderizar a aplicação</h2>
      <p>{{ fatalError }}</p>
      <p class="muted-text">Abra a página de logs após reiniciar o sistema para obter mais detalhes.</p>
    </div>
  </div>
  <router-view v-else />
</template>
