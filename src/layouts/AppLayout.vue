<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, RouterView, useRouter } from "vue-router";
import { entityConfigs } from "../config/entities";
import { useSessionStore } from "../stores/session";
import logoMark from "../assets/branding/logo-mark.png";

const session = useSessionStore();
const router = useRouter();

const cadastros = computed(() => [
  session.can("empresas:view") ? entityConfigs.empresas : null,
  session.can("funcionarios:view") ? entityConfigs.funcionarios : null,
  session.can("usuarios:view") ? { title: "Usuários", route: "/usuarios" } : null,
  session.can("perfis:view") ? { title: "Perfis de acesso", route: "/perfis" } : null,
  entityConfigs.departamentos,
  entityConfigs.funcoes,
  entityConfigs.centro_custos,
  entityConfigs.horarios,
  entityConfigs.escalas,
  entityConfigs.equipamentos,
  entityConfigs.eventos,
  entityConfigs.justificativas
].filter(Boolean));

async function logout() {
  await session.logout();
  router.push("/login");
}
</script>

<template>
  <div class="page-shell">
    <aside class="sidebar">
      <div class="brand-box">
        <img :src="logoMark" alt="Ponto Manager" class="brand-mark" />
        <div class="brand-copy">
          <h1>Ponto Manager</h1>
          <div class="muted-light">controle de jornada • Rust + Tauri + Vue + SQLite</div>
        </div>
      </div>

      <div class="menu-group">
        <div class="menu-group-title">Início</div>
        <RouterLink v-if="session.can('dashboard:view')" to="/">Dashboard</RouterLink>
      </div>

      <div class="menu-group">
        <div class="menu-group-title">Cadastros</div>
        <RouterLink v-for="item in cadastros" :key="item!.route" :to="item!.route">
          {{ item!.title }}
        </RouterLink>
        <RouterLink v-if="session.can('jornadas:view')" to="/jornadas">Jornadas de trabalho</RouterLink>
      </div>

      <div class="menu-group">
        <div class="menu-group-title">Operação</div>
        <RouterLink v-if="session.can('batidas:view')" to="/batidas">Batidas</RouterLink>
        <RouterLink v-if="session.can('tratamentos:view')" to="/tratamentos">Tratamento de ponto</RouterLink>
        <RouterLink v-if="session.can('afd:import')" to="/afd">Importação AFD</RouterLink>
        <RouterLink v-if="session.can('apuracao:view')" to="/apuracao">Apuração</RouterLink>
        <RouterLink v-if="session.can('banco_horas:view')" to="/banco-horas">Banco de horas</RouterLink>
        <RouterLink v-if="session.can('fechamentos:view')" to="/fechamentos">Fechamento mensal</RouterLink>
        <RouterLink v-if="session.can('sync:view')" to="/sync-queue">Fila de sincronização</RouterLink>
      </div>

      <div class="menu-group sidebar-footer">
        <div class="menu-group-title">Sessão</div>
        <div class="user-box">
          <strong>{{ session.user?.nome || "Sem usuário" }}</strong>
          <span>{{ session.user?.login || "-" }}</span>
        </div>
        <div class="pill-box">
          <span class="status-pill" :class="session.user?.master_user ? 'pill-master' : 'pill-user'">
            {{ session.user?.master_user ? 'MASTER' : 'USUÁRIO' }}
          </span>
          <span v-for="perfil in session.user?.profile_names || []" :key="perfil" class="status-pill pill-secondary">{{ perfil }}</span>
        </div>
        <div class="muted-row">Empresas: {{ (session.user?.company_names || []).join(', ') || 'todas / não vinculado' }}</div>
        <button class="secondary" @click="logout">Sair</button>
      </div>
    </aside>

    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
