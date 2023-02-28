use chrono::prelude::*;
use mysql::prelude::Queryable;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Write, BufRead};
use tokio;

static BASE_URL: &str  = "https://api.worldoftanks.eu/wot/globalmap/eventrating/?application_id=562a644f751f1c85ba948e29080f29e8&front_id=we_2023_bg&event_id=we_2023&limit=100&page_no=";
static CLANS: [&str; 34] = [
    "STALE",
    "PRMKS",
    "APAXE",
    "LBAC",
    "LEAL",
    "IMB",
    "STOPA",
    "RATIO",
    "B_D_L",
    "ESPA2",
    "SC-SQ",
    "FORAJIDOS",
    "ESPA",
    "EXODO",
    "FYH",
    "B-E-A",
    "MOTIN",
    "EXO2",
    "BRIN",
    "FENYX",
    "CONO",
    "TOROS",
    "PA_CA",
    "BV_",
    "PULPO",
    "WOTLS",
    "T_M_C",
    "CNTR",
    "SPAIN",
    "JAMON",
    "-VLK-",
    "SC-SQ",
    "-CLG-",
    "WB1RD",
];

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

#[allow(unused)]
async fn get_clans_id() -> Result<Vec<Clan>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut clans_objects = Vec::with_capacity(CLANS.len());
    for clan_name in CLANS {
        let url = "https://api.worldoftanks.eu/wot/clans/list/?application_id=562a644f751f1c85ba948e29080f29e8&language=es&search=".to_owned() + clan_name;
        let r: Response = serde_json::from_str(&client.get(url).send().await?.text().await?).unwrap();
        for clan in r.data {
            if CLANS.iter().any(|c| *c == clan.tag) {
                clans_objects.push(clan)
            }
        }
    }
    return Ok(clans_objects);
}

/// Obtiene la informacion sobre los clanes en la ultima campaña  
async fn get_event() -> Result<std::collections::HashMap<ClanRating, u32>, Box<dyn Error>> {
    let cliente = reqwest::Client::new();
    let mut ordered_clans = std::collections::HashMap::with_capacity(CLANS.len());
    let mut page = 1;
    let mut count = 100;
    
    let clans_names = read_clans().unwrap();

    while count != 0 {
        let url = BASE_URL.to_string() + &page.to_string();
        let response: FameRating = serde_json::from_str(&cliente.get(url).send().await?.text().await?)?;
        for clan in response.data.into_iter() {
            if clans_names.iter().any(|c| *c == clan.tag) {
                let rank = clan.rank.clone();
                ordered_clans.insert(clan, rank);
            }
        }
        page += 1;
        count = response.meta.count;
    }

    return Ok(ordered_clans);
}

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

//use mysql::prelude::*;
use mysql::Pool;
use mysql::{self, params, PooledConn};

#[tokio::main]
async fn main() {
    /*
    let mut conn = connect();
    update_ranking(
        &mut conn,
        &[ClanRating {
            tag: "BV_".to_owned(),
            name: "Boinas Verdes".to_owned(),
            rank: 830,
            rank_delta: 30,
        }],
    );
    */

    let clans = get_event().await.unwrap();
    let file_name = format!("Clans_{}.txt", Local::now().format("%H-%M_%d_%m"));
    let mut file = std::io::BufWriter::new(std::fs::File::create(file_name).unwrap());
    let mut hash_vec: Vec<(&ClanRating, &u32)> = clans.iter().collect();
    hash_vec.sort_by(|a, b| a.1.cmp(b.1));

    for (i, (clan, rank)) in hash_vec.iter().enumerate() {
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
