use std::path::Path;
use serde_json;
use riven::RiotApi;
use riven::RiotApiConfig;

const REGIONAL_ROUTE: riven::consts::RegionalRoute = riven::consts::RegionalRoute::EUROPE;
const PAGE_SIZE: i32 = 100;

struct Reader {
    riot_api: riven::RiotApi,
    summoner: Option<Box<riven::models::summoner_v4::Summoner>>,
    match_ids: Vec<String>,
}

impl Reader {
    pub fn new(api_key: &str) -> Reader {
        Reader {
            riot_api: RiotApi::new(RiotApiConfig::with_key(api_key.trim()).preconfig_burst()),
            summoner: None,
            match_ids: Vec::new(),
        }
    }

    pub async fn read(&mut self) {
        self.read_summoner().await;
        self.read_match_ids().await;
        self.read_match_history().await;
    }

    async fn read_summoner(&mut self) {
        let summoner = self.riot_api.summoner_v4()
            .get_by_summoner_name(riven::consts::PlatformRoute::EUW1, "YumaWhen").await
            .expect("Read summoner info")
            .expect("Find an existing summoner");
        self.summoner = Some(Box::new(summoner));
    }

    async fn read_match_ids(&mut self) {
        let summoner = self.summoner.as_ref().expect("summoner is required");
        self.match_ids.clear();
        let mut offset: i32 = 0;
        while offset >= 0 {
            let match_ids = self.riot_api.match_v5()
                .get_match_ids_by_puuid(
                    REGIONAL_ROUTE,
                    summoner.puuid.as_str(),
                    Some(PAGE_SIZE),
                    None,
                    Some(riven::consts::Queue::SUMMONERS_RIFT_5V5_RANKED_SOLO),
                    Some(0),
                    Some(offset),
                    None
                ).await
                .expect("Match ids");
            offset = if match_ids.len() >= (PAGE_SIZE as usize) {
                offset + PAGE_SIZE
            } else {
                -1
            };
            self.match_ids.extend(match_ids);
        };
        println!("Match ids found: {}", self.match_ids.len());
    }

    async fn read_match_history(&self) {
        for (i, match_id) in self.match_ids.iter().enumerate() {
            let file_path = self.get_match_history_file_path(match_id);
            if !Path::new(&file_path).exists() {
                println!("Saving match history {} of {}", i, self.match_ids.len());
                let match_history = self.riot_api.match_v5().get_match(REGIONAL_ROUTE, match_id).await
                    .expect("Read match history")
                    .expect("Find match history");
                let match_history_json = serde_json::to_string_pretty(&match_history)
                    .expect("Serialize match history");
                std::fs::write(file_path, match_history_json)
                    .expect("Save json to file");
            }
        }
        println!("Saved match history [{}]", self.match_ids.len());
    }

    fn get_match_history_file_path<'a>(&self, id: &str) -> String {
        String::from("./data/") + id + ".json"
    }
}

pub fn store() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let api_key = std::fs::read_to_string("./riot-api-key.txt").unwrap();
        let mut reader = Reader::new(&api_key);
        reader.read().await;
    });
}
