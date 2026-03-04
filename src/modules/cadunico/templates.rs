use askama::Template;

#[derive(Template)]
#[template(path = "cadunico/index.html")]
pub struct CadUnicoIndexTemplate;
