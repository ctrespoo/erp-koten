use thiserror::Error;

use super::forms::CadUnicoFormData;

pub struct CadUnicoRepository;

pub struct CadUnicoListQuery {
    pub search: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub page_size: i64,
}

pub struct CadUnicoListPage {
    pub items: Vec<CadUnicoListItem>,
    pub next_cursor: Option<i64>,
    pub prev_cursor: Option<i64>,
}

pub struct CadUnicoListItem {
    pub id: i64,
    pub cpf_cnpj: String,
    pub fantasia: String,
    pub cidade: String,
    pub uf: String,
}

impl CadUnicoRepository {
    pub async fn insert(
        pool: &sqlx::PgPool,
        data: &CadUnicoFormData,
    ) -> Result<(), CadUnicoRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO cadunico (
                cpf_cnpj,
                inscricao_estadual,
                inscricao_municipal,
                fantasia,
                inss,
                crea,
                email,
                telefones,
                aniversario,
                id_estrangeiro,
                codigo_pais,
                cep,
                endereco,
                numero,
                complemento,
                bairro,
                cidade,
                uf,
                codigo_ibge,
                enviar_nfe,
                enviar_boleto,
                enviar_extrato,
                etiqueta,
                comissao,
                construcao_civil,
                cep_cobranca,
                endereco_cobranca,
                numero_cobranca,
                complemento_cobranca,
                bairro_cobranca,
                cidade_cobranca,
                uf_cobranca,
                codigo_ibge_cobranca,
                referencia_cobranca
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                $31, $32, $33, $34
            )
            "#,
        )
        .bind(&data.cpf_cnpj)
        .bind(data.inscricao_estadual.as_deref())
        .bind(data.inscricao_municipal.as_deref())
        .bind(&data.fantasia)
        .bind(data.inss.as_deref())
        .bind(data.crea.as_deref())
        .bind(data.email.as_deref())
        .bind(&data.telefones)
        .bind(data.aniversario)
        .bind(data.id_estrangeiro.as_deref())
        .bind(data.codigo_pais.as_deref())
        .bind(&data.cep)
        .bind(&data.endereco)
        .bind(data.numero.as_deref())
        .bind(data.complemento.as_deref())
        .bind(&data.bairro)
        .bind(&data.cidade)
        .bind(&data.uf)
        .bind(&data.codigo_ibge)
        .bind(data.enviar_nfe)
        .bind(data.enviar_boleto)
        .bind(data.enviar_extrato)
        .bind(data.etiqueta)
        .bind(data.comissao)
        .bind(data.construcao_civil)
        .bind(data.cep_cobranca.as_deref())
        .bind(data.endereco_cobranca.as_deref())
        .bind(data.numero_cobranca.as_deref())
        .bind(data.complemento_cobranca.as_deref())
        .bind(data.bairro_cobranca.as_deref())
        .bind(data.cidade_cobranca.as_deref())
        .bind(data.uf_cobranca.as_deref())
        .bind(data.codigo_ibge_cobranca.as_deref())
        .bind(data.referencia_cobranca.as_deref())
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|error| match error {
            sqlx::Error::Database(database_error)
                if database_error.code().as_deref() == Some("23505") =>
            {
                CadUnicoRepositoryError::DuplicateCpfCnpj
            }
            other => CadUnicoRepositoryError::Database(other),
        })
    }

    pub async fn list(
        pool: &sqlx::PgPool,
        query: &CadUnicoListQuery,
    ) -> Result<CadUnicoListPage, CadUnicoRepositoryError> {
        let search = query.search.as_deref().unwrap_or("");
        let search_pattern = format!("%{search}%");
        let page_limit = query.page_size + 1;

        if let Some(after) = query.after {
            let mut rows = sqlx::query_as::<_, (i64, String, String, String, String)>(
                r#"
                SELECT id, cpf_cnpj, fantasia, cidade, uf
                FROM cadunico
                WHERE (
                    $1 = ''
                    OR cpf_cnpj ILIKE $2
                    OR fantasia ILIKE $2
                    OR cidade ILIKE $2
                )
                AND id > $3
                ORDER BY id ASC
                LIMIT $4
                "#,
            )
            .bind(search)
            .bind(&search_pattern)
            .bind(after)
            .bind(page_limit)
            .fetch_all(pool)
            .await?;

            let has_newer_rows = rows.len() as i64 > query.page_size;
            if has_newer_rows {
                rows.pop();
            }

            rows.reverse();

            let items: Vec<_> = rows.into_iter().map(CadUnicoListItem::from).collect();
            let next_cursor = items.last().map(|item| item.id);
            let prev_cursor = has_newer_rows.then(|| items.first().map(|item| item.id)).flatten();

            return Ok(CadUnicoListPage {
                items,
                next_cursor,
                prev_cursor,
            });
        }

        let mut rows = sqlx::query_as::<_, (i64, String, String, String, String)>(
            r#"
            SELECT id, cpf_cnpj, fantasia, cidade, uf
            FROM cadunico
            WHERE (
                $1 = ''
                OR cpf_cnpj ILIKE $2
                OR fantasia ILIKE $2
                OR cidade ILIKE $2
            )
            AND ($3::bigint IS NULL OR id < $3)
            ORDER BY id DESC
            LIMIT $4
            "#,
        )
        .bind(search)
        .bind(&search_pattern)
        .bind(query.before)
        .bind(page_limit)
        .fetch_all(pool)
        .await?;

        let has_older_rows = rows.len() as i64 > query.page_size;
        if has_older_rows {
            rows.pop();
        }

        let items: Vec<_> = rows.into_iter().map(CadUnicoListItem::from).collect();
        let next_cursor = has_older_rows
            .then(|| items.last().map(|item| item.id))
            .flatten();
        let prev_cursor = query.before.and_then(|_| items.first().map(|item| item.id));

        Ok(CadUnicoListPage {
            items,
            next_cursor,
            prev_cursor,
        })
    }

    pub async fn delete(pool: &sqlx::PgPool, id: i64) -> Result<(), CadUnicoRepositoryError> {
        sqlx::query("DELETE FROM cadunico WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(CadUnicoRepositoryError::from)
    }
}

impl From<(i64, String, String, String, String)> for CadUnicoListItem {
    fn from(row: (i64, String, String, String, String)) -> Self {
        Self {
            id: row.0,
            cpf_cnpj: row.1,
            fantasia: row.2,
            cidade: row.3,
            uf: row.4,
        }
    }
}

#[derive(Debug, Error)]
pub enum CadUnicoRepositoryError {
    #[error("cadunico with this cpf_cnpj already exists")]
    DuplicateCpfCnpj,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
