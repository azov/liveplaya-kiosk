use crate::{
    err::{Error, Result},
    io,
};
use actix_web::{self, get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use tokio::sync::mpsc;

pub type JsonQuery = JsonValue;
pub use serde_json::{json, Value as JsonValue};


#[get("/")]
async fn redirect_to_app() -> impl Responder {
    HttpResponse::Found()
    .header("Location", "/map") 
    .finish()
}

#[get("/")]
async fn get_app() -> impl Responder {
    //HttpResponse::TemporaryRedirect()
    HttpResponse::Ok().content_type("text/html").body(
        "
        <p>Congratulations, your server is running!</p>
        <p>Now specify <code>wwwroot</code> option to see the UI or 
        <a href='api/v0/'>play with the API</a>.</p>
        ",
    )
}

#[get("/api/v0/")]
async fn get_view(
    req: HttpRequest,
    query: web::Query<io::user::Query>, /*, urlargs: web::Path<String> */
) -> impl Responder {
    let q: io::user::Query = match query.0.try_into() {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
            "status": "bad request",
            "message": e.to_string(),
            }));
        }
    };

    let back = req.app_data::<mpsc::Sender<io::user::Event>>().unwrap();
    let (res_tx, res_rx) = tokio::sync::oneshot::channel::<Result<io::user::View>>();
    let evt = io::user::Event::ViewRequest(q, res_tx);
    if let Err(_) = back.try_send(evt) {
        return HttpResponse::ServiceUnavailable().json(json!({
        "status": "server busy",
        "message": "We're experiencing high request volume, please try again later.",
        }));
    }
    match res_rx.await {
        Ok(Ok(view)) => HttpResponse::Ok().json(append(
            serde_json::to_value(view).unwrap(),
            json!({
                "status": "ok",
            }),
        )),
        Ok(Err(Error::BadRequest(e))) => HttpResponse::BadRequest().json(json!({
        "status": "bad request",
        "message": e,
        })),
        Ok(Err(e)) => {
            log::error!("internal server error: {}", e);
            HttpResponse::InternalServerError().json(json!({
            "status": "internal server error",
            "message": "something went wrong on our side, please try again later",
            }))
        }
        Err(_) => {
            log::error!("internal server error: failed to get response from backend");
            HttpResponse::InternalServerError().json(json!({
            "status": "internal server error",
            "message": "something went wrong on our side, please try again later",
            }))
        }
    }
}

pub async fn run(
    port: u16,
    www_root: Option<std::path::PathBuf>,
    backend: mpsc::Sender<io::user::Event>,
) -> Result<()> {
    log::debug!(
        "wwwroot: {}",
        www_root
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or("none".into())
    );
    HttpServer::new(move || {
        let app = App::new()
            .app_data(backend.clone())
            .wrap(actix_web::middleware::Logger::new("%a %r %s"))
            .service(get_view);
        let app = if let Some(dir) = &www_root {
            app.service(
                actix_files::Files::new("/", dir)
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
        } else {
            app.service(get_app)
        };
        app
    })
    .bind(("0.0.0.0", port))
    .map_err(|e| Error::OtherWithContext("failed to bind web server", e.to_string()))?
    .run()
    .await
    .map_err(|e| Error::OtherWithContext("failed to run web server", e.to_string()))
}

fn append(mut a: JsonValue, mut b: JsonValue) -> JsonValue {
    a.as_object_mut()
        .unwrap()
        .append(b.as_object_mut().unwrap());
    a
}
