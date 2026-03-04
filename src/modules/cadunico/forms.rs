use serde::Deserialize;
use sqlx::types::chrono::NaiveDate;

use super::errors::CadUnicoFormError;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct CadUnicoFormInput {
    pub cpf_cnpj: String,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub fantasia: String,
    pub inss: Option<String>,
    pub crea: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub telefones: Vec<String>,
    pub aniversario: Option<String>,
    pub id_estrangeiro: Option<String>,
    pub codigo_pais: Option<String>,
    pub cep: String,
    pub endereco: String,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub codigo_ibge: String,
    #[serde(default)]
    pub enviar_nfe: bool,
    #[serde(default)]
    pub enviar_boleto: bool,
    #[serde(default)]
    pub enviar_extrato: bool,
    #[serde(default)]
    pub etiqueta: bool,
    #[serde(default)]
    pub comissao: bool,
    #[serde(default)]
    pub construcao_civil: bool,
    pub cep_cobranca: Option<String>,
    pub endereco_cobranca: Option<String>,
    pub numero_cobranca: Option<String>,
    pub complemento_cobranca: Option<String>,
    pub bairro_cobranca: Option<String>,
    pub cidade_cobranca: Option<String>,
    pub uf_cobranca: Option<String>,
    pub codigo_ibge_cobranca: Option<String>,
    pub referencia_cobranca: Option<String>,
}

#[derive(Debug)]
pub struct CadUnicoFormData {
    pub cpf_cnpj: String,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub fantasia: String,
    pub inss: Option<String>,
    pub crea: Option<String>,
    pub email: Option<String>,
    pub telefones: Vec<String>,
    pub aniversario: Option<NaiveDate>,
    pub id_estrangeiro: Option<String>,
    pub codigo_pais: Option<String>,
    pub cep: String,
    pub endereco: String,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub codigo_ibge: String,
    pub enviar_nfe: bool,
    pub enviar_boleto: bool,
    pub enviar_extrato: bool,
    pub etiqueta: bool,
    pub comissao: bool,
    pub construcao_civil: bool,
    pub cep_cobranca: Option<String>,
    pub endereco_cobranca: Option<String>,
    pub numero_cobranca: Option<String>,
    pub complemento_cobranca: Option<String>,
    pub bairro_cobranca: Option<String>,
    pub cidade_cobranca: Option<String>,
    pub uf_cobranca: Option<String>,
    pub codigo_ibge_cobranca: Option<String>,
    pub referencia_cobranca: Option<String>,
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn normalize_required_string(value: String) -> String {
    value.trim().to_owned()
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn normalize_optional_digits(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let digits = digits_only(&value);
        (!digits.is_empty()).then_some(digits)
    })
}

fn normalize_optional_uppercase(value: Option<String>) -> Option<String> {
    normalize_optional_string(value).map(|value| value.to_uppercase())
}

fn normalize_optional_date(value: Option<String>) -> Result<Option<NaiveDate>, CadUnicoFormError> {
    let Some(value) = normalize_optional_string(value) else {
        return Ok(None);
    };

    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| CadUnicoFormError::validation(vec!["aniversario"]))
}

fn parse_checkbox_value(value: &str) -> bool {
    matches!(value, "1" | "true" | "TRUE" | "on" | "ON")
}

impl CadUnicoFormInput {
    pub fn minimal_valid() -> Self {
        Self {
            cpf_cnpj: "12345678901".into(),
            fantasia: "Cliente teste".into(),
            cep: "01001000".into(),
            endereco: "Rua A".into(),
            bairro: "Centro".into(),
            cidade: "Sao Paulo".into(),
            uf: "SP".into(),
            codigo_ibge: "3550308".into(),
            ..Self::default()
        }
    }

    pub fn from_form_body(body: &str) -> Self {
        let mut input = Self::default();

        for (name, value) in form_urlencoded::parse(body.as_bytes()) {
            let value = value.into_owned();

            match name.as_ref() {
                "cpf_cnpj" => input.cpf_cnpj = value,
                "inscricao_estadual" => input.inscricao_estadual = Some(value),
                "inscricao_municipal" => input.inscricao_municipal = Some(value),
                "fantasia" => input.fantasia = value,
                "inss" => input.inss = Some(value),
                "crea" => input.crea = Some(value),
                "email" => input.email = Some(value),
                "telefones" => input.telefones.push(value),
                "aniversario" => input.aniversario = Some(value),
                "id_estrangeiro" => input.id_estrangeiro = Some(value),
                "codigo_pais" => input.codigo_pais = Some(value),
                "cep" => input.cep = value,
                "endereco" => input.endereco = value,
                "numero" => input.numero = Some(value),
                "complemento" => input.complemento = Some(value),
                "bairro" => input.bairro = value,
                "cidade" => input.cidade = value,
                "uf" => input.uf = value,
                "codigo_ibge" => input.codigo_ibge = value,
                "enviar_nfe" => input.enviar_nfe = parse_checkbox_value(&value),
                "enviar_boleto" => input.enviar_boleto = parse_checkbox_value(&value),
                "enviar_extrato" => input.enviar_extrato = parse_checkbox_value(&value),
                "etiqueta" => input.etiqueta = parse_checkbox_value(&value),
                "comissao" => input.comissao = parse_checkbox_value(&value),
                "construcao_civil" => input.construcao_civil = parse_checkbox_value(&value),
                "cep_cobranca" => input.cep_cobranca = Some(value),
                "endereco_cobranca" => input.endereco_cobranca = Some(value),
                "numero_cobranca" => input.numero_cobranca = Some(value),
                "complemento_cobranca" => input.complemento_cobranca = Some(value),
                "bairro_cobranca" => input.bairro_cobranca = Some(value),
                "cidade_cobranca" => input.cidade_cobranca = Some(value),
                "uf_cobranca" => input.uf_cobranca = Some(value),
                "codigo_ibge_cobranca" => input.codigo_ibge_cobranca = Some(value),
                "referencia_cobranca" => input.referencia_cobranca = Some(value),
                _ => {}
            }
        }

        input
    }

    pub fn normalize(self) -> Result<CadUnicoFormData, CadUnicoFormError> {
        let Self {
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
            referencia_cobranca,
        } = self;

        Ok(CadUnicoFormData {
            cpf_cnpj: digits_only(&cpf_cnpj),
            inscricao_estadual: normalize_optional_string(inscricao_estadual),
            inscricao_municipal: normalize_optional_string(inscricao_municipal),
            fantasia: normalize_required_string(fantasia),
            inss: normalize_optional_string(inss),
            crea: normalize_optional_string(crea),
            email: normalize_optional_string(email),
            telefones: telefones
                .into_iter()
                .map(|value| digits_only(&value))
                .filter(|value| !value.is_empty())
                .collect(),
            aniversario: normalize_optional_date(aniversario)?,
            id_estrangeiro: normalize_optional_string(id_estrangeiro),
            codigo_pais: normalize_optional_string(codigo_pais),
            cep: digits_only(&cep),
            endereco: normalize_required_string(endereco),
            numero: normalize_optional_string(numero),
            complemento: normalize_optional_string(complemento),
            bairro: normalize_required_string(bairro),
            cidade: normalize_required_string(cidade),
            uf: normalize_required_string(uf).to_uppercase(),
            codigo_ibge: normalize_required_string(codigo_ibge),
            enviar_nfe,
            enviar_boleto,
            enviar_extrato,
            etiqueta,
            comissao,
            construcao_civil,
            cep_cobranca: normalize_optional_digits(cep_cobranca),
            endereco_cobranca: normalize_optional_string(endereco_cobranca),
            numero_cobranca: normalize_optional_string(numero_cobranca),
            complemento_cobranca: normalize_optional_string(complemento_cobranca),
            bairro_cobranca: normalize_optional_string(bairro_cobranca),
            cidade_cobranca: normalize_optional_string(cidade_cobranca),
            uf_cobranca: normalize_optional_uppercase(uf_cobranca),
            codigo_ibge_cobranca: normalize_optional_string(codigo_ibge_cobranca),
            referencia_cobranca: normalize_optional_string(referencia_cobranca),
        })
    }
}

#[cfg(test)]
mod normalize {
    use super::CadUnicoFormInput;

    #[test]
    fn normalize_should_strip_non_digits_from_cpf_cnpj() {
        let input = CadUnicoFormInput {
            cpf_cnpj: "123.456.789-01".into(),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize().unwrap();

        assert_eq!(normalized.cpf_cnpj, "12345678901");
    }

    #[test]
    fn normalize_should_uppercase_uf_fields() {
        let input = CadUnicoFormInput {
            uf: "sp".into(),
            uf_cobranca: Some("rj".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize().unwrap();

        assert_eq!(normalized.uf, "SP");
        assert_eq!(normalized.uf_cobranca.as_deref(), Some("RJ"));
    }

    #[test]
    fn normalize_should_convert_blank_optional_fields_to_none() {
        let input = CadUnicoFormInput {
            inscricao_estadual: Some("   ".into()),
            email: Some("   ".into()),
            numero: Some("   ".into()),
            cep_cobranca: Some("   ".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize().unwrap();

        assert_eq!(normalized.inscricao_estadual, None);
        assert_eq!(normalized.email, None);
        assert_eq!(normalized.numero, None);
        assert_eq!(normalized.cep_cobranca, None);
    }

    #[test]
    fn normalize_should_parse_aniversario_when_present() {
        let input = CadUnicoFormInput {
            aniversario: Some("2024-12-31".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize().unwrap();

        assert_eq!(
            normalized
                .aniversario
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("2024-12-31")
        );
    }

    #[test]
    fn normalize_should_return_error_when_aniversario_is_invalid() {
        let input = CadUnicoFormInput {
            aniversario: Some("31-12-2024".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let error = input.normalize().unwrap_err();

        assert_eq!(
            error.to_string(),
            "Revise os campos destacados e tente novamente."
        );
        assert_eq!(error.invalid_fields(), &["aniversario"]);
    }
}
