use askama::Template;

pub struct TabView<'a> {
    pub id: &'a str,
    pub label: &'a str,
}

pub struct CadUnicoListItemView {
    pub id: i64,
    pub cpf_cnpj: String,
    pub fantasia: String,
    pub cidade: String,
    pub uf: String,
}

pub struct CadUnicoListPageView {
    pub heading: &'static str,
    pub search_value: String,
    pub items: Vec<CadUnicoListItemView>,
    pub next_cursor: Option<i64>,
    pub prev_cursor: Option<i64>,
}

impl CadUnicoListPageView {
    pub fn empty() -> Self {
        Self {
            heading: "Cadastros",
            search_value: String::new(),
            items: Vec::new(),
            next_cursor: None,
            prev_cursor: None,
        }
    }
}

#[derive(Template)]
#[template(path = "cadunico/index.html")]
pub struct CadUnicoIndexTemplate<'a> {
    pub page: &'a CadUnicoListPageView,
}

#[derive(Template)]
#[template(path = "cadunico/_list.html")]
pub struct CadUnicoListPartialTemplate<'a> {
    pub page: &'a CadUnicoListPageView,
}

#[derive(Template)]
#[template(path = "cadunico/create.html")]
pub struct CadUnicoCreateTemplate<'a> {
    pub tabs: &'a [TabView<'a>],
}

#[derive(Template)]
#[template(path = "cadunico/error_modal.html")]
pub struct CadUnicoErrorModalTemplate<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub invalid_fields: &'a str,
}

pub const TABS: &[TabView<'static>] = &[
    TabView {
        id: "dados-principais",
        label: "Dados principais",
    },
    TabView {
        id: "endereco",
        label: "Endereco",
    },
    TabView {
        id: "parametros",
        label: "Parametros",
    },
    TabView {
        id: "cobranca",
        label: "Cobranca",
    },
];
