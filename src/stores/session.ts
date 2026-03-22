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

export const useSessionStore = defineStore("session", {
  state: () => ({
    user: null as AuthUser | null,
    sessionToken: readStorage() as string | null,
    loading: false,
    restoring: false,
    initialized: false
  }),
  getters: {
    isAuthenticated: (state) => Boolean(state.user && state.sessionToken),
    isMaster: (state) => Boolean(state.user?.master_user),
    permissionKeys: (state) => state.user?.permission_keys || []
  },
  actions: {
    can(permission: string) {
      if (!permission) return true;
      if (this.user?.master_user) return true;
      return this.permissionKeys.includes(permission);
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
          return;
        }
        this.user = response.user;
        this.sessionToken = response.session_token;
        writeStorage(response.session_token);
      } finally {
        this.initialized = true;
        this.restoring = false;
      }
    },
    async logout() {
      const currentToken = this.sessionToken;
      this.user = null;
      this.sessionToken = null;
      this.initialized = true;
      writeStorage(null);
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
