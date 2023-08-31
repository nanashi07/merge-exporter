use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use futures::future::join_all;
use lazy_static::lazy_static;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

lazy_static! {
    static ref CONFIG: ServerConfig = {
        ServerConfig {
            address: std::env::var("MERGER_ADDRESS").unwrap_or("0.0.0.0".to_owned()),
            port: std::env::var("MERGER_PORT")
                .map(|p| p.parse::<u16>().unwrap_or(8989))
                .unwrap_or(8989),
            uri: std::env::var("MERGER_URLS")
                .map(|rules| {
                    rules
                        .split_whitespace()
                        .into_iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                })
                .unwrap_or(Vec::new()),
        }
    };
}

type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct ServerConfig {
    address: String,
    port: u16,
    uri: Vec<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    __init_log().unwrap();

    HttpServer::new(|| App::new().service(export_metrics))
        .bind((CONFIG.address.as_str(), CONFIG.port))?
        .run()
        .await
}

fn __init_log() -> AppResult<()> {
    let pattern = "{d(%Y-%m-%dT%H:%M:%S)} {h({l})} [{t}:{L}] {m}{n}";

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();

    Ok(())
}

#[get("/{all:.*}")]
async fn export_metrics() -> impl Responder {
    let time = std::time::SystemTime::now();
    let mut tasks = Vec::new();

    for uri in &CONFIG.uri {
        log::debug!("access: {}", &uri);
        if uri.starts_with("http://") || uri.starts_with("https://") {
            tasks.push(fetch_metrics(&uri, false));
        } else if uri.starts_with("file://") {
            tasks.push(fetch_metrics(&uri["file://".len()..], true))
        }
    }

    let results = join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap_or("".to_owned()))
        .collect::<Vec<String>>();

    log::debug!("export time: {:?}", time.elapsed().unwrap().as_millis());
    HttpResponse::Ok().body(results.join("\n"))
}

async fn fetch_metrics(uri: &str, local: bool) -> AppResult<String> {
    if local {
        read_file(uri).await
    } else {
        fetch_http(uri).await
    }
}

async fn read_file(file: &str) -> AppResult<String> {
    log::debug!("read file: {}", file);
    if std::path::Path::new(file).exists() {
        Ok(std::fs::read_to_string(file)?)
    } else {
        Ok("".to_owned())
    }
}

async fn fetch_http(uri: &str) -> AppResult<String> {
    log::debug!("fetch: {}", uri);
    Ok(reqwest::get(uri).await?.text().await?)
}
