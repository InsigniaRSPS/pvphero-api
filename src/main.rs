#[macro_use]
extern crate serde_derive;

use std::{convert::Infallible, net::SocketAddr};
use std::sync::{Arc, RwLock};

use hyper::{Body, Request, Response, Server};
use hyper::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper_tls::HttpsConnector;

use settings::Settings;

use crate::item_prices::{fetch_item_prices, listen_for_items_refresh};
use crate::runelite_version::fetch_runelite_version;
use crate::worlds::{fetch_worlds, listen_for_worlds_refresh};

mod settings;
mod item_prices;
mod runelite_version;
mod worlds;

async fn price_api() -> Result<Response<Body>, Infallible> {
    unsafe {
        let item_prices = PRICES_LOCK.as_ref().unwrap().read().unwrap();
        Ok(Response::builder().body(Body::from(format!("{}", item_prices))).unwrap())
    }
}

async fn world_api() -> Result<Response<Body>, Infallible> {
    unsafe {
        let worlds = WORLDS_LOCK.as_ref().unwrap().read().unwrap();
        Ok(Response::builder().body(Body::from(format!("{}", worlds))).unwrap())
    }
}

async fn handle(mut request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let current_path = request.uri_mut().path();

    match current_path {
        "/prices" => price_api().await,
        "/worlds" => world_api().await,
        _ => Ok(Response::builder()
            .body(Body::from("We're online!")).unwrap())
    }
}

static mut PRICES_LOCK: Option<RwLock<String>> = None;
static mut WORLDS_LOCK: Option<RwLock<String>> = None;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = Arc::new(Settings::new());

    let items_settings = settings.clone();
    tokio::spawn(async move {
        let settings = items_settings;
        listen_for_items_refresh(&settings.redis.url, &settings.redis.password, settings.redis.port).await.expect("Item refresh failed!");
    });

    let worlds_settings = settings.clone();
    tokio::spawn(async move {
        let settings = worlds_settings;
        listen_for_worlds_refresh(&settings.redis.url, &settings.redis.password, settings.redis.port).await.expect("World refresh failed!");
    });

    let runelite_version = fetch_runelite_version().await?;

    unsafe {
        PRICES_LOCK = Some(RwLock::new(fetch_item_prices(&runelite_version, &settings.redis.url, &settings.redis.password, settings.redis.port).await.expect("Fetching redis item prices failed!")));
        WORLDS_LOCK = Some(RwLock::new(fetch_worlds(&settings.redis.url, &settings.redis.password, settings.redis.port).expect("Fetching redis worlds failed!")));
    }

    let webserver_address = SocketAddr::from(([127, 0, 0, 1], settings.server.port));

    let make_svc = make_service_fn(move |_conn| {
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(req)
            }))
        }
    });

    let server = Server::bind(&webserver_address).serve(make_svc);

    register_api_server(&settings.redis.url, &settings.redis.password, settings.redis.port).await?;

    println!("Listening on http://{}", webserver_address);


    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}

async fn register_api_server(url: &String, password: &String, port: u16) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(format!("redis://:{}@{}:{}/", password, url, port))?;
    let mut con = client.get_connection()?;
    let ip: String = get_ip().await?;
    redis::cmd("HSET").arg("servers").arg(ip).arg("API_SERVER").query(&mut con)?;
    Ok(())
}

async fn get_ip() -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = "https://checkip.amazonaws.com/".parse().unwrap();
    let mut res = client.get(url).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let ip = String::from_utf8(bytes.to_vec())?.trim_end().parse()?;
    println!("Ip is {}", ip);
    Ok(ip)
}