use axum::{Router, response::IntoResponse, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// 01. openapi
#[derive(OpenApi)]
#[openapi(
    paths(handle_hello), 
    tags((name="hello", description="hello endpoint")), 
    info(title="hello API", version="1.1.1", description="a simple API")
)]
struct MyApiDoc;

// 02. route, handler
fn route_hello() -> Router {
    Router::new().route("/", get(handle_hello))
}

#[utoipa::path(
    get, 
    path = "/", 
    tag = "hello", 
    responses((status = 200, description = "greet ok", body = String))
)]
async fn handle_hello() -> impl IntoResponse {
    "hello wrold from landing page"
}

// 03. shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };
    /*
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
    */

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("starting graceful shutdown");
}

#[tokio::main]
async fn main() {
    // 01. logger
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 02. API 
    let api_docs = MyApiDoc::openapi();

    // 03. server
    // 3.1 listener
    let addr = "127.0.0.1:8686";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // 3.2. axum app
    let app = route_hello()
        .merge(SwaggerUi::new("api-docs").url("/api-docs/openapi.json", api_docs));

    // 3.3. server
    tracing::info!("starting server at {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
