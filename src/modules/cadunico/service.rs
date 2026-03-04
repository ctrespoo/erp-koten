pub use super::forms::CadUnicoFormInput;

use super::errors::CadUnicoFormError;
use super::forms::CadUnicoFormData;

pub struct CadUnicoService;

impl CadUnicoService {
    pub fn validate(input: CadUnicoFormInput) -> Result<CadUnicoFormData, CadUnicoFormError> {
        let normalized = input.normalize();

        if normalized.cpf_cnpj.is_empty() {
            return Err(CadUnicoFormError::MissingCpfCnpj);
        }
        if normalized.fantasia.is_empty() {
            return Err(CadUnicoFormError::MissingFantasia);
        }
        if normalized.cep.is_empty() {
            return Err(CadUnicoFormError::MissingCep);
        }
        if normalized.endereco.is_empty() {
            return Err(CadUnicoFormError::MissingEndereco);
        }
        if normalized.bairro.is_empty() {
            return Err(CadUnicoFormError::MissingBairro);
        }
        if normalized.cidade.is_empty() {
            return Err(CadUnicoFormError::MissingCidade);
        }
        if normalized.uf.is_empty() {
            return Err(CadUnicoFormError::MissingUf);
        }
        if normalized.codigo_ibge.is_empty() {
            return Err(CadUnicoFormError::MissingCodigoIbge);
        }

        Ok(normalized)
    }
}

#[cfg(test)]
mod validate {
    use super::{CadUnicoFormInput, CadUnicoService};

    #[test]
    fn validate_should_return_error_when_required_fields_are_blank() {
        let input = CadUnicoFormInput::default();

        let error = CadUnicoService::validate(input).unwrap_err();

        assert_eq!(error.to_string(), "cpf_cnpj is required");
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
