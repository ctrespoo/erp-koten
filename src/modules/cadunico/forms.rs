use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrManyStrings {
    One(String),
    Many(Vec<String>),
}

fn deserialize_string_sequence_or_single<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<OneOrManyStrings>::deserialize(deserializer)? {
        Some(OneOrManyStrings::One(value)) => Ok(vec![value]),
        Some(OneOrManyStrings::Many(values)) => Ok(values),
        None => Ok(Vec::new()),
    }
}

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
    #[serde(default, deserialize_with = "deserialize_string_sequence_or_single")]
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
    pub fantasia: String,
    pub cep: String,
    pub endereco: String,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub codigo_ibge: String,
    pub uf_cobranca: Option<String>,
    pub telefones: Vec<String>,
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn normalize_optional_uppercase(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_uppercase())
    })
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

    pub fn normalize(self) -> CadUnicoFormData {
        let Self {
            cpf_cnpj,
            fantasia,
            cep,
            endereco,
            bairro,
            cidade,
            uf,
            codigo_ibge,
            uf_cobranca,
            telefones,
            ..
        } = self;

        CadUnicoFormData {
            cpf_cnpj: digits_only(&cpf_cnpj),
            fantasia: fantasia.trim().to_owned(),
            cep: digits_only(&cep),
            endereco: endereco.trim().to_owned(),
            bairro: bairro.trim().to_owned(),
            cidade: cidade.trim().to_owned(),
            uf: uf.trim().to_uppercase(),
            codigo_ibge: codigo_ibge.trim().to_owned(),
            uf_cobranca: normalize_optional_uppercase(uf_cobranca),
            telefones: telefones
                .into_iter()
                .map(|value| digits_only(&value))
                .filter(|value| !value.is_empty())
                .collect(),
        }
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

        let normalized = input.normalize();

        assert_eq!(normalized.cpf_cnpj, "12345678901");
    }

    #[test]
    fn normalize_should_uppercase_uf_fields() {
        let input = CadUnicoFormInput {
            uf: "sp".into(),
            uf_cobranca: Some("rj".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize();

        assert_eq!(normalized.uf, "SP");
        assert_eq!(normalized.uf_cobranca.as_deref(), Some("RJ"));
    }
}
