use chrono::prelude::*;
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Write, BufRead};
use tokio;

static BASE_URL: &str  = "https://api.worldoftanks.eu/wot/globalmap/eventrating/?application_id=562a644f751f1c85ba948e29080f29e8&front_id=we_2023_bg&event_id=we_2023&limit=100&page_no=";


/// Estructura que contiene los valores importantes de cada clan
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
struct ClanRating {
    tag: String,
    name: String,
    rank: u32,
    rank_delta: i32,
}

/// Estructura que contiene los datos importantes de la request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestData {
    count: usize,
}

/// Estructura con los campos que devuelve la request a la API
/// https://api.worldoftanks.eu/wot/globalmap/eventaccountratings/?application_id=
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FameRating {
    meta: RequestData,
    data: Vec<ClanRating>,
}

/// Estructura que contiene los datos de un clan
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
struct Clan {
    name: String,
    tag: String,
    clan_id: u64,
}

/// Estructura con los campos que devuelve la API
/// https://api.worldoftanks.eu/wot/clans/list/?application_id=
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
    data: Vec<Clan>,
}

/*
#[allow(unused)]
async fn get_clans_id(clans: &[&str]) -> Result<Vec<Clan>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut clans_objects = Vec::with_capacity(clans.len());
    for clan_name in clans {
        let url = "https://api.worldoftanks.eu/wot/clans/list/?application_id=562a644f751f1c85ba948e29080f29e8&language=es&search=".to_owned() + clan_name;
        let r: Response = serde_json::from_str(&client.get(url).send().await?.text().await?).unwrap();
        for clan in r.data {
            if clans.iter().any(|c| *c == clan.tag) {
                clans_objects.push(clan)
            }
        }
    }
    return Ok(clans_objects);
}
*/

/// Obtiene los clanes del archivo "Listado.txt"
fn read_clans() -> Result<Vec<String>, std::io::Error> {
    let mut clanes = Vec::new();
    let mut reader = std::io::BufReader::new(std::fs::File::open("Listado.txt")?);
    let mut line = String::new();

    reader.read_line(&mut line).ok();
    while !line.is_empty() {
        clanes.push(line.trim().to_string());
        line.clear();
        reader.read_line(&mut line).ok();
    }


    Ok(clanes)
}



async fn get_parsed_data(cliente: &reqwest::Client, url: &str) -> Result<FameRating, Box<dyn Error>>{
     let a = serde_json::from_str::<FameRating>(&cliente.get(url).send().await?.text().await?)?;
     Ok(a)
}

/// Obtiene la informacion sobre los clanes en la ultima campaña  
async fn get_event() -> Result<Vec<(ClanRating, u32)>, Box<dyn Error>> {

    let cliente = reqwest::Client::new();   // Estructura que se encarga de hacer las request a la
                                            // API
    let mut page = 1;
    let mut count = 100; 
    let clans_names = read_clans().expect("No ha sido posible leer el archivo \"Listado.txt\"");
    let mut clans_tuple = Vec::with_capacity(clans_names.len());

    // Cuando la API devuelve una request con count == 0 
    // significa que ya no hay mas clanes.
    while count != 0 {

        let url = BASE_URL.to_string() + &page.to_string();
        let mut max_tries = 3;
        let mut response: Option<FameRating> = None;

        // Intenta 3 veces parsear la respuesta de la API
        // Hay veces que la request no es exitosa y la API no manda la
        // respuesta con la estructura adecuada o simplemente no la manda.
        while max_tries > 0 && !response.is_some() {
            response = match get_parsed_data(&cliente, &url).await {
                Ok(r) => Some(r),
                Err(_) => {
                    max_tries -= 1;
                    None
                }
            }
        }

        let response = response.expect("Tras 3 intentos ha sido imposible obtener respuesta por parte de la API de wargaming");

        for clan in response.data.into_iter() {
            if clans_names.iter().any(|c| *c == clan.tag) {
                let rank = clan.rank.clone();
                clans_tuple.push((clan, rank))
            }
        }

        page += 1;
        count = response.meta.count;
    }

    return Ok(clans_tuple);
}

#[tokio::main]
async fn main() { 
    let mut clans = get_event().await.unwrap();
    let file_name = format!("Clans_{}.txt", Local::now().format("%H-%M_%d_%m"));
    let mut file = std::io::BufWriter::new(std::fs::File::create(file_name).unwrap());
    clans.sort_by(|a, b| a.1.cmp(&b.1));

    for (i, (clan, rank)) in clans.iter().enumerate() {
        let delta = {
            if clan.rank_delta > 0 {
                format!("^{:>2}", clan.rank_delta.abs())
            } else if clan.rank_delta < 0 {
                format!("v{:>2}", clan.rank_delta.abs())
            } else {
                "- ".to_owned()
            }
        };
        let a = format!("{:>2}    {} {}: {}\n", i + 1, delta, clan.name, rank);
        print!("{a}");
        file.write_all(a.as_bytes()).ok();
    }
}
