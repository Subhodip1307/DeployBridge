use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use std::{collections::HashMap, net::SocketAddr};
use tokio::process::Command;
mod structures;
use sha2::{Digest, Sha256};
use structures::{Bashinfo, Config, DockerPayload, Token};
use tokio::fs;
use tokio::sync::{OnceCell, mpsc};
use tokio::time::{Duration, sleep};

static PROJECTS: OnceCell<HashMap<String, Bashinfo>> = OnceCell::const_new();
static DEV: OnceCell<String> = OnceCell::const_new();
static ORG: OnceCell<String> = OnceCell::const_new();
static TAG: OnceCell<String> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    println!("Runing Version {}", env!("CARGO_PKG_VERSION"));
    // setting values to the variables
    set_values().await;
    let (tx, rx) = mpsc::channel::<String>(10);
    tokio::spawn(async move {
        worker(rx).await;
    });
    let app = Router::new()
        .route("/", get(|| async { "runing" }))
        .route(
            &format!("/{}", option_env!("url_path").unwrap_or("send")),
            post(docker_view),
        )
        .with_state(tx);
    let addr: SocketAddr = "127.0.0.14:8000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn docker_view(
    Query(params): Query<Token>,
    State(sender): State<mpsc::Sender<String>>,
    Json(data): Json<DockerPayload>,
) -> StatusCode {
    println!("data is {:?}", data);
    let __project = match PROJECTS.get() {
        Some(v) => v,
        None => &HashMap::new(),
    };

    if data.is_deployble()
        && __project.contains_key(&params.token)
        && data.check_repo_name(__project[&params.token].get_repo())
    {
        match sender.send(params.token).await {
            Ok(_) => return StatusCode::CREATED,
            Err(_) => return StatusCode::INSUFFICIENT_STORAGE,
        }
    }
    StatusCode::OK
}

async fn worker(mut rece: mpsc::Receiver<String>) {
    let __project = match PROJECTS.get() {
        Some(v) => v,
        None => &HashMap::new(),
    };

    let timeout = Duration::from_secs(60);

    while let Some(t) = rece.recv().await {
        let item = &__project[t.as_str()];
        if let Ok(content) = fs::read(item.get_path()).await {
            // TODO: check the file size before genating hash
            let hex = hex::encode(Sha256::digest(&content));
            if hex != item.shahash {
                println!("hash Matching failed");
                continue;
            }
            // executing the bash
            tokio::select! {
                result = Command::new("bash").arg("script.sh").output() => {
                    let output = result.expect("failed to run script");
                    println!("status: {}", output.status);
                    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                _ = sleep(timeout) => {
                    println!("script killed: exceeded time limit");
                }
            }
        };
    }
}

async fn set_values() {
    #[cfg(debug_assertions)]
    let read_file = tokio::fs::read_to_string("config.toml").await;
    #[cfg(not(debug_assertions))]
    let read_file = tokio::fs::read_to_string("/etc/deploy_bridge/config.toml").await;

    let file_data = match read_file {
        Ok(v) => v,
        Err(e) => {
            println!("erro is {}", e);
            return;
        }
    };

    let load_config: Result<Config, toml::de::Error> = toml::from_str(&file_data);

    match load_config {
        Err(e) => {
            println!("Config File Not found {}", e);
        }
        Ok(v) => {
            println!("setting value");
            let _ = DEV.set(v.main.dev);
            let _ = ORG.set(v.main.org);
            let _ = TAG.set(v.main.tag);
            // now setting hashmap
            let e = PROJECTS.set(v.projects.unwrap_or_default());
            if let Err(err) = e { println!("err is {:?}", err) }
        }
    }
}
