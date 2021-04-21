#[derive(Serialize, Deserialize, Debug)]
pub struct World {
    pub id: i32,
    pub types: Vec<String>,
    pub address: String,
    pub activity: String,
    pub location: i32,
    pub players: i32,
}

#[derive(Serialize, Debug)]
pub struct Worlds {
    pub worlds: Vec<World>
}

pub fn fetch_worlds(redis_url: &String, redis_password: &String, redis_port: u16) -> redis::RedisResult<String> {
    let client = redis::Client::open(format!("redis://:{}@{}:{}/", redis_password, redis_url, redis_port))?;
    let mut con = client.get_connection()?;
    let json_worlds: Vec<String> = redis::cmd("SMEMBERS").arg("worlds").query(&mut con)?;

    let mut all_worlds = Worlds {
        worlds: vec![]
    };

    for json_world in json_worlds.iter() {
        let world: World = serde_json::from_str(json_world).unwrap();
        all_worlds.worlds.push(world);
    }

    let worlds_json = serde_json::to_string(&all_worlds).unwrap();

    println!("{}", worlds_json);

    Ok(worlds_json)
}