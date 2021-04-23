use std::collections::HashMap;

use bytes::Buf as _;
use hyper::Client;
use hyper_tls::HttpsConnector;
use redis::Commands;

use crate::PRICES_LOCK;
use crate::runelite_version::fetch_runelite_version;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemPrice {
    pub id: i32,
    pub name: String,
    pub price: i32,
    #[serde(rename(serialize = "wikiPrice", deserialize = "wikiPrice"))]
    pub wiki_price: i32,
}

impl ItemPrice {
    pub fn update_price(&mut self, multiplier: f64) {
        self.price = std::cmp::max((self.price as f64 * multiplier) as i32, 1);
        self.wiki_price = std::cmp::max((self.wiki_price as f64 * multiplier) as i32, 1);
    }
}

pub async fn fetch_item_prices(runelite_version: &String, redis_url: &String, redis_password: &String, redis_port: u16) -> Result<String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let runelite_url = format!("https://api.runelite.net/runelite-{}/item/prices.js", runelite_version).parse().expect(&*format!("Failed to parse runelite url. runeliteversion={}", runelite_version));
    let res = client.get(runelite_url).await?;
    let body = hyper::body::aggregate(res).await?;
    let mut prices: Vec<ItemPrice> = serde_json::from_reader(body.reader())?;

    apply_custom_item_prices(&mut prices, redis_url, redis_password, redis_port).await?;

    let prices_json = serde_json::to_string(&prices)?;

    Ok(prices_json)
}

async fn apply_custom_item_prices(prices: &mut Vec<ItemPrice>, redis_url: &String, redis_password: &String, redis_port: u16) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(format!("redis://:{}@{}:{}/", redis_password, redis_url, redis_port))?;
    let mut con = client.get_connection()?;
    let item_mapping: HashMap<i32, f64> = con.hgetall("item_prices")?;

    for item in &mut prices.iter_mut() {
        if item_mapping.contains_key(&item.id) {
            let multiplier = item_mapping.get(&item.id).expect(&*format!("Failed to find price mapping for item {}", &item.id));
            let price = item.wiki_price;
            item.update_price(*multiplier);
            println!("Update price for {}-{} from {} to {}", item.id, item.name, price, item.wiki_price);
        }
    }

    Ok(())
}

pub async fn listen_for_items_refresh(url: &String, password: &String, port: u16) -> redis::RedisResult<()> {
    let client = redis::Client::open(format!("redis://:{}@{}:{}/", password, url, port))?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();

    pubsub.subscribe("items_refresh")?;

    println!("Subscribing for item refresh notifications");

    loop {
        let msg = pubsub.get_message()?;
        let payload: String = msg.get_payload()?;
        println!("Refreshing items '{}': {}", msg.get_channel_name(), payload);

        let runelite_version = fetch_runelite_version().await.unwrap();
        let items = fetch_item_prices(&runelite_version, &url, &password, port).await.expect("Failed to fetch redis item prices");

        unsafe {
            let mut item_lock = PRICES_LOCK.as_ref().unwrap().write().expect("Failed to get write lock on PRICES_LOCK");
            *item_lock = items;
        }
    }
}