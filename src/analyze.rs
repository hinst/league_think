use chrono::NaiveDateTime;
use std::ops::Add;
use std::collections::HashMap;

struct Analyzer {
    summoner_id: String,
    champion_infos: HashMap<String, ChampionInfo>,
}

struct WinRateInfo {
    count_of_wins: i32,
    count_of_matches: i32,
}

struct ChampionInfo {
    count_of_matches: i32,
    win_rates_vs_champions: HashMap<String, WinRateInfo>,
}

impl ChampionInfo {
    pub fn new() -> ChampionInfo {
        ChampionInfo {
            count_of_matches: 0,
            win_rates_vs_champions: HashMap::new(),
        }
    }
}

impl Analyzer {
    pub fn new(summoner_id: String) -> Analyzer {
        return Analyzer {
            summoner_id,
            champion_infos: HashMap::new()
        }
    }

    pub fn analyze_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.champion_infos.clear();
        let file_paths = std::fs::read_dir("./data").expect("Data directory is required");
        let mut file_count = 0;
        for file_path in file_paths {
            let file_path = file_path.expect("A valid file path is required");
            let file_content = std::fs::read_to_string(file_path.path())?;
            let match_history: riven::models::match_v5::Match = serde_json::from_str(&file_content)?;
            let moment = NaiveDateTime::from_timestamp(
                match_history.info.game_creation / 1000,
                (match_history.info.game_creation % 1000) as u32);
            println!("Analyzing file {} -> {}...", file_count, moment);
            self.add_match_history(&match_history);
            file_count += 1;
        }
        println!("Analysis is now finished; there were this many files: {}", file_count);
        Ok(())
    }

    fn add_match_history(&mut self, match_history: &riven::models::match_v5::Match) {
        for participant in &match_history.info.participants {
            if participant.summoner_id == self.summoner_id {
                let my_champion = participant.champion_name.clone();
                let champion_info = self.champion_infos.entry(my_champion).or_insert(ChampionInfo::new());
                champion_info.count_of_matches += 1;
            }
        }
    }

    fn get_summary_text(&self) -> String {
        let mut text = String::new();
        for (champion_name, champion_info) in &self.champion_infos {
            text = text
                .add(champion_name).add(" ")
                .add(champion_info.count_of_matches.to_string().as_str())
                .add("\n");
        }
        return text;
    }
}

pub fn analyze() {
    let summoner_id = std::fs::read_to_string("./summoner-id.txt")
        .expect("summoner-id is required");
    let mut analyzer = Analyzer::new(summoner_id);
    analyzer.analyze_files().unwrap();
    println!("Champion summary\n{}", analyzer.get_summary_text());
}