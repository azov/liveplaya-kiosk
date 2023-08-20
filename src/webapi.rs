use crate::{
    err::{Error, Result},
    io,
    util::twoway,
};
use actix_web::{self, get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use tokio_util::sync::CancellationToken;

pub type JsonQuery = JsonValue;
pub use serde_json::{json, Value as JsonValue};


#[get("/")]
async fn get_index() -> impl Responder {
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
    query: web::Query<io::Query>, /*, urlargs: web::Path<String> */
) -> impl Responder {
    let query: io::Query = match query.0.try_into() {
        Ok(q) => {
            log::debug!("query: {} -> {:?}", req.query_string(), q);
            q
        },
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
            "status": "bad request",
            "message": e.to_string(),
            }));
        }
    };

    let io = req.app_data::<IO>().unwrap();
    let res = twoway::request(query, &io.query_sender, &io.stop).await;
    match res {
        Ok(res) => HttpResponse::Ok().json(append(
            serde_json::to_value(res).unwrap(),
            json!({
                "status": "ok",
            }),
        )),
        Err(Error::BadRequest(e)) => {
            HttpResponse::BadRequest().json(json!({
            "status": "bad request",
            "message": e,
            }))
        },
        Err(e) => {
            log::error!("internal server error: {}", e);
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
    query_sender: io::QuerySender,
    stop: CancellationToken,
) -> Result<()> {
    let io = IO { query_sender, stop };
    log::debug!(
        "wwwroot: {}",
        www_root
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or("none".into())
    );
    HttpServer::new(move || {
        let app = App::new()
            .app_data(io.clone())
            .wrap(actix_web::middleware::Logger::new("%a %r %s"))
            .service(get_view);
        let app = if let Some(dir) = &www_root {
            app.service(
                actix_files::Files::new("/", dir)
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
        } else {
            app.service(get_index)
        };
        app
    })
    .bind(("0.0.0.0", port))
    .map_err(|e| Error::OtherWithContext("failed to bind web server", e.to_string()))?
    .run()
    .await
    .map_err(|e| Error::OtherWithContext("failed to run web server", e.to_string()))
}

#[derive(Debug, Clone)]
struct IO {
    query_sender: io::QuerySender,
    stop: CancellationToken,
}

fn append(mut a: JsonValue, mut b: JsonValue) -> JsonValue {
    a.as_object_mut()
        .unwrap()
        .append(b.as_object_mut().unwrap());
    a
}
