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


const BASE_HOST: &str = ".bleebo.reeceyang.xyz";
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .wrap_fn(|req, srv| {
                println!("Hi from start. You requested: {}", req.uri());
                println!("{}", req.headers().get("Host").unwrap().to_str().unwrap());
                let host = req.headers().get("Host").unwrap().to_str().unwrap().strip_suffix(BASE_HOST).unwrap();

                let new_req = TestRequest::with_uri(format!("/site/{}/{}", host, req.uri()).as_str()).to_srv_request();
                srv.call(new_req)
            })
            .service(actix_files::Files::new("/site", "site/").index_file("index.html"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
