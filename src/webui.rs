use liveplaya::actix_web::{http, App, HttpRequest, HttpResponse, Responder};

static INDEX_HTML: &'static str = include_str!("webui/index.html");

#[cfg(debug_assertions)]
static APP_JS: &'static str = include_str!("webui/build/debug.js");
#[cfg(not(debug_assertions))]
static APP_JS: &'static str = include_str!("webui/build/min.js");

fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(INDEX_HTML)
}

fn app_js(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(APP_JS)
}

pub fn configure(app: App) -> App {
    app.route("/", http::Method::GET, index)
        .route("/app.js", http::Method::GET, app_js)
}
