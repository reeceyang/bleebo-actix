use std::pin::Pin;

use actix_web::{
    dev::{Service as _, ServiceRequest},
    get, post,
    test::TestRequest,
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};

use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
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

const BASE_HOST: &str = "bleebo.reeceyang.xyz";

// auth middleware
async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| Pin::new(data).get_ref().clone())
        .unwrap_or_else(Default::default);
    println!("Hi from start. You requested: {}", req.uri());
    println!("{}", req.headers().get("Host").unwrap().to_str().unwrap());
    let host = req.headers().get("Host").unwrap().to_str().unwrap();

    if host == BASE_HOST {
        // TODO: validate credentials
        println!(
            "user_id {} password {:?}",
            credentials.user_id(),
            credentials.password()
        );
        // match auth::validate_token(credentials.token()) {
        //     Ok(res) => {
        //         if res == true {
        //             Ok(req)
        //         } else {
        //             Err(AuthenticationError::from(config).into())
        //         }
        //     }
        //     Err(_) => Err(AuthenticationError::from(config).into()),
        // }
        return Err((AuthenticationError::from(config).into(), req));
    }

    let subdomain = host
        .strip_suffix(BASE_HOST)
        .unwrap()
        .strip_suffix(".")
        .unwrap();

    let new_req = TestRequest::with_uri(format!("/site/{}/{}", subdomain, req.uri()).as_str())
        .to_srv_request();
    Ok(new_req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .wrap(HttpAuthentication::basic(validator))
            .service(actix_files::Files::new("/site", "site/").index_file("index.html"))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
