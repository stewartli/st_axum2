use axum::{
    extract::Request, 
    http::StatusCode, 
    middleware::{from_fn, Next}, 
    response::{Html, IntoResponse}, 
    routing::get, 
    Router, 
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::{services::ServeDir, trace::{self, TraceLayer}};
use askama::{self, Template};

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

// 03. template 
#[derive(Template)]
#[template(path = "page.html")]
struct MyPageTemplate{
    title: String, 
    ctx: Option<String>, 
}

async fn handle_500(req: Request, next: Next) -> impl IntoResponse{
    let res = next.run(req).await;
    match res.status(){
        StatusCode::INTERNAL_SERVER_ERROR => {
            let tmp = MyPageTemplate{
                title: String::from("server error"),
                ctx: Some(String::from("try later")),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Html(tmp.render().unwrap())).into_response()
        }
        _ => res,
    }
}

async fn handle_404() -> impl IntoResponse{
    let tmp = MyPageTemplate{
        title: String::from("not found"),
        ctx: Some(String::from("unfound url")),
    };
    (StatusCode::NOT_FOUND, Html(tmp.render().unwrap())).into_response()
}

// 04. shutdown
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
        // http://localhost:8686/api-docs/
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", api_docs))
        // http://localhost:8686/static/app.css
        .nest_service("/static", ServeDir::new("static"))
        // http://localhost:8686/ab
        .layer(from_fn(handle_500))
        .fallback(handle_404)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

    // 3.3. server
    tracing::info!("starting server at {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
