use crate::{
    err::{Error, Result},
    twoway,
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
        This is API root. Try <a href='api/v0/'>this</a>.
        ",
    )
}

#[get("/api/v0/")]
async fn get_view(
    req: HttpRequest,
    query: web::Query<JsonQuery>, /*, urlargs: web::Path<String> */
) -> impl Responder {
    let query: JsonQuery = match query.0.try_into() {
        Ok(q) => q,
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
            res,
            json!({
                "status": "ok",
            }),
        )),
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
    query_sender: twoway::Sender<JsonQuery, JsonValue>,
    stop: CancellationToken,
) -> Result<()> {
    let io = IO { query_sender, stop };
    HttpServer::new(move || {
        App::new()
            .app_data(io.clone())
            .wrap(actix_web::middleware::Logger::new("%a %r %s"))
            // .service(connect)
            .service(get_view)
            .service(get_index)
    })
    .bind(("0.0.0.0", port))
    .map_err(|e| Error::OtherWithContext("failed to bind web server", e.to_string()))?
    .run()
    .await
    .map_err(|e| Error::OtherWithContext("failed to run web server", e.to_string()))
}

#[derive(Debug, Clone)]
struct IO {
    query_sender: twoway::Sender<JsonQuery, JsonValue>,
    stop: CancellationToken,
}

fn append(mut a: JsonValue, mut b: JsonValue) -> JsonValue {
    a.as_object_mut().unwrap().append(b.as_object_mut().unwrap());
    a
}
