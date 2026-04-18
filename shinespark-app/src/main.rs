use std::sync::Arc;

use axum::extract::State;
use shinespark::config::AppConfig;

extern crate shinespark;
mod http;

#[derive(Clone)]
pub struct AppContainer {
    pub db: shinespark::db::Database,
    pub user_usecase: Arc<dyn shinespark_identity::usecases::UserUsecase>,
    pub login_usecase: Arc<dyn shinespark_identity::usecases::LoginUsecase>,
    pub rbac_usecase: Arc<dyn shinespark_identity::usecases::RbacUsecase>,
    pub jwt_ident_usecase: Arc<dyn shinespark_identity::usecases::JwtIdentUsecase>,
    pub jwt_service: Arc<dyn shinespark_identity::infra::JwtService>,
}

impl AppContainer {
    pub fn new(db: shinespark::db::Database, config: &AppConfig) -> Self {
        let password_service = Arc::new(shinespark::crypto::password::B64PasswordService::new());
        let user_repository = Arc::new(shinespark_identity::infra::SqlxUserRepository::new());
        let user_usecase = Arc::new(shinespark_identity::infra::DefaultUserUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        let login_usecase = Arc::new(shinespark_identity::infra::DefaultLoginUsecase::new(
            user_repository.clone(),
            password_service.clone(),
        ));
        let rbac_usecase = Arc::new(shinespark_identity::infra::DefaultRbacUsecase::new());

        let jwt_service = Arc::new(shinespark_identity::infra::HS256JwtService::new(&config.jwt));
        let jwt_repository = Arc::new(shinespark_identity::infra::SqlxJwtIdentRepository::new());
        let jwt_ident_usecase = Arc::new(shinespark_identity::infra::DefaultJwtIdentUsecase::new(
            login_usecase.clone(),
            user_usecase.clone(),
            jwt_service.clone(),
            jwt_repository,
        ));

        Self {
            db,
            user_usecase,
            login_usecase,
            rbac_usecase,
            jwt_ident_usecase,
            jwt_service,
        }
    }
}

#[tokio::main]
async fn main() {
    AppConfig::load_dotenv();
    let config = AppConfig::new().expect("failed to load config");
    shinespark::trace::init(&config.trace).expect("failed to init trace");
    let db =
        shinespark::db::Database::new(&config.database).await.expect("failed to create database");
    let container = Arc::new(AppContainer::new(db, &config));

    shinespark_identity::infra::seed_admin(
        &mut container.db.handle(),
        container.user_usecase.clone(),
    )
    .await;

    let router = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|State(container): State<Arc<AppContainer>>| async move {
                let mut handle = container.db.handle();
                let result = sqlx::query("SELECT 1").execute(handle.inner()).await.unwrap();
                format!("Hello, world! {}", result.rows_affected())
            }),
        )
        .merge(http::routes::identity::routes())
        .layer(http::session::simple_layer())
        .with_state(container);

    shinespark::http::run(router, &config.http).await.expect("failed to run http server");
}
