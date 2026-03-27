import { defineStore } from "pinia";
import { invokeCommand } from "../services/tauri";

export interface AuthUser {
  id: number;
  nome: string;
  login: string;
  email?: string | null;
  telefone?: string | null;
  cargo?: string | null;
  administrador: boolean;
  master_user: boolean;
  senha_provisoria: boolean;
  permission_keys: string[];
  profile_names: string[];
  company_ids: number[];
  company_names: string[];
}

interface LoginResponse {
  success: boolean;
  message: string;
  session_token?: string | null;
  user?: AuthUser;
}

const STORAGE_KEY = "pontos-desktop-session";
const ACTIVE_COMPANY_KEY = "pontos-desktop-active-company";

function readStorage(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(STORAGE_KEY);
}

function writeStorage(value: string | null) {
  if (typeof window === "undefined") return;
  if (!value) {
    window.localStorage.removeItem(STORAGE_KEY);
    return;
  }
  window.localStorage.setItem(STORAGE_KEY, value);
}

function readActiveCompany(): number | null {
  if (typeof window === "undefined") return null;
  const raw = window.localStorage.getItem(ACTIVE_COMPANY_KEY);
  if (!raw) return null;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : null;
}

function writeActiveCompany(value: number | null) {
  if (typeof window === "undefined") return;
  if (value == null) {
    window.localStorage.removeItem(ACTIVE_COMPANY_KEY);
    return;
  }
  window.localStorage.setItem(ACTIVE_COMPANY_KEY, String(value));
}

export const useSessionStore = defineStore("session", {
  state: () => ({
    user: null as AuthUser | null,
    sessionToken: readStorage() as string | null,
    activeCompanyId: readActiveCompany() as number | null,
    loading: false,
    restoring: false,
    initialized: false
  }),
  getters: {
    isAuthenticated: (state) => Boolean(state.user && state.sessionToken),
    isMaster: (state) => Boolean(state.user?.master_user),
    permissionKeys: (state) => state.user?.permission_keys || [],
    activeCompanyName(state): string {
      const user = state.user;
      if (!user || state.activeCompanyId == null) return "Todas as empresas";
      const index = user.company_ids.findIndex((id) => id === state.activeCompanyId);
      return index >= 0 ? user.company_names[index] : "Empresa ativa";
    }
  },
  actions: {
    can(permission: string) {
      if (!permission) return true;
      if (this.user?.master_user) return true;
      return this.permissionKeys.includes(permission);
    },
    ensureActiveCompany() {
      if (!this.user) {
        this.activeCompanyId = null;
        writeActiveCompany(null);
        return;
      }
      const available = this.user.company_ids || [];
      if (!available.length) {
        this.activeCompanyId = null;
        writeActiveCompany(null);
        return;
      }
      if (this.activeCompanyId && available.includes(this.activeCompanyId)) {
        writeActiveCompany(this.activeCompanyId);
        return;
      }
      this.activeCompanyId = available[0];
      writeActiveCompany(this.activeCompanyId);
    },
    setActiveCompany(companyId: number | null) {
      if (!this.user) {
        this.activeCompanyId = null;
        writeActiveCompany(null);
        return;
      }
      if (companyId == null) {
        this.activeCompanyId = null;
        writeActiveCompany(null);
        return;
      }
      if (!this.user.master_user && !this.user.company_ids.includes(companyId)) {
        return;
      }
      this.activeCompanyId = companyId;
      writeActiveCompany(companyId);
    },
    async login(login: string, senha: string) {
      this.loading = true;
      try {
        const response = await invokeCommand<LoginResponse>("auth_login", { login, senha });
        if (!response.success || !response.user || !response.session_token) {
          throw new Error(response.message || "Falha ao autenticar.");
        }
        this.user = response.user;
        this.sessionToken = response.session_token;
        writeStorage(response.session_token);
        this.ensureActiveCompany();
        this.initialized = true;
        return response;
      } finally {
        this.loading = false;
      }
    },
    async restore() {
      if (this.initialized) return;
      this.restoring = true;
      try {
        if (!this.sessionToken) {
          this.user = null;
          return;
        }
        const response = await invokeCommand<LoginResponse>("auth_restore", {
          session_token: this.sessionToken
        });
        if (!response.success || !response.user || !response.session_token) {
          this.user = null;
          this.sessionToken = null;
          writeStorage(null);
          writeActiveCompany(null);
          return;
        }
        this.user = response.user;
        this.sessionToken = response.session_token;
        writeStorage(response.session_token);
        this.ensureActiveCompany();
      } finally {
        this.initialized = true;
        this.restoring = false;
      }
    },
    async logout() {
      const currentToken = this.sessionToken;
      this.user = null;
      this.sessionToken = null;
      this.activeCompanyId = null;
      this.initialized = true;
      writeStorage(null);
      writeActiveCompany(null);
      if (currentToken) {
        try {
          await invokeCommand<boolean>("auth_logout", { session_token: currentToken });
        } catch {
          // noop
        }
      }
    }
  }
});
