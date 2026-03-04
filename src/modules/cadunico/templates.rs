use askama::Template;

pub struct TabView<'a> {
    pub id: &'a str,
    pub label: &'a str,
}

#[derive(Template)]
#[template(path = "cadunico/index.html")]
pub struct CadUnicoIndexTemplate;

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
