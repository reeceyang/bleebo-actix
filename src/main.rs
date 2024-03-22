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

use std::future::{ready, Ready};

use actix_web::dev::{forward_ready, Service, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct HostRoute;

impl HostRoute {
    pub fn new() -> Self {
        HostRoute {}
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for HostRoute
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = HostRouteMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HostRouteMiddleware { service }))
    }
}

pub struct HostRouteMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for HostRouteMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        let host = req.headers().get("Host").unwrap().to_str().unwrap();

        if host == BASE_HOST {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;

                Ok(res)
            });
        }

        let subdomain = host
            .strip_suffix(BASE_HOST)
            .unwrap()
            .strip_suffix(".")
            .unwrap();

        let new_req = TestRequest::with_uri(format!("/site/{}/{}", subdomain, req.uri()).as_str())
            .to_srv_request();

        let fut = self.service.call(new_req);

        Box::pin(async move {
            let res = fut.await?;

            Ok(res)
        })
    }
}

const BASE_HOST: &str = "bleebo.reeceyang.xyz";

// auth middleware
async fn upload(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, Error> {
    let config = req
        .app_data::<Config>()
        .map(|data| Pin::new(data).get_ref().clone())
        .unwrap_or_else(Default::default);
    println!("Hi from start. You requested: {}", req.uri());
    println!("{}", req.headers().get("Host").unwrap().to_str().unwrap());
    let host = req.headers().get("Host").unwrap().to_str().unwrap();

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
    return Err(AuthenticationError::from(config).into());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(HostRoute::new())
            .service(actix_files::Files::new("/site", "site/").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
