use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::LazyLock;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc;
mod structures;
use sha2::{Digest, Sha256};
use structures::{Bashinfo, DockerPayload, Token};
use tokio::fs;
static PROJECTS: LazyLock<HashMap<&str, Bashinfo>> = LazyLock::new(|| {
    HashMap::from([(
        "",
        Bashinfo {
            path: "",
            shahash: "",
            project: "",
        },
    )])
});
use std::process::Command;
const DEV: &str = "";
const ORG: &str = "";
const TAG: &str = "";

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<String>(10);
    tokio::spawn(async move {
        worker(rx).await;
    });
    let app = Router::new()
        .route("/", get(|| async { "runing" }))
        .route("/send", post(docker_view))
        .with_state(tx);
    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn docker_view(
    Query(params): Query<Token>,
    State(sender): State<mpsc::Sender<String>>,
    Json(data): Json<DockerPayload>,
) -> StatusCode {
    println!("data is {:?}", data);

    if data.is_deployble()
        && PROJECTS.contains_key(&params.token.as_str())
        && data.check_repo_name(PROJECTS[&params.token.as_str()].project)
    {
        match sender.send(params.token).await {
            Ok(_) => return StatusCode::CREATED,
            Err(_) => return StatusCode::INSUFFICIENT_STORAGE,
        }
    }
    StatusCode::OK
}

async fn worker(mut rece: mpsc::Receiver<String>) {
    while let Some(t) = rece.recv().await {
        let item = &PROJECTS[t.as_str()];
        if let Ok(content) = fs::read(item.path).await {
            let hex = hex::encode(Sha256::digest(&content));
            if hex != item.shahash {
                println!("hash Matching failed");
                continue;
            }
            // executing the bash
            let output = Command::new("bash")
                .arg(item.path) // path to your .sh file
                .output() // runs it, waits, captures stdout/stderr
                .expect("failed to run script");
            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        };
    }
}
