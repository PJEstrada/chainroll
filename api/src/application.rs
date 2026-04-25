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
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest("/employees", employee_routes())
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
        .route("/count", get(routes::employee::count::count_employees))
        .route(
            "/{id}",
            get(routes::employee::get::get_employee)
                .delete(routes::employee::delete::delete_employee),
        )
        .route(
            "/{id}/exists",
            get(routes::employee::exists::employee_exists),
        )
}
