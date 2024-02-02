use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use futures::future::join_all;
use lazy_static::lazy_static;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use reqwest::Client;
use std::{
    env, fs,
    path::Path,
    str::FromStr,
    time::{Duration, SystemTime},
};

lazy_static! {
    // initialze configuration
    static ref CONFIG: ServerConfig = {
        ServerConfig {
            address: env::var("MERGER_ADDRESS").unwrap_or("0.0.0.0".to_owned()),
            port: env::var("MERGER_PORT")
                .map(|p| p.parse::<u16>().unwrap_or(8989))
                .unwrap_or(8989),
            uris: env::var("MERGER_URLS")
                .map(|rules| {
                    rules
                        .split_whitespace()
                        .into_iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                })
                .unwrap_or(Vec::new()),
            timeout: env::var("MERGER_TIMEOUT")
                .map(|p| p.parse::<u64>().unwrap_or(3))
                .unwrap_or(3),
            level: env::var("MERGER_LOG_LEVEL").unwrap_or("INFO".to_owned()),
        }
    };
}

type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct ServerConfig {
    address: String,
    port: u16,
    uris: Vec<String>,
    timeout: u64,
    level: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    __init_log().unwrap();

    // start http server
    HttpServer::new(|| App::new().service(export_metrics))
        .bind((CONFIG.address.as_str(), CONFIG.port))?
        .run()
        .await
}

fn __init_log() -> AppResult<()> {
    let pattern = "{d(%Y-%m-%dT%H:%M:%S)} {h({l})} [{t}:{L}] {m}{n}";
    let level = log::LevelFilter::from_str(&CONFIG.level).unwrap_or(log::LevelFilter::Info);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();

    Ok(())
}

#[get("/{all:.*}")]
async fn export_metrics() -> impl Responder {
    let time = SystemTime::now();
    let mut tasks = Vec::new();

    // create task
    for item_uris in &CONFIG.uris {
        tasks.push(fetch_metrics(item_uris));
    }

    // wait for all tasks result
    let results = join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap_or("".to_owned()))
        .filter(|c| c.len() > 0)
        .collect::<Vec<String>>();

    log::debug!("export time: {:?}", time.elapsed().unwrap().as_millis());
    HttpResponse::Ok().body(results.join("\n"))
}

async fn fetch_metrics(uris: &str) -> AppResult<String> {
    log::debug!("access: {}", &uris);
    for uri in uris.trim().split(",") {
        if uri.starts_with("http://") || uri.starts_with("https://") {
            if let Ok(response) = fetch_http(&uri).await {
                return Ok(response);
            }
        } else if uri.starts_with("file://") {
            if let Ok(text) = read_file(&uri["file://".len()..]).await {
                return Ok(text);
            }
        }
    }

    Ok(String::new())
}

async fn read_file(file: &str) -> AppResult<String> {
    log::debug!("read file: {}", file);
    if Path::new(file).exists() {
        Ok(fs::read_to_string(file)?)
    } else {
        Ok("".to_owned())
    }
}

async fn fetch_http(uri: &str) -> AppResult<String> {
    log::debug!("fetch: {}", uri);
    let client: Client = Client::builder()
        .timeout(Duration::from_secs(CONFIG.timeout))
        .build()?;
    Ok(client.get(uri).send().await?.text().await?)
}
