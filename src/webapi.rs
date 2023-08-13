use crate::svc::*;
use actix_web::{self, get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};


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
    query: web::Query<Query>, /*, urlargs: web::Path<String> */
) -> impl Responder {
    let query: Query = match query.0.try_into() {
        Ok(q) => q,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
            "status": "bad request",
            "message": e.to_string(),
            }));
        }
    };

    let tasks = req.app_data::<Tasks>().unwrap();
    let (req, res) = Request::new(query);
    let res = match tasks.submit(Task::GetView(req)) {
        Ok(_) => res.recv().await,
        Err(e) => Err(e),
    };
    match res {
        Ok(view) => HttpResponse::Ok().json(json!({
            "status": "ok",
            "view": view,
        })),
        Err(e) => {
            log::error!("internal server error: {}", e);
            HttpResponse::InternalServerError().json(json!({
            "status": "internal server error",
            "message": "something went wrong on our side, please try again later",
            }))
        }
    }
}

pub struct Server{
    port: u16,
    back: Tasks,
}
impl Server {
    pub fn new(port: u16, back: Tasks) -> Self {
        Self{port, back}
    }

    pub fn run(self) -> Result<()> {
        let run = async move { self.run_async().await };
    
        if let Ok(rth) = tokio::runtime::Handle::try_current() {
            // have runtime, will run in it
            rth.block_on(run)
        } else {
            // neet to create a new runtime
            tokio::runtime::Runtime::new()
                .expect("tokio runtime creation should not fail")
                .handle()
                .block_on(run)
        }
    }
    
    async fn run_async(self) -> Result<()> {
        HttpServer::new(move || {
            App::new()
                .app_data(self.back.clone())
                .wrap(actix_web::middleware::Logger::new("%a %r %s"))
                // .service(connect)
                .service(get_view)
                .service(get_index)
        })
        .bind(("0.0.0.0", self.port))
        .map_err(|e| Error::OtherWithContext("failed to bind web server", e.to_string()))?
        .run()
        .await
        .map_err(|e| Error::OtherWithContext("failed to run web server", e.to_string()))
    }
    
}

