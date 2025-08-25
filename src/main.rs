use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use reqwest::Client;
use std::{fs, sync::Arc};

struct AppState {
    targets: Arc<Vec<String>>,
    client: Client,
}

async fn fanout(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> HttpResponse {
    let client = data.client.clone();
    let targets = data.targets.clone();
    let headers = req.headers().clone();
    let body = body.clone();

    tokio::spawn(async move {
        for url in targets.iter() {
            let client = client.clone();
            let headers = headers.clone();
            let body = body.clone();
            let url = url.clone();

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

    // respond immediately
    HttpResponse::Ok().body("Forwarded")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load targets.txt
    let contents = fs::read_to_string("targets.txt").unwrap_or_default();
    let targets: Vec<String> = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    println!("✅ Loaded targets: {:?}", targets);

    let state = web::Data::new(AppState {
        targets: Arc::new(targets),
        client: Client::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/fanout", web::post().to(fanout))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
