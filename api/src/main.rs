use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let result = axum::serve(listener, app).await;
    if let Err(e) = result {
        eprintln!("server error: {}", e);
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
