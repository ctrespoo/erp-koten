use thiserror::Error;

const DEFAULT_SUMMARY: &str = "Revise os campos destacados e tente novamente.";
const KNOWN_FIELDS: &[&str] = &[
    "cpf_cnpj",
    "fantasia",
    "telefones",
    "cep",
    "endereco",
    "bairro",
    "cidade",
    "uf",
    "codigo_ibge",
];

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{summary}")]
pub struct CadUnicoFormError {
    summary: &'static str,
    invalid_fields: Vec<&'static str>,
}

impl CadUnicoFormError {
    pub fn validation(invalid_fields: Vec<&'static str>) -> Self {
        Self {
            summary: DEFAULT_SUMMARY,
            invalid_fields,
        }
    }

    pub fn from_rejection_message(message: &str) -> Self {
        let invalid_fields = KNOWN_FIELDS
            .iter()
            .copied()
            .filter(|field| message.contains(field))
            .collect();

        Self::validation(invalid_fields)
    }

    pub fn invalid_fields(&self) -> &[&'static str] {
        &self.invalid_fields
    }

    pub fn invalid_fields_csv(&self) -> String {
        self.invalid_fields.join(",")
    }
}
