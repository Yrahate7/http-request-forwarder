use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::task;

#[derive(Clone)]
struct AppState {
    targets: Arc<RwLock<HashMap<String, Vec<String>>>>,
    client: Client,
}

#[derive(Deserialize)]
struct TargetPayload {
    url: String,
}

#[post("/add_target/{id}")]
async fn add_target(
    state: web::Data<AppState>,
    path: web::Path<String>,
    payload: web::Json<TargetPayload>,
) -> impl Responder {
    let id = path.into_inner();
    let mut guard = state.targets.write().unwrap();
    guard
        .entry(id.clone())
        .or_default()
        .push(payload.url.clone());

    HttpResponse::Ok().json(json!({
        "status": "added",
        "id": id,
        "url": payload.url
    }))
}

#[post("/remove_target/{id}")]
async fn remove_target(
    state: web::Data<AppState>,
    path: web::Path<String>,
    payload: web::Json<TargetPayload>,
) -> impl Responder {
    let id = path.into_inner();
    let mut guard = state.targets.write().unwrap();
    if let Some(urls) = guard.get_mut(&id) {
        urls.retain(|u| u != &payload.url);
    }

    HttpResponse::Ok().json(json!({
        "status": "removed",
        "id": id,
        "url": payload.url
    }))
}

#[post("/fanout/{id}/{tail:.*}")]
async fn fanout(
    req: HttpRequest,
    body: Bytes,
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (id, tail) = path.into_inner();
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.to_string(), s.to_string())))
        .collect();

    let body_vec = body.to_vec();
    let method = req.method().clone();

    let urls = {
        let guard = state.targets.read().unwrap();
        guard.get(&id).cloned().unwrap_or_default()
    };

    if urls.is_empty() {
        return HttpResponse::NotFound().json(json!({
            "error": format!("No targets configured for id {}", id)
        }));
    }

    let client = state.client.clone();
    task::spawn(async move {
        for target in urls {
            let url = if tail.is_empty() {
                target.clone()
            } else {
                format!("{}/{}", target.trim_end_matches('/'), tail)
            };

            let mut req_builder = client.request(method.clone(), &url).body(body_vec.clone());
            for (k, v) in &headers {
                req_builder = req_builder.header(k, v);
            }

            match req_builder.send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        println!("Fanout to {} failed with status {}", url, resp.status());
                    }
                }
                Err(e) => {
                    println!("Fanout to {} error: {}", url, e);
                }
            }
        }
    });

    HttpResponse::Ok().json(json!({
        "status": "queued",
        "id": id
    }))
}

#[get("/list_targets/{id}")]
async fn list_targets(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let guard = state.targets.read().unwrap();
    let urls = guard.get(&id).cloned().unwrap_or_default();

    HttpResponse::Ok().json(json!({
        "id": id,
        "targets": urls
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        targets: Arc::new(RwLock::new(HashMap::new())),
        client: Client::new(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(add_target)
            .service(remove_target)
            .service(list_targets)
            .service(fanout)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
