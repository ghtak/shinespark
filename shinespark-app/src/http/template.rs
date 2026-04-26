use std::path::PathBuf;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use minijinja::{Environment, path_loader};

#[allow(dead_code)]
enum Inner {
    Dev(Environment<'static>),
    Prod(PathBuf),
}

pub struct TemplateEnv(Inner);

impl TemplateEnv {
    pub fn new(dir: &str) -> Self {
        #[cfg(debug_assertions)]
        let mut env = Environment::new();
        env.set_loader(path_loader(dir));
        return Self(Inner::Dev(env));

        #[cfg(not(debug_assertions))]
        return Self(Inner::Prod(PathBuf::from(dir)));
    }

    pub fn render(&self, name: &str, ctx: minijinja::Value) -> Result<String, minijinja::Error> {
        match &self.0 {
            Inner::Dev(env) => env.get_template(name)?.render(ctx),
            Inner::Prod(dir) => {
                let mut env = Environment::new();
                env.set_loader(path_loader(dir.clone()));
                env.get_template(name)?.render(ctx)
            }
        }
    }
}

pub struct TemplateResponse(pub String);

impl IntoResponse for TemplateResponse {
    fn into_response(self) -> Response {
        ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], self.0).into_response()
    }
}
