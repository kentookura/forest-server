use crate::Broadcaster;
use actix_cors::Cors;
use actix_htmx::HtmxMiddleware;
use actix_web::dev::Server;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web_lab::extract::Path;
use std::sync::Arc;
use tokio;
use tokio::process::Command;

pub mod sse;
pub mod websocket;
use crate::forest::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[get("/events")]
async fn event_stream(broadcaster: web::Data<Broadcaster>) -> impl Responder {
    broadcaster.new_client().await
}

#[post("/broadcast/{msg}")]
async fn broadcast_msg(
    broadcaster: web::Data<Broadcaster>,
    Path((msg,)): Path<(String,)>,
) -> impl Responder {
    broadcaster.broadcast(&msg).await;
    HttpResponse::Ok().body("msg sent")
}

//#[actix_rt::main]
pub async fn server(port: u16) -> Server {
    if !std::path::Path::new("./output").exists() {
        std::fs::create_dir("./output").expect("failed creating output directory");
    }

    let data = Broadcaster::create();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("tauri://localhost")
            .allowed_methods(vec!["GET", "POST"]);

        let generated = generate();

        App::new()
            .app_data(web::Data::from(Arc::clone(&data)))
            .service(event_stream)
            .service(broadcast_msg)
            .wrap(HtmxMiddleware)
            .service(web::resource("/").to(|| async { HomeTemplate {} }))
            .wrap(Logger::default())
            .wrap(cors)
            .service(new_tree)
    })
    .bind(("127.0.0.1", port))
    .expect("Failed to bind addr")
    .run()
}

#[post("/tree")]
async fn new_tree(new: String) -> impl Responder {
    match Command::new("forester")
        .args(&[
            "new",
            "--dir",
            "trees",
            "--prefix",
            new.as_ref(), //.appendChild(new.prefix.into()),
        ])
        .output()
        .await
    {
        Ok(output) => HttpResponse::Created().body(output.stdout),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    };
    HttpResponse::Accepted()
}
