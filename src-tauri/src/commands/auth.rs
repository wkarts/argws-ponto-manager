use chrono::{Duration, Utc};
use rusqlite::{params, OptionalExtension};
use tauri::State;
use uuid::Uuid;

use crate::{
    app_state::SharedState,
    db::open_connection,
    models::{AuthUser, LoginResponse, SessionIdentity},
    security::{hash_password, verify_password},
};

pub(crate) const PERMISSION_CATALOG: [(&str, &str); 29] = [
    ("dashboard:view", "Visualizar dashboard"),
    ("empresas:view", "Visualizar empresas usuárias"),
    ("empresas:manage", "Gerenciar empresas usuárias"),
    ("funcionarios:view", "Visualizar funcionários"),
    ("funcionarios:manage", "Gerenciar funcionários"),
    ("horarios:view", "Visualizar horários"),
    ("horarios:manage", "Gerenciar horários"),
    ("escalas:view", "Visualizar escalas"),
    ("escalas:manage", "Gerenciar escalas"),
    ("jornadas:view", "Visualizar jornadas"),
    ("jornadas:manage", "Gerenciar jornadas"),
    ("equipamentos:view", "Visualizar equipamentos"),
    ("equipamentos:manage", "Gerenciar equipamentos"),
    ("eventos:view", "Visualizar eventos"),
    ("eventos:manage", "Gerenciar eventos"),
    ("justificativas:view", "Visualizar justificativas"),
    ("justificativas:manage", "Gerenciar justificativas"),
    ("batidas:view", "Visualizar batidas"),
    ("batidas:manage", "Gerenciar batidas"),
    ("tratamentos:view", "Visualizar tratamento de ponto"),
    ("tratamentos:manage", "Gerenciar tratamento de ponto"),
    ("afd:import", "Importar AFD"),
    ("apuracao:view", "Visualizar apuração"),
    ("apuracao:process", "Processar apuração"),
    ("banco_horas:view", "Visualizar banco de horas"),
    ("banco_horas:manage", "Gerenciar banco de horas"),
    ("fechamentos:view", "Visualizar fechamentos"),
    ("fechamentos:manage", "Gerenciar fechamentos"),
    ("sync:view", "Visualizar fila de sincronização"),
];

pub(crate) fn all_permission_keys() -> Vec<String> {
    let mut keys = PERMISSION_CATALOG
        .iter()
        .map(|(key, _)| (*key).to_string())
        .collect::<Vec<_>>();
    keys.extend([
        "usuarios:view".to_string(),
        "usuarios:manage".to_string(),
        "perfis:view".to_string(),
        "perfis:manage".to_string(),
        "config:view".to_string(),
        "config:manage".to_string(),
        "relatorios:export".to_string(),
    ]);
    keys.sort();
    keys.dedup();
    keys
}

pub(crate) fn build_auth_user(
    conn: &rusqlite::Connection,
    user_id: i64,
) -> Result<AuthUser, String> {
    let base = conn
        .query_row(
            "SELECT id, nome, login, email, telefone, cargo, administrador, master_user, senha_provisoria, ativo
             FROM usuarios
             WHERE id = ?1
             LIMIT 1",
            [user_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("Falha ao carregar usuário autenticado: {err}"))?
        .ok_or_else(|| "Usuário da sessão não encontrado.".to_string())?;

    if base.9 == 0 {
        return Err("Usuário inativo.".to_string());
    }

    let mut stmt_profiles = conn
        .prepare(
            "SELECT p.nome
             FROM usuarios_perfis up
             INNER JOIN perfis_acesso p ON p.id = up.perfil_id
             WHERE up.usuario_id = ?1
             ORDER BY p.nome ASC",
        )
        .map_err(|err| format!("Falha ao preparar perfis do usuário: {err}"))?;

    let profile_names = stmt_profiles
        .query_map([user_id], |row| row.get::<_, String>(0))
        .map_err(|err| format!("Falha ao consultar perfis do usuário: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear perfis do usuário: {err}"))?;

    let mut stmt_companies = conn
        .prepare(
            "SELECT e.id, e.nome
             FROM usuarios_empresas ue
             INNER JOIN empresas e ON e.id = ue.empresa_id
             WHERE ue.usuario_id = ?1
             ORDER BY e.nome ASC",
        )
        .map_err(|err| format!("Falha ao preparar empresas do usuário: {err}"))?;

    let company_rows = stmt_companies
        .query_map([user_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|err| format!("Falha ao consultar empresas do usuário: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear empresas do usuário: {err}"))?;

    let company_ids = company_rows.iter().map(|item| item.0).collect::<Vec<_>>();
    let company_names = company_rows
        .into_iter()
        .map(|item| item.1)
        .collect::<Vec<_>>();

    let permission_keys = if base.7 == 1 {
        all_permission_keys()
    } else {
        let mut stmt_permissions = conn
            .prepare(
                "SELECT DISTINCT pp.permissao_chave
                 FROM usuarios_perfis up
                 INNER JOIN perfis_permissoes pp ON pp.perfil_id = up.perfil_id
                 WHERE up.usuario_id = ?1
                 ORDER BY pp.permissao_chave ASC",
            )
            .map_err(|err| format!("Falha ao preparar permissões do usuário: {err}"))?;

        stmt_permissions
            .query_map([user_id], |row| row.get::<_, String>(0))
            .map_err(|err| format!("Falha ao consultar permissões do usuário: {err}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("Falha ao mapear permissões do usuário: {err}"))?
    };

    Ok(AuthUser {
        id: base.0,
        nome: base.1,
        login: base.2,
        email: base.3,
        telefone: base.4,
        cargo: base.5,
        administrador: base.6 == 1,
        master_user: base.7 == 1,
        senha_provisoria: base.8 == 1,
        permission_keys,
        profile_names,
        company_ids,
        company_names,
    })
}

pub(crate) fn require_session_by_token(
    conn: &rusqlite::Connection,
    session_token: &str,
) -> Result<SessionIdentity, String> {
    let token = session_token.trim();
    if token.is_empty() {
        return Err("Sessão inválida ou expirada.".to_string());
    }

    let now = Utc::now().to_rfc3339();

    let session = conn
        .query_row(
            "SELECT us.usuario_id, u.master_user, u.administrador, u.ativo
             FROM user_sessions us
             INNER JOIN usuarios u ON u.id = us.usuario_id
             WHERE us.session_token = ?1 AND us.expires_at > ?2
             LIMIT 1",
            params![token, now],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("Falha ao validar sessão: {err}"))?
        .ok_or_else(|| "Sessão inválida ou expirada.".to_string())?;

    if session.3 == 0 {
        return Err("Usuário inativo.".to_string());
    }

    conn.execute(
        "UPDATE user_sessions SET last_activity_at = ?1 WHERE session_token = ?2",
        params![now, token],
    )
    .map_err(|err| format!("Falha ao atualizar atividade da sessão: {err}"))?;

    Ok(SessionIdentity {
        user_id: session.0,
        master_user: session.1 == 1,
        administrador: session.2 == 1,
    })
}

#[tauri::command]
pub fn auth_login(
    state: State<'_, SharedState>,
    login: String,
    senha: String,
) -> Result<LoginResponse, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let normalized_login = login.trim().to_lowercase();

    let row = conn
        .query_row(
            "SELECT id, nome, login, senha_hash, administrador, ativo
             FROM usuarios
             WHERE LOWER(login) = ?1
             LIMIT 1",
            [normalized_login],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar usuário: {err}"))?;

    let Some(row) = row else {
        return Ok(LoginResponse {
            success: false,
            message: "Usuário não encontrado.".to_string(),
            session_token: None,
            user: None,
        });
    };

    if row.5 == 0 {
        return Ok(LoginResponse {
            success: false,
            message: "Usuário inativo.".to_string(),
            session_token: None,
            user: None,
        });
    }

    let password_ok = verify_password(&senha, &row.3)?;
    if !password_ok {
        return Ok(LoginResponse {
            success: false,
            message: "Senha inválida.".to_string(),
            session_token: None,
            user: None,
        });
    }

    let now = Utc::now();
    let session_token = Uuid::new_v4().to_string();
    let expires_at = (now + Duration::days(7)).to_rfc3339();
    let now_str = now.to_rfc3339();

    conn.execute(
        "DELETE FROM user_sessions WHERE usuario_id = ?1 OR expires_at <= ?2",
        params![row.0, now_str],
    )
    .map_err(|err| format!("Falha ao limpar sessões antigas: {err}"))?;

    conn.execute(
        "INSERT INTO user_sessions (usuario_id, session_token, created_at, expires_at, last_activity_at)
         VALUES (?1, ?2, ?3, ?4, ?3)",
        params![row.0, session_token, now_str, expires_at],
    )
    .map_err(|err| format!("Falha ao criar sessão: {err}"))?;

    conn.execute(
        "UPDATE usuarios SET ultimo_login_em = ?1 WHERE id = ?2",
        params![Utc::now().to_rfc3339(), row.0],
    )
    .map_err(|err| format!("Falha ao atualizar último login: {err}"))?;

    let user = build_auth_user(&conn, row.0)?;

    Ok(LoginResponse {
        success: true,
        message: if user.senha_provisoria {
            "Login efetuado com sucesso. Este usuário está com senha provisória.".to_string()
        } else {
            "Login efetuado com sucesso.".to_string()
        },
        session_token: Some(session_token),
        user: Some(user),
    })
}

#[tauri::command]
pub fn auth_restore(
    state: State<'_, SharedState>,
    session_token: String,
) -> Result<LoginResponse, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let identity = match require_session_by_token(&conn, &session_token) {
        Ok(identity) => identity,
        Err(_) => {
            return Ok(LoginResponse {
                success: false,
                message: "Sessão inválida ou expirada.".to_string(),
                session_token: None,
                user: None,
            })
        }
    };

    let user = build_auth_user(&conn, identity.user_id)?;
    Ok(LoginResponse {
        success: true,
        message: "Sessão restaurada com sucesso.".to_string(),
        session_token: Some(session_token),
        user: Some(user),
    })
}

#[tauri::command]
pub fn auth_logout(state: State<'_, SharedState>, session_token: String) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    conn.execute(
        "DELETE FROM user_sessions WHERE session_token = ?1",
        [session_token],
    )
    .map_err(|err| format!("Falha ao encerrar sessão: {err}"))?;
    Ok(true)
}

#[tauri::command]
pub fn auth_change_password(
    state: State<'_, SharedState>,
    session_token: String,
    current_password: String,
    new_password: String,
) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let identity = require_session_by_token(&conn, &session_token)?;

    if new_password.trim().len() < 6 {
        return Err("A nova senha deve conter ao menos 6 caracteres.".to_string());
    }

    let current_hash: String = conn
        .query_row(
            "SELECT senha_hash FROM usuarios WHERE id = ?1 LIMIT 1",
            [identity.user_id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao consultar senha atual: {err}"))?;

    if !verify_password(current_password.trim(), &current_hash)? {
        return Err("A senha atual informada está incorreta.".to_string());
    }

    let new_hash = hash_password(new_password.trim())?;
    conn.execute(
        "UPDATE usuarios SET senha_hash = ?1, senha_provisoria = 0, updated_at = ?2 WHERE id = ?3",
        params![new_hash, Utc::now().to_rfc3339(), identity.user_id],
    )
    .map_err(|err| format!("Falha ao atualizar senha: {err}"))?;

    Ok(true)
}
