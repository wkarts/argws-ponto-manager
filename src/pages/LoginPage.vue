<script setup lang="ts">
import { reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { useSessionStore } from "../stores/session";

const router = useRouter();
const session = useSessionStore();

const form = reactive({
  login: "admin",
  senha: "admin123"
});

const error = ref("");
const info = ref("");

async function submit() {
  error.value = "";
  info.value = "";
  try {
    const response = await session.login(form.login, form.senha);
    info.value = response.message;
    await router.push("/");
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Falha ao autenticar.";
  }
}
</script>

<template>
  <div class="login-page">
    <div class="login-box">
      <div class="badge">Pontos Desktop</div>
      <h1>Acesso ao sistema</h1>
      <p class="muted">
        Estrutura com sessão persistente local, usuário master e perfis de acesso.<br />
        <strong>admin</strong> / <strong>admin123</strong>
      </p>

      <form class="grid" @submit.prevent="submit">
        <div class="field">
          <label>Login</label>
          <input v-model="form.login" type="text" autocomplete="username" />
        </div>

        <div class="field">
          <label>Senha</label>
          <input v-model="form.senha" type="password" autocomplete="current-password" />
        </div>

        <div v-if="info" class="alert info">{{ info }}</div>
        <div v-if="error" class="alert error">{{ error }}</div>

        <button class="primary" type="submit" :disabled="session.loading || session.restoring">
          {{ session.loading ? "Entrando..." : session.restoring ? "Restaurando..." : "Entrar" }}
        </button>
      </form>
    </div>
  </div>
</template>
