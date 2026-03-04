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
