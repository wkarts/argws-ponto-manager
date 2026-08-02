use chrono::Utc;
use rusqlite::{params, OptionalExtension};

pub const STATUS_ATIVA: &str = "ativa";
pub const STATUS_DUPLICIDADE: &str = "duplicidade";

#[derive(Debug, Clone)]
pub struct PunchIntegritySnapshot {
    pub funcionario_id: i64,
    pub data_referencia: String,
    pub ativo: bool,
    pub status: String,
}

pub fn origem_oficial(origem: &str) -> bool {
    let normalized = origem.trim().to_lowercase();
    normalized.contains("afd") || normalized.contains("conector") || normalized.contains("rep")
}

pub fn carregar_snapshot(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<PunchIntegritySnapshot>, String> {
    conn.query_row(
        "SELECT funcionario_id, data_referencia, COALESCE(ativo, 1),
                COALESCE(status, 'ativa')
           FROM batidas
          WHERE id = ?1
          LIMIT 1",
        [id],
        |row| {
            Ok(PunchIntegritySnapshot {
                funcionario_id: row.get(0)?,
                data_referencia: row.get(1)?,
                ativo: row.get::<_, i64>(2)? != 0,
                status: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|err| format!("Falha ao carregar integridade da batida: {err}"))
}

pub fn marcar_duplicidade(
    conn: &rusqlite::Connection,
    id: i64,
    batida_principal_id: i64,
    motivo: &str,
) -> Result<bool, String> {
    if id == batida_principal_id {
        return Err("A batida duplicada não pode ser a própria marcação principal.".to_string());
    }

    let target = carregar_snapshot(conn, id)?
        .ok_or_else(|| "Batida selecionada para duplicidade não encontrada.".to_string())?;
    let principal = carregar_snapshot(conn, batida_principal_id)?
        .ok_or_else(|| "Batida principal da duplicidade não encontrada.".to_string())?;

    if target.funcionario_id != principal.funcionario_id
        || target.data_referencia != principal.data_referencia
    {
        return Err(
            "A batida principal deve pertencer ao mesmo funcionário e à mesma data.".to_string(),
        );
    }
    if !principal.ativo {
        return Err("A batida principal precisa estar ativa.".to_string());
    }
    if !target.ativo && target.status == STATUS_DUPLICIDADE {
        return Ok(false);
    }

    let now = Utc::now().to_rfc3339();
    let affected = conn
        .execute(
            "UPDATE batidas
                SET ativo = 0,
                    status = ?1,
                    duplicada_de_id = ?2,
                    inativada_em = ?3,
                    inativada_motivo = ?4,
                    reativada_em = NULL,
                    updated_at = ?3
              WHERE id = ?5",
            params![
                STATUS_DUPLICIDADE,
                batida_principal_id,
                now,
                motivo.trim(),
                id
            ],
        )
        .map_err(|err| format!("Falha ao marcar batida como duplicidade: {err}"))?;

    if affected > 0 {
        conn.execute(
            "UPDATE afd_marcacoes
                SET status = 'duplicidade',
                    mensagem = CASE
                        WHEN COALESCE(mensagem, '') = '' THEN ?2
                        ELSE mensagem || ' ' || ?2
                    END
              WHERE batida_id = ?1",
            params![
                id,
                format!("Batida ocultada como duplicidade da marcação {batida_principal_id}.")
            ],
        )
        .map_err(|err| format!("Falha ao atualizar rastreabilidade AFD da duplicidade: {err}"))?;
    }

    Ok(affected > 0)
}

pub fn reativar_duplicidade(
    conn: &rusqlite::Connection,
    id: i64,
    motivo: &str,
) -> Result<bool, String> {
    let target = carregar_snapshot(conn, id)?
        .ok_or_else(|| "Batida selecionada para reativação não encontrada.".to_string())?;
    if target.ativo {
        return Ok(false);
    }
    if target.status != STATUS_DUPLICIDADE {
        return Err("Somente batidas marcadas como duplicidade podem ser reativadas.".to_string());
    }

    let now = Utc::now().to_rfc3339();
    let affected = conn
        .execute(
            "UPDATE batidas
                SET ativo = 1,
                    status = ?1,
                    duplicada_de_id = NULL,
                    reativada_em = ?2,
                    inativada_motivo = CASE
                        WHEN TRIM(?3) = '' THEN inativada_motivo
                        ELSE COALESCE(inativada_motivo, '') || ' | Reativação: ' || TRIM(?3)
                    END,
                    updated_at = ?2
              WHERE id = ?4",
            params![STATUS_ATIVA, now, motivo, id],
        )
        .map_err(|err| format!("Falha ao reativar batida duplicada: {err}"))?;

    if affected > 0 {
        conn.execute(
            "UPDATE afd_marcacoes
                SET status = 'reativada',
                    mensagem = CASE
                        WHEN COALESCE(mensagem, '') = '' THEN 'Batida reativada pelo usuário.'
                        ELSE mensagem || ' Batida reativada pelo usuário.'
                    END
              WHERE batida_id = ?1",
            [id],
        )
        .map_err(|err| format!("Falha ao atualizar rastreabilidade AFD da reativação: {err}"))?;
    }

    Ok(affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().expect("sqlite em memória");
        conn.execute_batch(
            "CREATE TABLE batidas (
                id INTEGER PRIMARY KEY,
                funcionario_id INTEGER NOT NULL,
                data_referencia TEXT NOT NULL,
                hora TEXT NOT NULL,
                origem TEXT NOT NULL,
                ativo INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'ativa',
                duplicada_de_id INTEGER,
                inativada_em TEXT,
                inativada_motivo TEXT,
                reativada_em TEXT,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE afd_marcacoes (
                id INTEGER PRIMARY KEY,
                batida_id INTEGER,
                status TEXT NOT NULL,
                mensagem TEXT
             );
             INSERT INTO batidas VALUES
                (1, 10, '2026-08-01', '08:00', 'afd_671', 1, 'ativa', NULL, NULL, NULL, NULL, '2026-08-01'),
                (2, 10, '2026-08-01', '08:00', 'conector', 1, 'ativa', NULL, NULL, NULL, NULL, '2026-08-01');
             INSERT INTO afd_marcacoes VALUES (1, 2, 'importada', 'Importada.');",
        )
        .expect("schema de teste");
        conn
    }

    #[test]
    fn identifica_origens_oficiais_sem_proteger_marcacao_manual() {
        assert!(origem_oficial("afd_671"));
        assert!(origem_oficial("conector"));
        assert!(origem_oficial("REP"));
        assert!(!origem_oficial("cartao_inline"));
    }

    #[test]
    fn duplicidade_e_ocultada_sem_exclusao_e_pode_ser_reativada() {
        let conn = database();
        assert!(marcar_duplicidade(&conn, 2, 1, "Duplicidade confirmada").unwrap());
        let hidden = carregar_snapshot(&conn, 2).unwrap().unwrap();
        assert!(!hidden.ativo);
        assert_eq!(hidden.status, STATUS_DUPLICIDADE);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM batidas", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            2
        );

        assert!(reativar_duplicidade(&conn, 2, "Conferência do RH").unwrap());
        let restored = carregar_snapshot(&conn, 2).unwrap().unwrap();
        assert!(restored.ativo);
        assert_eq!(restored.status, STATUS_ATIVA);
    }

    #[test]
    fn rejeita_principal_de_outro_funcionario() {
        let conn = database();
        conn.execute(
            "INSERT INTO batidas VALUES (3, 11, '2026-08-01', '08:00', 'afd_671', 1, 'ativa', NULL, NULL, NULL, NULL, '2026-08-01')",
            [],
        )
        .unwrap();
        assert!(marcar_duplicidade(&conn, 2, 3, "inválida").is_err());
    }
}
