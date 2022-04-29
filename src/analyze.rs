use chrono::NaiveDateTime;
use std::ops::Add;
use std::collections::HashMap;

struct WinRateInfo {
    count_of_wins: i32,
    count_of_matches: i32,
}

impl WinRateInfo {
    pub fn new() -> WinRateInfo {
        return WinRateInfo {
            count_of_wins: 0,
            count_of_matches: 0
        }
    }

    fn get_win_rate(&self) -> f32 {
        if self.count_of_matches > 0 {
            return (self.count_of_wins as f32) / (self.count_of_matches as f32);
        } else {
            return 0.0;
        }
    }
}

const SUMMARY_LIMIT: usize = 5;
const STATISTICAL_SIGNIFICANCE_THRESHOLD: i32 = 8;

struct ChampionInfo {
    count_of_matches: i32,
    win_rates_vs_champions: HashMap<String, WinRateInfo>,
}

impl ChampionInfo {
    pub fn new() -> ChampionInfo {
        return ChampionInfo {
            count_of_matches: 0,
            win_rates_vs_champions: HashMap::new(),
        }
    }

    pub fn get_summary_text(&self) -> String {
        let mut text = String::new();
        text = text
            .add("count of matches: ")
            .add(&self.count_of_matches.to_string());

        let mut enemies: Vec<(&String, &WinRateInfo)> = Vec::new();
        for (champion_name, win_rate_info) in &self.win_rates_vs_champions {
            if win_rate_info.count_of_matches >= STATISTICAL_SIGNIFICANCE_THRESHOLD {
                enemies.push((&champion_name, &win_rate_info));
            }
        }
        enemies.sort_by(|a, b|
            a.1.get_win_rate().partial_cmp(&b.1.get_win_rate()).unwrap()
        );
        let enemies = enemies;

        let mut easiest_enemies: Vec<(&String, &WinRateInfo)> = Vec::new();
        for enemy in enemies.iter().rev().take(SUMMARY_LIMIT) {
            easiest_enemies.push(*enemy);
        }
        let easiest_enemies = easiest_enemies;
        for (champion_name, win_rate_info) in easiest_enemies {
            text = text.add("\n")
                .add(champion_name)
                .add(" ")
                .add(&format_ratio(
                    win_rate_info.count_of_wins,
                    win_rate_info.count_of_matches
                ))
                .add(" of ")
                .add(&win_rate_info.count_of_matches.to_string());
        }
        return text;
    }
}

struct Analyzer {
    summoner_id: String,
    champion_infos: HashMap<String, ChampionInfo>,
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

                let enemies = find_by_team_id(&match_history.info, participant.team_id, false);
                for enemy in enemies {
                    let matchup_info = champion_info.win_rates_vs_champions
                        .entry(enemy.champion_name.clone())
                        .or_insert(WinRateInfo::new());
                    matchup_info.count_of_matches += 1;
                    if participant.win {
                        matchup_info.count_of_wins += 1;
                    }
                }
            }
        }
    }

    fn get_summary_text(&self) -> String {
        let mut champions: Vec<(&String, &ChampionInfo)> = Vec::new();
        for (champion_name, champion_info) in &self.champion_infos {
            champions.push((champion_name, &champion_info));
        }
        champions.sort_by(|a, b|
            a.1.count_of_matches.partial_cmp(&b.1.count_of_matches).unwrap().reverse()
        );
        let mut text = String::new();
        for champion in champions {
            let (champion_name, champion_info) = champion;
            text = text
                .add(champion_name).add(" ")
                .add(&champion_info.get_summary_text())
                .add("\n");
        }
        return text;
    }
}

fn find_by_team_id(info: &riven::models::match_v5::Info, team_id: riven::consts::Team, equal: bool)
        -> Vec<&riven::models::match_v5::Participant> {
    let mut matched_participants = Vec::new();
    for participant in &info.participants {
        let is_matched = equal && team_id == participant.team_id ||
            !equal && team_id != participant.team_id;
        if is_matched {
            matched_participants.push(participant);
        }
    }
    return matched_participants;
}

fn format_ratio(a: i32, b: i32) -> String {
    if b != 0 {
        let ratio = (a as f32) / (b as f32);
        let integer_ratio = (100.0 * ratio) as i32;
        let mut text = String::new();
        text = text.add(&integer_ratio.to_string()).add("%");
        return text;
    } else {
        return String::from("?");
    }
}

pub fn analyze() {
    let summoner_id = std::fs::read_to_string("./summoner-id.txt")
        .expect("summoner-id is required");
    let mut analyzer = Analyzer::new(summoner_id);
    analyzer.analyze_files().unwrap();
    println!("Champion summary\n{}", analyzer.get_summary_text());
}