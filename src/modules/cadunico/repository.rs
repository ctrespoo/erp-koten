use thiserror::Error;

use super::forms::CadUnicoFormData;

pub struct CadUnicoRepository;

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
}

#[derive(Debug, Error)]
pub enum CadUnicoRepositoryError {
    #[error("cadunico with this cpf_cnpj already exists")]
    DuplicateCpfCnpj,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
