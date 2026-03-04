use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CadUnicoFormError {
    #[error("cpf_cnpj is required")]
    MissingCpfCnpj,
    #[error("fantasia is required")]
    MissingFantasia,
    #[error("cep is required")]
    MissingCep,
    #[error("endereco is required")]
    MissingEndereco,
    #[error("bairro is required")]
    MissingBairro,
    #[error("cidade is required")]
    MissingCidade,
    #[error("uf is required")]
    MissingUf,
    #[error("codigo_ibge is required")]
    MissingCodigoIbge,
}
