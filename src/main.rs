use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

type Targets = HashMap<String, Vec<String>>;

#[derive(Deserialize)]
struct TargetRequest {
    url: String,
}

async fn read_targets(file: &str) -> Targets {
    match fs::read_to_string(file) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

async fn write_targets(file: &str, targets: &Targets) {
    if let Ok(json) = serde_json::to_string_pretty(targets) {
        let _ = fs::write(file, json);
    }
}

async fn add_target(
    path: web::Path<String>,
    target: web::Json<TargetRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let mut targets = read_targets(&data.file).await;
    targets.entry(id).or_default().push(target.url.clone());
    write_targets(&data.file, &targets).await;
    HttpResponse::Ok().json(targets)
}

async fn remove_target(
    path: web::Path<String>,
    target: web::Json<TargetRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let mut targets = read_targets(&data.file).await;
    if let Some(urls) = targets.get_mut(&id) {
        urls.retain(|u| u != &target.url);
    }
    write_targets(&data.file, &targets).await;
    HttpResponse::Ok().json(targets)
}

async fn list_targets(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let targets = read_targets(&data.file).await;
    let urls = targets.get(&id).cloned().unwrap_or_default();
    HttpResponse::Ok().json(urls)
}

async fn fanout(
    path: web::Path<(String, String)>,
    body: String,
    data: web::Data<AppState>,
) -> impl Responder {
    let (id, _suffix) = path.into_inner();
    let targets = read_targets(&data.file).await;

    if let Some(urls) = targets.get(&id) {
        for url in urls {
            let url = url.clone();
            let body = body.clone();
            actix_web::rt::spawn(async move {
                if let Err(err) = reqwest::Client::new().post(&url).body(body).send().await {
                    eprintln!("❌ Failed to send to {}: {}", url, err);
                }
            });
        }
    }

    HttpResponse::Ok().body("ok")
}

struct AppState {
    file: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let file = args.get(1).cloned().unwrap_or("targets.json".to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { file: file.clone() }))
            .route("/add_target/{id}", web::post().to(add_target))
            .route("/remove_target/{id}", web::post().to(remove_target))
            .route("/list_targets/{id}", web::get().to(list_targets))
            .route("/fanout/{id}/{suffix:.*}", web::post().to(fanout))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
