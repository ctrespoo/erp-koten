pub use super::forms::CadUnicoFormInput;

use sqlx::PgPool;
use thiserror::Error;

use super::errors::CadUnicoFormError;
use super::forms::CadUnicoFormData;
use super::repository::{CadUnicoRepository, CadUnicoRepositoryError};

pub struct CadUnicoService;

impl CadUnicoService {
    pub async fn create(
        pool: &PgPool,
        input: CadUnicoFormInput,
    ) -> Result<(), CadUnicoServiceError> {
        let normalized = Self::validate(input)?;

        CadUnicoRepository::insert(pool, &normalized)
            .await
            .map_err(|error| match error {
                CadUnicoRepositoryError::DuplicateCpfCnpj => {
                    CadUnicoServiceError::Form(CadUnicoFormError::duplicate_cpf_cnpj())
                }
                CadUnicoRepositoryError::Database(error) => CadUnicoServiceError::Unexpected(error),
            })
    }

    pub fn validate(input: CadUnicoFormInput) -> Result<CadUnicoFormData, CadUnicoFormError> {
        let normalized = input.normalize()?;
        let mut invalid_fields = Vec::new();

        if normalized.cpf_cnpj.is_empty() {
            invalid_fields.push("cpf_cnpj");
        }
        if normalized.fantasia.is_empty() {
            invalid_fields.push("fantasia");
        }
        if normalized.cep.is_empty() {
            invalid_fields.push("cep");
        }
        if normalized.endereco.is_empty() {
            invalid_fields.push("endereco");
        }
        if normalized.bairro.is_empty() {
            invalid_fields.push("bairro");
        }
        if normalized.cidade.is_empty() {
            invalid_fields.push("cidade");
        }
        if normalized.uf.is_empty() {
            invalid_fields.push("uf");
        }
        if normalized.codigo_ibge.is_empty() {
            invalid_fields.push("codigo_ibge");
        }

        if !invalid_fields.is_empty() {
            return Err(CadUnicoFormError::validation(invalid_fields));
        }

        Ok(normalized)
    }
}

#[derive(Debug, Error)]
pub enum CadUnicoServiceError {
    #[error(transparent)]
    Form(#[from] CadUnicoFormError),
    #[error(transparent)]
    Unexpected(#[from] sqlx::Error),
}

#[cfg(test)]
mod validate {
    use super::{CadUnicoFormInput, CadUnicoService};

    #[test]
    fn validate_should_return_error_when_required_fields_are_blank() {
        let input = CadUnicoFormInput::default();

        let error = CadUnicoService::validate(input).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Revise os campos destacados e tente novamente."
        );
        assert_eq!(
            error.invalid_fields(),
            &[
                "cpf_cnpj",
                "fantasia",
                "cep",
                "endereco",
                "bairro",
                "cidade",
                "uf",
                "codigo_ibge",
            ]
        );
    }

    #[test]
    fn validate_should_accept_minimal_valid_payload() {
        let input = CadUnicoFormInput::minimal_valid();

        let result = CadUnicoService::validate(input);

        assert!(
            result.is_ok(),
            "unexpected validation error: {:?}",
            result.unwrap_err()
        );
    }
}
