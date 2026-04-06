use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use std::path::Path;

use crate::{db::open_connection, security::hash_password};
const BOOTSTRAP_SEED_KEY: &str = "bootstrap_seed_version";
const BOOTSTRAP_SEED_STATUS_KEY: &str = "bootstrap_seed_status";
const BOOTSTRAP_SEED_VERSION: i64 = 1;

pub fn migrate(db_path: &Path) -> Result<(), String> {
    let conn = open_connection(db_path)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS empresas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL,
            nome_fantasia TEXT,
            documento TEXT,
            inscricao_estadual TEXT,
            inscricao_municipal TEXT,
            telefone TEXT,
            email TEXT,
            responsavel_nome TEXT,
            responsavel_telefone TEXT,
            cep TEXT,
            endereco TEXT,
            numero TEXT,
            complemento TEXT,
            bairro TEXT,
            cidade TEXT,
            estado TEXT,
            observacoes TEXT,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS usuarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL,
            login TEXT NOT NULL UNIQUE,
            email TEXT,
            telefone TEXT,
            cargo TEXT,
            observacoes TEXT,
            senha_hash TEXT NOT NULL,
            master_user INTEGER NOT NULL DEFAULT 0,
            administrador INTEGER NOT NULL DEFAULT 0,
            senha_provisoria INTEGER NOT NULL DEFAULT 0,
            ultimo_login_em TEXT,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );


        CREATE TABLE IF NOT EXISTS perfis_acesso (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nome TEXT NOT NULL UNIQUE,
            descricao TEXT,
            perfil_master INTEGER NOT NULL DEFAULT 0,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS perfis_permissoes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            perfil_id INTEGER NOT NULL,
            permissao_chave TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (perfil_id) REFERENCES perfis_acesso(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS usuarios_perfis (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            usuario_id INTEGER NOT NULL,
            perfil_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE,
            FOREIGN KEY (perfil_id) REFERENCES perfis_acesso(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS usuarios_empresas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            usuario_id INTEGER NOT NULL,
            empresa_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            usuario_id INTEGER NOT NULL,
            session_token TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS departamentos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS funcoes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS centro_custos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo TEXT,
            descricao TEXT NOT NULL,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS horarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            numero INTEGER,
            descricao TEXT NOT NULL,
            entrada_1 TEXT,
            saida_1 TEXT,
            entrada_2 TEXT,
            saida_2 TEXT,
            carga_horaria_minutos INTEGER NOT NULL DEFAULT 480,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS escalas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            horario_id INTEGER,
            dias_ativos TEXT,
            tolerancia_minutos INTEGER NOT NULL DEFAULT 5,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (horario_id) REFERENCES horarios(id)
        );

        CREATE TABLE IF NOT EXISTS jornadas_trabalho (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            empresa_id INTEGER,
            codigo TEXT,
            descricao TEXT NOT NULL,
            tipo_jornada TEXT NOT NULL DEFAULT 'fixa',
            perfil_flexivel TEXT,
            permite_folga_movel INTEGER NOT NULL DEFAULT 0,
            permite_meia_folga INTEGER NOT NULL DEFAULT 0,
            dia_folga_base INTEGER,
            periodo_meia_folga TEXT,
            heuristica_troca_folga INTEGER NOT NULL DEFAULT 1,
            dias_trabalho_semana INTEGER NOT NULL DEFAULT 6,
            folgas_mensais INTEGER NOT NULL DEFAULT 0,
            sabado_tipo TEXT NOT NULL DEFAULT 'integral',
            suporta_diarista_generico INTEGER NOT NULL DEFAULT 0,
            limite_dias_diarista INTEGER NOT NULL DEFAULT 0,
            semana_alternada_folga INTEGER NOT NULL DEFAULT 0,
            tolerancia_entrada_minutos INTEGER NOT NULL DEFAULT 5,
            tolerancia_saida_minutos INTEGER NOT NULL DEFAULT 5,
            tolerancia_intervalo_minutos INTEGER NOT NULL DEFAULT 5,
            carga_semanal_minutos INTEGER NOT NULL DEFAULT 2640,
            limite_diario_minutos INTEGER NOT NULL DEFAULT 600,
            banco_horas_ativo INTEGER NOT NULL DEFAULT 1,
            exige_marcacao_intervalo INTEGER NOT NULL DEFAULT 1,
            compensa_atraso_com_extra INTEGER NOT NULL DEFAULT 1,
            modo_tratamento_afd TEXT NOT NULL DEFAULT 'auto',
            observacoes TEXT,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id)
        );

        CREATE TABLE IF NOT EXISTS jornada_dias (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            jornada_id INTEGER NOT NULL,
            dia_semana INTEGER NOT NULL,
            entrada_1 TEXT,
            saida_1 TEXT,
            entrada_2 TEXT,
            saida_2 TEXT,
            carga_prevista_minutos INTEGER NOT NULL DEFAULT 0,
            intervalo_minutos INTEGER NOT NULL DEFAULT 0,
            folga INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (jornada_id) REFERENCES jornadas_trabalho(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS equipamentos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            empresa_id INTEGER,
            codigo TEXT,
            descricao TEXT NOT NULL,
            modelo TEXT,
            ip TEXT,
            porta INTEGER,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id)
        );

        CREATE TABLE IF NOT EXISTS eventos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo TEXT,
            descricao TEXT NOT NULL,
            tipo TEXT,
            impacta_banco_horas INTEGER NOT NULL DEFAULT 0,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS justificativas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            abono INTEGER NOT NULL DEFAULT 0,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS funcionarios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            empresa_id INTEGER,
            matricula TEXT,
            nome TEXT NOT NULL,
            nome_social TEXT,
            documento TEXT,
            rg TEXT,
            pis TEXT,
            email TEXT,
            telefone TEXT,
            celular TEXT,
            data_nascimento TEXT,
            data_admissao TEXT,
            data_demissao TEXT,
            ferias_inicio TEXT,
            ferias_fim TEXT,
            ferias_dias INTEGER NOT NULL DEFAULT 0,
            sexo TEXT,
            estado_civil TEXT,
            cep TEXT,
            endereco TEXT,
            numero TEXT,
            complemento TEXT,
            bairro TEXT,
            cidade TEXT,
            estado TEXT,
            departamento_id INTEGER,
            funcao_id INTEGER,
            centro_custo_id INTEGER,
            horario_id INTEGER,
            escala_id INTEGER,
            jornada_id INTEGER,
            observacoes TEXT,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id),
            FOREIGN KEY (departamento_id) REFERENCES departamentos(id),
            FOREIGN KEY (funcao_id) REFERENCES funcoes(id),
            FOREIGN KEY (centro_custo_id) REFERENCES centro_custos(id),
            FOREIGN KEY (horario_id) REFERENCES horarios(id),
            FOREIGN KEY (escala_id) REFERENCES escalas(id),
            FOREIGN KEY (jornada_id) REFERENCES jornadas_trabalho(id)
        );


        CREATE TABLE IF NOT EXISTS feriados (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            data TEXT NOT NULL,
            descricao TEXT NOT NULL,
            contexto_tipo TEXT NOT NULL DEFAULT 'global',
            empresa_id INTEGER,
            departamento_id INTEGER,
            regra_jornada TEXT,
            regra_compensacao TEXT,
            observacoes TEXT,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id),
            FOREIGN KEY (departamento_id) REFERENCES departamentos(id)
        );

        CREATE TABLE IF NOT EXISTS feriados_empresas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feriado_id INTEGER NOT NULL,
            empresa_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feriado_id) REFERENCES feriados(id) ON DELETE CASCADE,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS feriados_departamentos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feriado_id INTEGER NOT NULL,
            departamento_id INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feriado_id) REFERENCES feriados(id) ON DELETE CASCADE,
            FOREIGN KEY (departamento_id) REFERENCES departamentos(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS jornada_contextos_regras (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            empresa_id INTEGER,
            departamento_id INTEGER,
            funcao_id INTEGER,
            centro_custo_id INTEGER,
            jornada_id INTEGER,
            regra_compensacao TEXT,
            banco_horas_ativo INTEGER NOT NULL DEFAULT 1,
            permite_hora_extra INTEGER NOT NULL DEFAULT 0,
            compensa_atraso_com_extra INTEGER NOT NULL DEFAULT 0,
            usa_banco_para_excedente INTEGER NOT NULL DEFAULT 0,
            ativo INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id),
            FOREIGN KEY (departamento_id) REFERENCES departamentos(id),
            FOREIGN KEY (funcao_id) REFERENCES funcoes(id),
            FOREIGN KEY (centro_custo_id) REFERENCES centro_custos(id),
            FOREIGN KEY (jornada_id) REFERENCES jornadas_trabalho(id)
        );

        CREATE TABLE IF NOT EXISTS afd_importacoes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            empresa_id INTEGER,
            equipamento_id INTEGER,
            nome_arquivo TEXT NOT NULL,
            layout_portaria TEXT NOT NULL,
            formato_detectado TEXT NOT NULL,
            periodo_inicial TEXT,
            periodo_final TEXT,
            total_linhas INTEGER NOT NULL DEFAULT 0,
            total_marcacoes INTEGER NOT NULL DEFAULT 0,
            total_processadas INTEGER NOT NULL DEFAULT 0,
            total_descartadas INTEGER NOT NULL DEFAULT 0,
            hash_arquivo TEXT,
            conteudo_bruto TEXT,
            resumo_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id),
            FOREIGN KEY (equipamento_id) REFERENCES equipamentos(id)
        );

        CREATE TABLE IF NOT EXISTS batidas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            funcionario_id INTEGER NOT NULL,
            equipamento_id INTEGER,
            afd_importacao_id INTEGER,
            afd_layout_portaria TEXT,
            justificativa_id INTEGER,
            manual_ajuste INTEGER NOT NULL DEFAULT 0,
            validado INTEGER NOT NULL DEFAULT 1,
            data_referencia TEXT NOT NULL,
            hora TEXT NOT NULL,
            nsr TEXT,
            origem TEXT NOT NULL DEFAULT 'manual',
            observacao TEXT,
            tipo TEXT NOT NULL DEFAULT 'entrada',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id),
            FOREIGN KEY (equipamento_id) REFERENCES equipamentos(id),
            FOREIGN KEY (afd_importacao_id) REFERENCES afd_importacoes(id),
            FOREIGN KEY (justificativa_id) REFERENCES justificativas(id)
        );

        CREATE TABLE IF NOT EXISTS afd_marcacoes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            importacao_id INTEGER NOT NULL,
            nsr TEXT,
            tipo_registro TEXT,
            chave_trabalhador TEXT,
            data_hora_marcacao TEXT,
            data_hora_gravacao TEXT,
            coletor_codigo TEXT,
            online INTEGER,
            hash_registro TEXT,
            linha_bruta TEXT,
            funcionario_id INTEGER,
            batida_id INTEGER,
            status TEXT NOT NULL DEFAULT 'imported',
            mensagem TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (importacao_id) REFERENCES afd_importacoes(id) ON DELETE CASCADE,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id),
            FOREIGN KEY (batida_id) REFERENCES batidas(id)
        );

        CREATE TABLE IF NOT EXISTS ocorrencias_ponto (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            funcionario_id INTEGER NOT NULL,
            data_referencia TEXT NOT NULL,
            justificativa_id INTEGER,
            tipo TEXT NOT NULL,
            abonar_dia INTEGER NOT NULL DEFAULT 0,
            minutos_abonados INTEGER NOT NULL DEFAULT 0,
            observacao TEXT,
            anexo_nome TEXT,
            anexo_mime TEXT,
            anexo_base64 TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id),
            FOREIGN KEY (justificativa_id) REFERENCES justificativas(id)
        );

        CREATE TABLE IF NOT EXISTS banco_horas_lancamentos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            funcionario_id INTEGER NOT NULL,
            jornada_id INTEGER,
            data_referencia TEXT NOT NULL,
            minutos INTEGER NOT NULL,
            categoria TEXT NOT NULL,
            classificacao TEXT NOT NULL,
            origem TEXT NOT NULL,
            referencia_id INTEGER,
            observacao TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id),
            FOREIGN KEY (jornada_id) REFERENCES jornadas_trabalho(id)
        );

        CREATE TABLE IF NOT EXISTS fechamentos_mensais (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            funcionario_id INTEGER NOT NULL,
            empresa_id INTEGER,
            ano INTEGER NOT NULL,
            mes INTEGER NOT NULL,
            data_inicial TEXT NOT NULL,
            data_final TEXT NOT NULL,
            total_esperado_minutos INTEGER NOT NULL DEFAULT 0,
            total_trabalhado_minutos INTEGER NOT NULL DEFAULT 0,
            total_saldo_minutos INTEGER NOT NULL DEFAULT 0,
            total_atraso_minutos INTEGER NOT NULL DEFAULT 0,
            total_extra_minutos INTEGER NOT NULL DEFAULT 0,
            total_banco_horas_minutos INTEGER NOT NULL DEFAULT 0,
            resumo_json TEXT,
            relatorio_path TEXT,
            status TEXT NOT NULL DEFAULT 'fechado',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id),
            FOREIGN KEY (empresa_id) REFERENCES empresas(id)
        );

        CREATE TABLE IF NOT EXISTS relatorios_gerados (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            descricao TEXT NOT NULL,
            tipo_relatorio TEXT NOT NULL,
            origem_rotina TEXT NOT NULL,
            formato TEXT NOT NULL,
            file_name TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            file_path TEXT NOT NULL,
            competencia TEXT,
            funcionario_id INTEGER,
            funcionario_nome TEXT,
            usuario_login TEXT,
            detalhado INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'GERADO',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (funcionario_id) REFERENCES funcionarios(id)
        );

        CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_name TEXT NOT NULL,
            action_name TEXT NOT NULL,
            record_id INTEGER,
            payload_json TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_name TEXT NOT NULL,
            action_name TEXT NOT NULL,
            record_id INTEGER,
            payload_json TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            chave TEXT PRIMARY KEY,
            valor TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );



        CREATE TABLE IF NOT EXISTS app_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            level TEXT NOT NULL,
            category TEXT NOT NULL,
            message TEXT NOT NULL,
            source TEXT,
            route TEXT,
            details_json TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS admin_guard (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            support_secret_hash TEXT,
            totp_secret_encrypted TEXT,
            totp_enabled INTEGER NOT NULL DEFAULT 0,
            recovery_codes_encrypted TEXT,
            licensing_protected INTEGER NOT NULL DEFAULT 1,
            white_label_protected INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_rotated_at TEXT
        );

        CREATE TABLE IF NOT EXISTS admin_unlock_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            usuario_id INTEGER NOT NULL,
            scope TEXT NOT NULL,
            unlock_token TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_used_at TEXT,
            FOREIGN KEY (usuario_id) REFERENCES usuarios(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS local_licenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            empresa_id INTEGER NOT NULL,
            cnpj TEXT NOT NULL,
            license_kind TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            issued_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            fingerprint TEXT,
            payload_encrypted TEXT,
            integrity_hash TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (empresa_id) REFERENCES empresas(id)
        );

        CREATE TABLE IF NOT EXISTS configuracoes (
            nome TEXT PRIMARY KEY,
            valor TEXT,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .map_err(|err| format!("Falha ao executar migrations: {err}"))?;

    migrate_existing_schema(&conn)?;
    ensure_indexes(&conn)?;
    seed_data(&conn)
}

fn migrate_existing_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    for (table, column, definition) in [
        ("empresas", "nome_fantasia", "TEXT"),
        ("empresas", "inscricao_estadual", "TEXT"),
        ("empresas", "inscricao_municipal", "TEXT"),
        ("empresas", "responsavel_nome", "TEXT"),
        ("empresas", "responsavel_telefone", "TEXT"),
        ("empresas", "cep", "TEXT"),
        ("empresas", "numero", "TEXT"),
        ("empresas", "complemento", "TEXT"),
        ("empresas", "observacoes", "TEXT"),
        ("usuarios", "email", "TEXT"),
        ("usuarios", "telefone", "TEXT"),
        ("usuarios", "cargo", "TEXT"),
        ("usuarios", "observacoes", "TEXT"),
        ("usuarios", "master_user", "INTEGER NOT NULL DEFAULT 0"),
        ("usuarios", "senha_provisoria", "INTEGER NOT NULL DEFAULT 0"),
        ("usuarios", "ultimo_login_em", "TEXT"),
        ("funcionarios", "nome_social", "TEXT"),
        ("funcionarios", "rg", "TEXT"),
        ("funcionarios", "celular", "TEXT"),
        ("funcionarios", "data_nascimento", "TEXT"),
        ("funcionarios", "data_demissao", "TEXT"),
        ("funcionarios", "ferias_inicio", "TEXT"),
        ("funcionarios", "ferias_fim", "TEXT"),
        ("funcionarios", "ferias_dias", "INTEGER NOT NULL DEFAULT 0"),
        ("funcionarios", "sexo", "TEXT"),
        ("funcionarios", "estado_civil", "TEXT"),
        ("funcionarios", "cep", "TEXT"),
        ("funcionarios", "endereco", "TEXT"),
        ("funcionarios", "numero", "TEXT"),
        ("funcionarios", "complemento", "TEXT"),
        ("funcionarios", "bairro", "TEXT"),
        ("funcionarios", "cidade", "TEXT"),
        ("funcionarios", "estado", "TEXT"),
        ("funcionarios", "observacoes", "TEXT"),
        ("funcionarios", "jornada_id", "INTEGER"),
        ("jornadas_trabalho", "perfil_flexivel", "TEXT"),
        (
            "jornadas_trabalho",
            "permite_folga_movel",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        (
            "jornadas_trabalho",
            "permite_meia_folga",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        ("jornadas_trabalho", "dia_folga_base", "INTEGER"),
        ("jornadas_trabalho", "dia_folga_mensal_base", "INTEGER"),
        ("jornadas_trabalho", "periodo_meia_folga", "TEXT"),
        (
            "jornadas_trabalho",
            "heuristica_troca_folga",
            "INTEGER NOT NULL DEFAULT 1",
        ),
        (
            "jornadas_trabalho",
            "dias_trabalho_semana",
            "INTEGER NOT NULL DEFAULT 6",
        ),
        (
            "jornadas_trabalho",
            "folgas_mensais",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        (
            "jornadas_trabalho",
            "sabado_tipo",
            "TEXT NOT NULL DEFAULT 'integral'",
        ),
        (
            "jornadas_trabalho",
            "suporta_diarista_generico",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        (
            "jornadas_trabalho",
            "limite_dias_diarista",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        (
            "jornadas_trabalho",
            "semana_alternada_folga",
            "INTEGER NOT NULL DEFAULT 0",
        ),
        (
            "jornadas_trabalho",
            "tolerancia_entrada_minutos",
            "INTEGER NOT NULL DEFAULT 5",
        ),
        (
            "jornadas_trabalho",
            "tolerancia_saida_minutos",
            "INTEGER NOT NULL DEFAULT 5",
        ),
        (
            "jornadas_trabalho",
            "tolerancia_intervalo_minutos",
            "INTEGER NOT NULL DEFAULT 5",
        ),
        (
            "jornadas_trabalho",
            "carga_semanal_minutos",
            "INTEGER NOT NULL DEFAULT 2640",
        ),
        (
            "jornadas_trabalho",
            "limite_diario_minutos",
            "INTEGER NOT NULL DEFAULT 600",
        ),
        (
            "jornadas_trabalho",
            "banco_horas_ativo",
            "INTEGER NOT NULL DEFAULT 1",
        ),
        (
            "jornadas_trabalho",
            "exige_marcacao_intervalo",
            "INTEGER NOT NULL DEFAULT 1",
        ),
        (
            "jornadas_trabalho",
            "compensa_atraso_com_extra",
            "INTEGER NOT NULL DEFAULT 1",
        ),
        (
            "jornadas_trabalho",
            "modo_tratamento_afd",
            "TEXT NOT NULL DEFAULT 'auto'",
        ),
        ("jornadas_trabalho", "observacoes", "TEXT"),
        ("batidas", "afd_importacao_id", "INTEGER"),
        ("batidas", "afd_layout_portaria", "TEXT"),
        ("batidas", "justificativa_id", "INTEGER"),
        ("batidas", "manual_ajuste", "INTEGER NOT NULL DEFAULT 0"),
        ("batidas", "validado", "INTEGER NOT NULL DEFAULT 1"),
    ] {
        add_column_if_missing(conn, table, column, definition)?;
    }

    conn.execute_batch(
        r#"
        UPDATE empresas
           SET documento = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(documento, ''), '.', ''), '-', ''), '/', ''), '(', ''), ')', ''), ' ', '');
        UPDATE empresas
           SET telefone = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(telefone, ''), '(', ''), ')', ''), '-', ''), ' ', ''), '.', '');
        UPDATE empresas
           SET responsavel_telefone = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(responsavel_telefone, ''), '(', ''), ')', ''), '-', ''), ' ', ''), '.', '');
        UPDATE empresas
           SET cep = REPLACE(REPLACE(REPLACE(COALESCE(cep, ''), '.', ''), '-', ''), ' ', '');

        UPDATE usuarios
           SET login = LOWER(TRIM(COALESCE(login, '')));
        UPDATE usuarios
           SET telefone = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(telefone, ''), '(', ''), ')', ''), '-', ''), ' ', ''), '.', '');

        UPDATE funcionarios
           SET documento = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(documento, ''), '.', ''), '-', ''), '/', ''), '(', ''), ')', ''), ' ', '');
        UPDATE funcionarios
           SET pis = REPLACE(REPLACE(REPLACE(COALESCE(pis, ''), '.', ''), '-', ''), ' ', '');
        UPDATE funcionarios
           SET telefone = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(telefone, ''), '(', ''), ')', ''), '-', ''), ' ', ''), '.', '');
        UPDATE funcionarios
           SET celular = REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(celular, ''), '(', ''), ')', ''), '-', ''), ' ', ''), '.', '');
        UPDATE funcionarios
           SET cep = REPLACE(REPLACE(REPLACE(COALESCE(cep, ''), '.', ''), '-', ''), ' ', '');
        "#,
    )
    .map_err(|err| format!("Falha ao normalizar dados existentes: {err}"))?;

    Ok(())
}

fn add_column_if_missing(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn
        .prepare(&pragma)
        .map_err(|err| format!("Falha ao inspecionar tabela {table}: {err}"))?;

    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("Falha ao ler colunas de {table}: {err}"))?;

    let mut exists = false;
    for item in columns {
        if item.map_err(|err| format!("Falha ao mapear coluna de {table}: {err}"))? == column {
            exists = true;
            break;
        }
    }

    if !exists {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        conn.execute(&sql, [])
            .map_err(|err| format!("Falha ao adicionar coluna {column} em {table}: {err}"))?;
    }

    Ok(())
}

fn ensure_indexes(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_empresas_nome ON empresas(nome);
        CREATE INDEX IF NOT EXISTS idx_empresas_documento ON empresas(documento);
        CREATE INDEX IF NOT EXISTS idx_usuarios_login ON usuarios(login);
        CREATE INDEX IF NOT EXISTS idx_usuarios_master ON usuarios(master_user, ativo);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_perfis_nome ON perfis_acesso(nome);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_perfis_permissao ON perfis_permissoes(perfil_id, permissao_chave);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_usuarios_perfis ON usuarios_perfis(usuario_id, perfil_id);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_usuarios_empresas ON usuarios_empresas(usuario_id, empresa_id);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_user_sessions_token ON user_sessions(session_token);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_local_licenses_empresa ON local_licenses(empresa_id);
        CREATE INDEX IF NOT EXISTS idx_app_logs_created_at ON app_logs(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_app_logs_category ON app_logs(category);
        CREATE INDEX IF NOT EXISTS idx_admin_unlock_sessions_usuario ON admin_unlock_sessions(usuario_id);
        CREATE INDEX IF NOT EXISTS idx_admin_unlock_sessions_token ON admin_unlock_sessions(unlock_token);
        CREATE INDEX IF NOT EXISTS idx_local_licenses_cnpj ON local_licenses(cnpj);
        CREATE INDEX IF NOT EXISTS idx_app_settings_chave ON app_settings(chave);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_nome ON funcionarios(nome);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_documento ON funcionarios(documento);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_pis ON funcionarios(pis);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_empresa_matricula ON funcionarios(empresa_id, matricula);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_departamento ON funcionarios(departamento_id);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_funcao ON funcionarios(funcao_id);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_horario ON funcionarios(horario_id);
        CREATE INDEX IF NOT EXISTS idx_funcionarios_jornada ON funcionarios(jornada_id);
        CREATE INDEX IF NOT EXISTS idx_jornada_dias_jornada_dia ON jornada_dias(jornada_id, dia_semana);
        CREATE INDEX IF NOT EXISTS idx_batidas_funcionario_data ON batidas(funcionario_id, data_referencia);
        CREATE INDEX IF NOT EXISTS idx_batidas_nsr ON batidas(funcionario_id, nsr);
        CREATE INDEX IF NOT EXISTS idx_batidas_justificativa ON batidas(justificativa_id);
        CREATE INDEX IF NOT EXISTS idx_afd_importacoes_empresa ON afd_importacoes(empresa_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_afd_marcacoes_importacao ON afd_marcacoes(importacao_id);
        CREATE INDEX IF NOT EXISTS idx_ocorrencias_funcionario_data ON ocorrencias_ponto(funcionario_id, data_referencia);
        CREATE INDEX IF NOT EXISTS idx_ocorrencias_justificativa ON ocorrencias_ponto(justificativa_id);
        CREATE INDEX IF NOT EXISTS idx_banco_horas_funcionario_data ON banco_horas_lancamentos(funcionario_id, data_referencia);
        CREATE INDEX IF NOT EXISTS idx_relatorios_gerados_created_at ON relatorios_gerados(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_relatorios_gerados_tipo ON relatorios_gerados(tipo_relatorio);
        CREATE INDEX IF NOT EXISTS idx_relatorios_gerados_formato ON relatorios_gerados(formato);
        CREATE UNIQUE INDEX IF NOT EXISTS ux_fechamentos_funcionario_mes ON fechamentos_mensais(funcionario_id, ano, mes);
        "#,
    )
    .map_err(|err| format!("Falha ao criar índices: {err}"))?;

    Ok(())
}

fn seed_data(conn: &rusqlite::Connection) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    if bootstrap_seed_already_applied(conn)? {
        return Ok(());
    }

    if has_operational_data(conn)? {
        mark_bootstrap_seed(conn, &now, "skipped_existing_database")?;
        return Ok(());
    }

    let empresa_exists: Option<i64> = conn
        .query_row("SELECT id FROM empresas LIMIT 1", [], |row| row.get(0))
        .optional()
        .map_err(|err| format!("Falha ao verificar empresa inicial: {err}"))?;

    if empresa_exists.is_none() {
        conn.execute(
            "INSERT INTO empresas (
                nome, nome_fantasia, documento, telefone, email, responsavel_nome, responsavel_telefone,
                cep, endereco, numero, bairro, cidade, estado, ativo, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1, ?14, ?14)",
            params![
                "Empresa Demo Ltda",
                "Empresa Demo",
                "11222333000181",
                "75999990000",
                "demo@empresa.local",
                "Administrador Demo",
                "75999990000",
                "40000000",
                "Rua Exemplo",
                "100",
                "Centro",
                "Salvador",
                "BA",
                now,
            ],
        )
        .map_err(|err| format!("Falha ao criar empresa inicial: {err}"))?;
    }

    let admin_exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM usuarios WHERE login = 'admin' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar admin inicial: {err}"))?;

    if admin_exists.is_none() {
        let password_hash = hash_password("admin123")?;
        conn.execute(
            "INSERT INTO usuarios (nome, login, email, telefone, cargo, observacoes, senha_hash, master_user, administrador, senha_provisoria, ativo, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 1, 0, 1, ?8, ?8)",
            params!["Administrador Master", "admin", "admin@local", "75999990000", "Usuário master", "Usuário master inicial do sistema.", password_hash, now],
        )
        .map_err(|err| format!("Falha ao criar usuário admin inicial: {err}"))?;
    }

    ensure_access_seed(conn, &now)?;

    ensure_simple_seed(conn, "departamentos", "descricao", "Administrativo", &now)?;
    ensure_simple_seed(conn, "funcoes", "descricao", "Analista", &now)?;
    ensure_center_cost_seed(conn, &now)?;
    ensure_horario_seed(conn, &now)?;
    ensure_escala_seed(conn, &now)?;
    ensure_jornada_seed(conn, &now)?;
    ensure_equipamento_seed(conn, &now)?;
    ensure_evento_seed(conn, &now)?;
    ensure_justificativa_seed(conn, &now)?;
    ensure_funcionario_seed(conn, &now)?;

    conn.execute(
        "INSERT OR REPLACE INTO configuracoes (nome, valor, updated_at) VALUES ('carga_padrao_minutos', '480', ?1)",
        params![now],
    )
    .map_err(|err| format!("Falha ao gravar configuração padrão: {err}"))?;

    mark_bootstrap_seed(conn, &now, "applied")?;

    Ok(())
}

fn bootstrap_seed_already_applied(conn: &rusqlite::Connection) -> Result<bool, String> {
    let value: Option<i64> = conn
        .query_row(
            "SELECT CAST(valor AS INTEGER) FROM app_settings WHERE chave = ?1 LIMIT 1",
            params![BOOTSTRAP_SEED_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar versão do bootstrap seed: {err}"))?;

    Ok(value.unwrap_or(0) >= BOOTSTRAP_SEED_VERSION)
}

fn has_operational_data(conn: &rusqlite::Connection) -> Result<bool, String> {
    let business_tables = [
        "empresas",
        "usuarios",
        "perfis_acesso",
        "funcionarios",
        "departamentos",
        "funcoes",
        "centro_custos",
        "horarios",
        "escalas",
        "jornadas_trabalho",
        "equipamentos",
        "eventos",
        "justificativas",
        "batidas",
        "ocorrencias_ponto",
        "banco_horas_lancamentos",
        "fechamentos_mensais",
    ];

    for table in business_tables {
        let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} LIMIT 1)");
        let exists: i64 = conn.query_row(&sql, [], |row| row.get(0)).map_err(|err| {
            format!("Falha ao verificar dados operacionais na tabela {table}: {err}")
        })?;
        if exists == 1 {
            return Ok(true);
        }
    }

    Ok(false)
}

fn mark_bootstrap_seed(conn: &rusqlite::Connection, now: &str, status: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (chave, valor, updated_at) VALUES (?1, ?2, ?3)",
        params![BOOTSTRAP_SEED_KEY, BOOTSTRAP_SEED_VERSION.to_string(), now],
    )
    .map_err(|err| format!("Falha ao registrar versão do bootstrap seed: {err}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (chave, valor, updated_at) VALUES (?1, ?2, ?3)",
        params![BOOTSTRAP_SEED_STATUS_KEY, status, now],
    )
    .map_err(|err| format!("Falha ao registrar status do bootstrap seed: {err}"))?;

    Ok(())
}

fn ensure_access_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let admin_id: i64 = conn
        .query_row(
            "SELECT id FROM usuarios WHERE login = 'admin' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao localizar usuário admin: {err}"))?;

    conn.execute(
        "UPDATE usuarios
         SET master_user = 1, administrador = 1, senha_provisoria = 0, ativo = 1, updated_at = ?1
         WHERE id = ?2",
        params![now, admin_id],
    )
    .map_err(|err| format!("Falha ao promover usuário admin a master: {err}"))?;

    let perfil_master_id: i64 = match conn
        .query_row(
            "SELECT id FROM perfis_acesso WHERE LOWER(nome) = 'master' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar perfil master: {err}"))?
    {
        Some(id) => id,
        None => {
            conn.execute(
                "INSERT INTO perfis_acesso (nome, descricao, perfil_master, ativo, created_at, updated_at)
                 VALUES ('Master', 'Perfil com acesso total ao sistema.', 1, 1, ?1, ?1)",
                params![now],
            )
            .map_err(|err| format!("Falha ao criar perfil master: {err}"))?;
            conn.last_insert_rowid()
        }
    };

    conn.execute(
        "UPDATE perfis_acesso SET descricao = ?1, perfil_master = 1, ativo = 1, updated_at = ?2 WHERE id = ?3",
        params!["Perfil com acesso total ao sistema.", now, perfil_master_id],
    )
    .map_err(|err| format!("Falha ao atualizar perfil master: {err}"))?;

    conn.execute(
        "DELETE FROM perfis_permissoes WHERE perfil_id = ?1",
        [perfil_master_id],
    )
    .map_err(|err| format!("Falha ao limpar permissões do perfil master: {err}"))?;

    for key in access_permission_keys() {
        conn.execute(
            "INSERT OR IGNORE INTO perfis_permissoes (perfil_id, permissao_chave, created_at) VALUES (?1, ?2, ?3)",
            params![perfil_master_id, key, now],
        )
        .map_err(|err| format!("Falha ao gravar permissão do perfil master: {err}"))?;
    }

    conn.execute(
        "INSERT OR IGNORE INTO usuarios_perfis (usuario_id, perfil_id, created_at) VALUES (?1, ?2, ?3)",
        params![admin_id, perfil_master_id, now],
    )
    .map_err(|err| format!("Falha ao vincular perfil master ao admin: {err}"))?;

    let mut stmt = conn
        .prepare("SELECT id FROM empresas ORDER BY id ASC")
        .map_err(|err| format!("Falha ao preparar vínculo de empresas do admin: {err}"))?;
    let empresa_ids = stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|err| format!("Falha ao consultar empresas para o admin: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear empresas do admin: {err}"))?;

    for empresa_id in empresa_ids {
        conn.execute(
            "INSERT OR IGNORE INTO usuarios_empresas (usuario_id, empresa_id, created_at) VALUES (?1, ?2, ?3)",
            params![admin_id, empresa_id, now],
        )
        .map_err(|err| format!("Falha ao vincular empresa ao usuário admin: {err}"))?;
    }

    Ok(())
}

fn access_permission_keys() -> Vec<&'static str> {
    vec![
        "dashboard:view",
        "empresas:view",
        "empresas:manage",
        "funcionarios:view",
        "funcionarios:manage",
        "horarios:view",
        "horarios:manage",
        "escalas:view",
        "escalas:manage",
        "jornadas:view",
        "jornadas:manage",
        "feriados:view",
        "feriados:manage",
        "equipamentos:view",
        "equipamentos:manage",
        "eventos:view",
        "eventos:manage",
        "justificativas:view",
        "justificativas:manage",
        "batidas:view",
        "batidas:manage",
        "tratamentos:view",
        "tratamentos:manage",
        "afd:import",
        "apuracao:view",
        "apuracao:process",
        "banco_horas:view",
        "banco_horas:manage",
        "fechamentos:view",
        "fechamentos:manage",
        "sync:view",
        "usuarios:view",
        "usuarios:manage",
        "perfis:view",
        "perfis:manage",
        "config:view",
        "config:manage",
        "relatorios:export",
    ]
}

fn ensure_simple_seed(
    conn: &rusqlite::Connection,
    table: &str,
    field: &str,
    value: &str,
    now: &str,
) -> Result<(), String> {
    let sql_check = format!("SELECT id FROM {} WHERE {} = ?1 LIMIT 1", table, field);
    let exists: Option<i64> = conn
        .query_row(&sql_check, params![value], |row| row.get(0))
        .optional()
        .map_err(|err| format!("Falha ao verificar seed de {table}: {err}"))?;

    if exists.is_none() {
        let sql_insert = format!(
            "INSERT INTO {} ({}, ativo, created_at, updated_at) VALUES (?1, 1, ?2, ?2)",
            table, field
        );
        conn.execute(&sql_insert, params![value, now])
            .map_err(|err| format!("Falha ao inserir seed em {table}: {err}"))?;
    }

    Ok(())
}

fn ensure_center_cost_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM centro_custos WHERE codigo = 'ADM' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar centro de custo inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO centro_custos (codigo, descricao, ativo, created_at, updated_at) VALUES ('ADM', 'Administrativo', 1, ?1, ?1)",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir centro de custo inicial: {err}"))?;
    }
    Ok(())
}

fn ensure_horario_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM horarios WHERE descricao = 'Comercial 08h-18h' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar horário inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO horarios (numero, descricao, entrada_1, saida_1, entrada_2, saida_2, carga_horaria_minutos, ativo, created_at, updated_at)
             VALUES (1, 'Comercial 08h-18h', '08:00', '12:00', '13:00', '18:00', 540, 1, ?1, ?1)",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir horário inicial: {err}"))?;
    }
    Ok(())
}

fn ensure_escala_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM escalas WHERE descricao = 'Escala Segunda a Sexta' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar escala inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO escalas (descricao, horario_id, dias_ativos, tolerancia_minutos, ativo, created_at, updated_at)
             VALUES ('Escala Segunda a Sexta', 1, '1,2,3,4,5', 5, 1, ?1, ?1)",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir escala inicial: {err}"))?;
    }
    Ok(())
}

fn ensure_jornada_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM jornadas_trabalho WHERE descricao = 'Jornada Comercial Padrão' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar jornada inicial: {err}"))?;

    if exists.is_none() {
        conn.execute(
            "INSERT INTO jornadas_trabalho (
                empresa_id, codigo, descricao, tipo_jornada, tolerancia_entrada_minutos,
                tolerancia_saida_minutos, tolerancia_intervalo_minutos, carga_semanal_minutos,
                limite_diario_minutos, banco_horas_ativo, exige_marcacao_intervalo,
                compensa_atraso_com_extra, modo_tratamento_afd, observacoes, ativo, created_at, updated_at
             ) VALUES (
                1, 'JORN-001', 'Jornada Comercial Padrão', 'fixa', 5, 5, 5, 2700, 600, 1, 1, 1, 'auto', 'Jornada semanal comercial padrão.', 1, ?1, ?1
             )",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir jornada inicial: {err}"))?;
        let jornada_id = conn.last_insert_rowid();

        for dia in 1..=5 {
            conn.execute(
                "INSERT INTO jornada_dias (jornada_id, dia_semana, entrada_1, saida_1, entrada_2, saida_2, carga_prevista_minutos, intervalo_minutos, folga, created_at, updated_at)
                 VALUES (?1, ?2, '08:00', '12:00', '13:00', '18:00', 540, 60, 0, ?3, ?3)",
                params![jornada_id, dia, now],
            )
            .map_err(|err| format!("Falha ao inserir dia útil da jornada inicial: {err}"))?;
        }

        for dia in 6..=7 {
            conn.execute(
                "INSERT INTO jornada_dias (jornada_id, dia_semana, carga_prevista_minutos, intervalo_minutos, folga, created_at, updated_at)
                 VALUES (?1, ?2, 0, 0, 1, ?3, ?3)",
                params![jornada_id, dia, now],
            )
            .map_err(|err| format!("Falha ao inserir folga da jornada inicial: {err}"))?;
        }
    }

    Ok(())
}

fn ensure_equipamento_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM equipamentos WHERE descricao = 'REP Demo' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar equipamento inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO equipamentos (empresa_id, codigo, descricao, modelo, ip, porta, ativo, created_at, updated_at)
             VALUES (1, 'REP001', 'REP Demo', 'Virtual', '127.0.0.1', 3001, 1, ?1, ?1)",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir equipamento inicial: {err}"))?;
    }
    Ok(())
}

fn ensure_evento_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM eventos WHERE descricao = 'Hora extra 50%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar evento inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO eventos (codigo, descricao, tipo, impacta_banco_horas, ativo, created_at, updated_at)
             VALUES ('HEX50', 'Hora extra 50%', 'extra', 1, 1, ?1, ?1)",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir evento inicial: {err}"))?;
    }
    Ok(())
}

fn ensure_justificativa_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let defaults = [
        ("Atestado médico", 1),
        ("Falta justificada", 1),
        ("Esquecimento de marcação", 0),
        ("Ajuste manual autorizado", 0),
    ];

    for (descricao, abono) in defaults {
        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM justificativas WHERE descricao = ?1 LIMIT 1",
                params![descricao],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("Falha ao verificar justificativa inicial: {err}"))?;
        if exists.is_none() {
            conn.execute(
                "INSERT INTO justificativas (descricao, abono, ativo, created_at, updated_at) VALUES (?1, ?2, 1, ?3, ?3)",
                params![descricao, abono, now],
            )
            .map_err(|err| format!("Falha ao inserir justificativa inicial: {err}"))?;
        }
    }

    Ok(())
}

fn ensure_funcionario_seed(conn: &rusqlite::Connection, now: &str) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM funcionarios WHERE nome = 'Funcionário Demo' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar funcionário inicial: {err}"))?;
    if exists.is_none() {
        conn.execute(
            "INSERT INTO funcionarios (
                empresa_id, matricula, nome, documento, rg, pis, email, telefone, celular,
                data_nascimento, data_admissao, sexo, estado_civil, cep, endereco, numero,
                bairro, cidade, estado, departamento_id, funcao_id, centro_custo_id,
                horario_id, escala_id, jornada_id, observacoes, ativo, created_at, updated_at
             ) VALUES (
                1, '0001', 'Funcionário Demo', '39053344705', '1234567', '12044555877', 'funcionario@demo.local', '75999991111', '75999992222',
                '1990-01-01', '2024-01-01', 'M', 'solteiro', '40000000', 'Rua Colaborador', '50', 'Centro', 'Salvador', 'BA',
                1, 1, 1, 1, 1, 1, 'Funcionário de demonstração para testes locais.', 1, ?1, ?1
             )",
            params![now],
        )
        .map_err(|err| format!("Falha ao inserir funcionário inicial: {err}"))?;
    }
    Ok(())
}
