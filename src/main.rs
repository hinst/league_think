use riven::RiotApi;
use riven::RiotApiConfig;
use riven::consts::RegionalRoute;
use riven::consts::PlatformRoute;
use riven::consts::Queue;

fn main() {
    println!("STARTING...");

    // Enter tokio async runtime.
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        // Create RiotApi instance from key string.
        let api_key = std::fs::read_to_string("./riot-api-key.txt").unwrap();
        let api_key = api_key.trim();
        let riot_api = RiotApi::new(RiotApiConfig::with_key(api_key).preconfig_burst());

        // Get summoner data.
        let summoner = riot_api.summoner_v4()
            .get_by_summoner_name(PlatformRoute::EUW1, "YumaWhen").await
            .expect("Get summoner failed.")
            .expect("There is no summoner with that name.");

        let match_ids = riot_api.match_v5().get_match_ids_by_puuid(
            RegionalRoute::EUROPE,
            summoner.puuid.as_str(),
            Some(100),
            None,
            Some(Queue::SUMMONERS_RIFT_5V5_RANKED_SOLO),
            Some(0),
            Some(0), None)
            .await
            .expect("Match ids");

        println!("Match ids: {:?}", match_ids);
    });
}
