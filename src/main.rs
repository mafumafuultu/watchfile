use std::{fs, time::Duration};
use actix_web::{
    get, middleware::Logger, rt, web, App, HttpRequest, HttpResponse, HttpServer, Responder
};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
};
use futures_util::StreamExt;
use notify::{Config, RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};

const YML_PATH: &str = "./config.yaml";

#[derive(Deserialize)]
struct AppConfig {
    watch_path: String,
    server: ServerConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    address: String,
    port: u16,
}

#[derive(Serialize)]
struct AppInfo {
    app_name: String,
    version: String,
    repository: String,
}

#[get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body(include_str!("./static/index.html"))
}

#[get("/test")]
async fn test(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body(include_str!("./static/test.html"))
}

#[get("/version")]
async fn version(_req: HttpRequest) -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    let obj = AppInfo {
        app_name: "watchfile".to_string(),
        version: version.to_string(),
        repository: "https://github.com/mafumafuultu/watchfile".to_string(),
    };
    web::Json(obj)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_app_config();
    let address = format!("{}:{}", &config.server.address, config.server.port);

    println!("Listening on {}", address);
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(test)
            .service(version)
            .route("/watch", web::get().to(watch))
    })
    .bind((config.server.address, config.server.port))
    .expect(format!("Can not bind {}", address.as_str()).as_str())
    .run()
    .await
}

async fn watch(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (res, session, stream) = actix_ws::handle(&req, stream).expect("Can not create session");
    let _stream = stream
        .aggregate_continuations()
        .max_continuation_size(8 * 1024 * 1024 * 4);
    let config = get_app_config();
    rt::spawn(async move {
        if let Err(e) = async_watch(config.watch_path, session).await {
            println!("watch error: {:?}", e);
        }
        println!("done");
    });
    Ok(res)
}

async fn async_watch<P: AsRef<std::path::Path>>(
    path: P,
    mut session: actix_ws::Session,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher();

    watcher
        .watch(path.as_ref(), notify::RecursiveMode::NonRecursive)
        .expect("");

    let target_path = path.as_ref();
    if target_path.exists() {
        let mut before_meta = target_path.metadata().unwrap().modified().unwrap();
        while let Some(res) = rx.next().await {
            match res {
                Ok(event) => {
                    let new_meta = target_path.metadata().unwrap().modified().unwrap();
                    if before_meta < new_meta {
                        if let Ok(text) = tokio::fs::read_to_string(event.paths.get(0).unwrap()).await {
                            session.text(markdown::to_html_with_options(
                                &text,
                                &markdown::Options::gfm(),
                            ).expect("markdown error")).await.unwrap();
                        }

                        before_meta = new_meta;
                    }
                }
                Err(err) => println!("watch error: {:?}", err),
            }
        }
    }
    Ok(())
}

fn async_watcher() -> (
    notify::RecommendedWatcher,
    Receiver<Result<notify::Event, notify::Error>>,
) {
    let (mut tx, rx) = channel(16);
    let conf = Config::default().with_poll_interval(Duration::from_secs(1));
    let watcher: notify::RecommendedWatcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(ref _event) = res {
                futures::executor::block_on(async { tx.send(res).await.unwrap() })
            }
        },
        conf,
    )
    .expect("Can not create watcher");

    (watcher, rx)
}


fn get_app_config() -> AppConfig {
    let config_str = fs::read_to_string(YML_PATH).unwrap();
    serde_yaml::from_str(&config_str).expect("Unable to parse config.yaml")
}