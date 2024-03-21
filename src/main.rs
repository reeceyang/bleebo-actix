use actix_web::{
    dev::{Service as _, ServiceRequest},
    get, post,
    test::TestRequest,
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use futures_util::future::FutureExt;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .wrap_fn(|req, srv| {
                println!("Hi from start. You requested: {}", req.uri());
                let new_req =
                    TestRequest::with_uri(format!("/site{}", req.uri()).as_str()).to_srv_request();
                srv.call(new_req).map(|res| {
                    println!("Hi from response");
                    res
                })
            })
            .service(actix_files::Files::new("/site", "site/").index_file("index.html"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
