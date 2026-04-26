use std::path::PathBuf;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use minijinja::{Environment, path_loader};
use shinespark::config::AppConfig;

enum Inner {
    Dev(PathBuf),
    Prod(Environment<'static>),
}

pub struct TemplateEnv(Inner);

impl TemplateEnv {
    pub fn new(dir: &str) -> Self {
        let is_prod = AppConfig::run_mode() == "prod";
        if is_prod {
            let mut env = Environment::new();
            env.set_loader(path_loader(dir));
            Self(Inner::Prod(env))
        } else {
            Self(Inner::Dev(PathBuf::from(dir)))
        }
    }

    pub fn render(&self, name: &str, ctx: minijinja::Value) -> Result<String, minijinja::Error> {
        match &self.0 {
            Inner::Dev(dir) => {
                let mut env = Environment::new();
                env.set_loader(path_loader(dir.clone()));
                env.get_template(name)?.render(ctx)
            }
            Inner::Prod(env) => env.get_template(name)?.render(ctx),
        }
    }
}

pub struct TemplateResponse(pub String);

impl IntoResponse for TemplateResponse {
    fn into_response(self) -> Response {
        ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], self.0).into_response()
    }
}
