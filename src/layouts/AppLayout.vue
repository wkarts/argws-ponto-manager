<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { entityConfigs } from "../config/entities";
import { getAppMeta } from "../services/crud";
import { useSessionStore } from "../stores/session";
import logoMark from "../assets/branding/logo-mark.png";
import { logAppError, logAppInfo } from "../services/logger";

const session = useSessionStore();
const router = useRouter();
const route = useRoute();

const sidebarOpen = ref(true);
const meta = reactive({ version: "1.0.0", build_hash: "dev" });
const now = ref(new Date());
let timer: number | undefined;

const MENU_STATE_KEY = "ponto-shell-menu-state";

function readMenuState() {
  if (typeof window === "undefined") return null;
  try {
    return JSON.parse(window.localStorage.getItem(MENU_STATE_KEY) || "null");
  } catch {
    return null;
  }
}

const savedMenuState = readMenuState();

const groupState = reactive({
  inicio: savedMenuState?.inicio ?? true,
  cadastros: savedMenuState?.cadastros ?? false,
  operacao: savedMenuState?.operacao ?? false,
  relatorios: savedMenuState?.relatorios ?? false,
  documentacao: savedMenuState?.documentacao ?? false,
  sistema: savedMenuState?.sistema ?? false,
});

const cadastros = computed(() => [
  session.can("empresas:view") ? { title: "Empresas", route: "/empresas" } : null,
  session.can("funcionarios:view") ? { title: "Funcionários", route: "/funcionarios" } : null,
  session.can("usuarios:view") ? { title: "Usuários", route: "/usuarios" } : null,
  session.can("perfis:view") ? { title: "Perfis de acesso", route: "/perfis" } : null,
  entityConfigs.departamentos,
  entityConfigs.funcoes,
  entityConfigs.centro_custos,
  entityConfigs.horarios,
  entityConfigs.escalas,
  entityConfigs.equipamentos,
  entityConfigs.eventos,
  entityConfigs.justificativas,
  session.can("jornadas:view") ? { title: "Jornadas", route: "/jornadas" } : null,
].filter(Boolean) as { title: string; route: string }[]);

const operacao = computed(() => [
  session.can("batidas:view") ? { title: "Batidas", route: "/batidas" } : null,
  session.can("batidas:view") ? { title: "Cartão de ponto", route: "/cartao-ponto" } : null,
  session.can("tratamentos:view") ? { title: "Tratamento de ponto", route: "/tratamentos" } : null,
  session.can("afd:import") ? { title: "Importação AFD", route: "/afd" } : null,
  session.can("apuracao:view") ? { title: "Apuração", route: "/apuracao" } : null,
  session.can("banco_horas:view") ? { title: "Banco de horas", route: "/banco-horas" } : null,
  session.can("fechamentos:view") ? { title: "Fechamento mensal", route: "/fechamentos" } : null,
  { title: "Tratamento em lote", route: "/batidas-lote" },
  session.can("sync:view") ? { title: "Fila de sincronização", route: "/sync-queue" } : null,
].filter(Boolean) as { title: string; route: string }[]);

const relatorios = computed(() => [
  { title: "Central de relatórios", route: "/relatorios" },
  { title: "Exportação REP", route: "/rep" },
]);

const documentacao = computed(() => [
  { title: "Guia do usuário", route: "/documentacao/guia" },
  session.can("config:view") ? { title: "Documentação técnica", route: "/documentacao/tecnica" } : null,
].filter(Boolean) as { title: string; route: string }[]);

const sistema = computed(() => [
  { title: "Sistema e parâmetros", route: "/sistema" },
  { title: "Licenciamento", route: "/licenciamento" },
  { title: "Sobre", route: "/sobre" },
  { title: "Logs da aplicação", route: "/logs" },
]);

const pageTitle = computed(() => {
  const path = route.path;
  const all = [...cadastros.value, ...operacao.value, ...relatorios.value, ...documentacao.value, ...sistema.value, { title: 'Dashboard', route: '/' }];
  return all.find((item) => item.route === path)?.title || "Ponto Manager";
});

const dateLabel = computed(() => new Intl.DateTimeFormat("pt-BR", { dateStyle: "full" }).format(now.value));
const timeLabel = computed(() => new Intl.DateTimeFormat("pt-BR", { timeStyle: "medium" }).format(now.value));

function toggleGroup(key: keyof typeof groupState) {
  groupState[key] = !groupState[key];
}


async function logout() {
  await session.logout();
  router.push("/login");
}

function closeSidebarOnMobile() {
  if (window.innerWidth <= 1100) {
    sidebarOpen.value = false;
  }
}

watch(() => route.fullPath, closeSidebarOnMobile);
watch(groupState, (value) => {
  if (typeof window !== "undefined") {
    window.localStorage.setItem(MENU_STATE_KEY, JSON.stringify(value));
  }
}, { deep: true });

onMounted(async () => {
  try {
    const payload = await getAppMeta();
    meta.version = String(payload.version || meta.version);
    meta.build_hash = String(payload.build_hash || meta.build_hash);
  } catch (error) {
    logAppError("layout", "Falha ao carregar metadados da aplicação.", {
      error: error instanceof Error ? error.message : String(error),
    });
  }
  timer = window.setInterval(() => {
    now.value = new Date();
  }, 1000);
  closeSidebarOnMobile();
  logAppInfo("layout", "Shell principal montado.");
});

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <div class="shell-root" :class="{ 'sidebar-collapsed': !sidebarOpen }">
    <div v-if="sidebarOpen" class="shell-overlay" @click="sidebarOpen = false" />

    <aside class="sidebar" :class="{ open: sidebarOpen }">
      <div class="brand-box">
        <img :src="logoMark" alt="Ponto Manager" class="brand-mark" />
        <div class="brand-copy">
          <h1>Ponto Manager</h1>
          <div class="muted-light">jornada • rep • banco de horas</div>
        </div>
      </div>

      <div class="sidebar-scroll">
        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('inicio')">Início</button>
          <div v-show="groupState.inicio" class="menu-links">
            <RouterLink v-if="session.can('dashboard:view')" to="/">Dashboard</RouterLink>
          </div>
        </div>

        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('cadastros')">Cadastros</button>
          <div v-show="groupState.cadastros" class="menu-links submenu-links">
            <RouterLink v-for="item in cadastros" :key="item.route" :to="item.route">{{ item.title }}</RouterLink>
          </div>
        </div>

        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('operacao')">Operação</button>
          <div v-show="groupState.operacao" class="menu-links submenu-links">
            <RouterLink v-for="item in operacao" :key="item.route" :to="item.route">{{ item.title }}</RouterLink>
          </div>
        </div>

        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('relatorios')">Relatórios e integração</button>
          <div v-show="groupState.relatorios" class="menu-links submenu-links">
            <RouterLink v-for="item in relatorios" :key="item.route" :to="item.route">{{ item.title }}</RouterLink>
          </div>
        </div>

        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('documentacao')">Documentação</button>
          <div v-show="groupState.documentacao" class="menu-links submenu-links">
            <RouterLink v-for="item in documentacao" :key="item.route" :to="item.route">{{ item.title }}</RouterLink>
          </div>
        </div>


        <div class="menu-group">
          <button class="menu-group-button" @click="toggleGroup('sistema')">Sistema</button>
          <div v-show="groupState.sistema" class="menu-links submenu-links">
            <RouterLink v-for="item in sistema" :key="item.route" :to="item.route">{{ item.title }}</RouterLink>
          </div>
        </div>
      </div>

      <div class="sidebar-footer-fixed">
        <div class="clock-box">
          <div class="clock-time">{{ timeLabel }}</div>
          <div class="clock-date">{{ dateLabel }}</div>
        </div>
        <div class="build-box">
          <div><strong>Versão</strong> {{ meta.version }}</div>
          <div><strong>Build</strong> {{ meta.build_hash }}</div>
        </div>
        <div class="user-box">
          <strong>{{ session.user?.nome || "Sem usuário" }}</strong>
          <span>{{ session.user?.login || "-" }}</span>
        </div>
        <button class="secondary" @click="logout">Sair</button>
      </div>
    </aside>

    <div class="shell-main">
      <header class="topbar">
        <div class="topbar-left">
          <button class="icon-button" @click="sidebarOpen = !sidebarOpen">☰</button>
          <div>
            <div class="page-title">{{ pageTitle }}</div>
            <div class="page-subtitle">{{ session.activeCompanyName }}</div>
          </div>
        </div>
        <div class="topbar-right">
          <label class="company-selector">
            <span>Empresa ativa</span>
            <select :value="session.activeCompanyId ?? ''" @change="session.setActiveCompany(Number(($event.target as HTMLSelectElement).value) || null)">
              <option v-if="session.isMaster" value="">Todas / geral</option>
              <option v-for="(companyName, index) in session.user?.company_names || []" :key="companyName" :value="session.user?.company_ids[index]">
                {{ companyName }}
              </option>
            </select>
          </label>
        </div>
      </header>

      <main class="content-shell">
        <div class="page-content-scroll">
          <RouterView />
        </div>
      </main>
    </div>
  </div>
</template>
