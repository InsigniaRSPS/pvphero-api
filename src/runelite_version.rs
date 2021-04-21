use bytes::Buf as _;
use hyper::Client;
use hyper_tls::HttpsConnector;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
struct RuneliteClient {
    version: String,
}

#[derive(Deserialize, Debug)]
struct RuneliteBootstrap {
    client: RuneliteClient,
}

pub async fn fetch_runelite_version() -> Result<String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = "https://static.runelite.net/bootstrap.json".parse().unwrap();
    let res = client.get(url).await?;
    let body = hyper::body::aggregate(res).await?;
    let bootstrap: RuneliteBootstrap = serde_json::from_reader(body.reader())?;

    println!("Runelite version {}", bootstrap.client.version);

    Ok(bootstrap.client.version)
}