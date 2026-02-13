use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, fs, sync::Arc};

//
// ============================
// CONFIG STRUCTS
// ============================
//

type RouteTargets = HashMap<String, Vec<String>>;

#[derive(Clone)]
struct AppState {
    routes: Arc<RouteTargets>,
    client: Client,
}

//
// ============================
// FANOUT HANDLER
// ============================
//

async fn fanout(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let path = req.path().to_string();

    let targets = match data.routes.get(&path) {
        Some(t) => t.clone(),
        None => return HttpResponse::NotFound().body("Route not configured"),
    };

    let headers = req.headers().clone();
    let body = body.clone();
    let client = data.client.clone();

    tokio::spawn(async move {
        for url in targets {
            let client = client.clone();
            let headers = headers.clone();
            let body = body.clone();

            tokio::spawn(async move {
                let mut fwd = client.post(&url).body(body);

                for (name, value) in headers.iter() {
                    if name != "host" && name != "content-length" {
                        fwd = fwd.header(name, value);
                    }
                }

                match fwd.send().await {
                    Ok(_) => println!("➡️ Forwarded to {}", url),
                    Err(e) => eprintln!("❌ Failed to forward to {}: {}", url, e),
                }
            });
        }
    });

    HttpResponse::Ok().body("Forwarded")
}

//
// ============================
// LOAD ROUTES FROM JSON
// ============================
//

fn load_routes_from_file(path: &str) -> RouteTargets {
    let content = fs::read_to_string(path).expect("Failed to read routes.json");

    serde_json::from_str(&content).expect("Invalid routes.json format")
}

//
// ============================
// MAIN
// ============================
//

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let routes = load_routes_from_file("routes.json");

    if routes.is_empty() {
        panic!("routes.json is empty — no routes configured");
    }

    println!("🚦 Loaded routes:");
    for (route, targets) in &routes {
        println!("  {} -> {:?}", route, targets);
    }

    let state = web::Data::new(AppState {
        routes: Arc::new(routes),
        client: Client::new(),
    });

    HttpServer::new(move || {
        let mut app = App::new().app_data(state.clone());

        // Dynamically register all routes
        for route in state.routes.keys() {
            app = app.route(route, web::post().to(fanout));
        }

        app
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
