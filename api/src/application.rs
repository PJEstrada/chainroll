use crate::app_state::AppState;
use crate::routes;
use crate::utils::tracing::{make_span_with_request_id, on_request, on_response};
use axum::Router;
use axum::routing::get;
use axum::serve::Serve;
use http::Method;
use std::error::Error;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://167.71.176.216:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest("/employees", employee_routes())
            .nest(
                "/employees/{employee_id}/compensation-profiles",
                compensation_routes(),
            )
            .nest("/treasury-accounts", treasury_routes())
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        let server = axum::serve(listener, router);
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        self.server.await?;
        Ok(())
    }
}

fn employee_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(routes::employee::list::list_employees)
                .post(routes::employee::create::create_employee),
        )
        .route(
            "/{id}",
            get(routes::employee::get::get_employee)
                .put(routes::employee::update::update_employee)
                .delete(routes::employee::delete::delete_employee),
        )
        .route("/count", get(routes::employee::count::count_employees))
        .route(
            "/{id}/exists",
            get(routes::employee::exists::employee_exists),
        )
}

fn treasury_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(routes::treasury::list::list_treasury_accounts)
                .post(routes::treasury::create::create_treasury_account),
        )
        .route(
            "/{id}",
            get(routes::treasury::get::get_treasury_account)
                .put(routes::treasury::update::update_treasury_account)
                .delete(routes::treasury::deactivate::deactivate_treasury_account),
        )
}

fn compensation_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(routes::compensation::list::list_compensation_profiles)
                .post(routes::compensation::create::create_compensation_profile),
        )
        .route(
            "/active",
            get(routes::compensation::get_active::get_active_compensation_profile),
        )
        .route(
            "/{id}",
            get(routes::compensation::get::get_compensation_profile)
                .put(routes::compensation::update::update_compensation_profile),
        )
}
