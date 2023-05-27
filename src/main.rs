use axum::response::Response;
use axum::routing::get;
use axum::{response::IntoResponse, Router};
use http::HeaderMap;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;
use axum::http::StatusCode;
use axum::extract::{Extension, Path};
use rand::Rng;
use rand::distributions::Alphanumeric;

#[tokio::main]
async fn main() {
    let invoice_hashes: State = Arc::new(Mutex::new(HashSet::new()));

    let app = Router::new().route("/", get(index)).layer(axum::Extension(invoice_hashes));

    axum::Server::bind(&"0.0.0.0:81".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Global variable for invoice hashes
type State = Arc<Mutex<HashSet<String>>>;

async fn index(Path(()): Path<()>, state: Extension<State>, headers: HeaderMap) -> impl IntoResponse {
    println!("Hit index...");
    println!("Headers: {:?}", headers);
    if let Some(auth) = headers.get("Authorization") {
        let parts: Vec<_> = auth.to_str().unwrap().split(' ').collect();
        if parts.len() == 2 && parts[0] == "L402" {
            let preimage_parts: Vec<_> = parts[1].split(':').collect();
            if preimage_parts.len() == 2 && preimage_parts[1] == "testpreimage" {
                let mut invoice_hashes = state.lock().await;
                if !invoice_hashes.contains("testpreimage") {
                    invoice_hashes.insert("testpreimage".into());
                    return Response::new("yes".into());
                }
            }
        }
    }

    let resp: Response<String> = axum::response::Response::builder()
        .status(StatusCode::PAYMENT_REQUIRED)
        .header("WWW-Authenticate", format!(r#"L402 token="", invoice=""#))
        .body("Needs payment".into())
        .unwrap();

    resp
}

fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn randomword(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}