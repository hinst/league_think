use chrono::NaiveDateTime;
use clap::StructOpt;
use std::ops::Add;
use std::collections::HashMap;
use crate::string::*;

const SUMMARY_LIMIT: usize = 6;
const STATISTICAL_SIGNIFICANCE_THRESHOLD: i32 = 4;

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

    fn format_list_of_named(list: &[(&str, &WinRateInfo)], indentation: &str) -> String {
        let mut text = String::new();
        for (champion_name, win_rate_info) in list {
            text = text
                .add(indentation)
                .add(champion_name)
                .add(" ")
                .add(&format_ratio(
                    win_rate_info.count_of_wins,
                    win_rate_info.count_of_matches
                ))
                .add(" of ")
                .add(&win_rate_info.count_of_matches.to_string());
            text.push('\n');
        }
        return text;
    }
}

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
        text.push('\n');

        let mut significant_enemies: Vec<(&str, &WinRateInfo)> = Vec::new();
        for (champion_name, win_rate_info) in &self.win_rates_vs_champions {
            if win_rate_info.count_of_matches >= STATISTICAL_SIGNIFICANCE_THRESHOLD {
                significant_enemies.push((&champion_name, &win_rate_info));
            }
        }
        significant_enemies.sort_by(|a, b|
            a.1.get_win_rate().partial_cmp(&b.1.get_win_rate()).unwrap()
        );
        let enemies = significant_enemies;
        {
            let mut easiest_enemies: Vec<(&str, &WinRateInfo)> = Vec::new();
            for enemy in enemies.iter().rev().take(SUMMARY_LIMIT) {
                easiest_enemies.push(*enemy);
            }
            let easiest_enemies = easiest_enemies;
            text = text.add("easiest enemies: ").add(&easiest_enemies.len().to_string());
            text.push('\n');
            text = text.add(&WinRateInfo::format_list_of_named(&easiest_enemies, INDENTATION_STRING));
        }
        {
            let mut hardest_enemies: Vec<(&str, &WinRateInfo)> = Vec::new();
            for enemy in enemies.iter().take(SUMMARY_LIMIT) {
                hardest_enemies.push(*enemy);
            }
            let hardest_enemies = hardest_enemies;
            text = text.add("worst enemies: ").add(&hardest_enemies.len().to_string());
            text.push('\n');
            text = text.add(&WinRateInfo::format_list_of_named(&hardest_enemies, INDENTATION_STRING));
        }
        return text;
    }
}

struct Analyzer {
    duration_limit: chrono::Duration,
    summoner_id: String,
    champion_infos: HashMap<String, ChampionInfo>,
}

impl Analyzer {
    pub fn new(summoner_id: String) -> Analyzer {
        return Analyzer {
            duration_limit: chrono::Duration::days(0),
            summoner_id,
            champion_infos: HashMap::new()
        }
    }

    pub fn analyze_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.champion_infos.clear();
        let mut files: Vec<std::fs::DirEntry> = std::fs::read_dir("./data")
            .expect("Data directory is required")
            .map(|file_path| file_path.expect("A valid file path is required"))
            .collect();
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()).reverse());
        let mut latest_chronological_date: Option<NaiveDateTime> = None;
        let mut latest_processed_date: Option<NaiveDateTime> = None;
        let mut count_of_processed_files = 0;
        for (i, file_path) in files.iter().enumerate() {
            match latest_chronological_date {
                Some(latest_chronological_date) => {
                    match latest_processed_date {
                        Some(latest_processed_date) => {
                            let duration = latest_chronological_date
                                .signed_duration_since(latest_processed_date);
                            if duration > self.duration_limit {
                                println!("Duration limit reached at {}", latest_processed_date);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            let file_content = std::fs::read_to_string(file_path.path())?;
            let match_history: riven::models::match_v5::Match = serde_json::from_str(&file_content)?;
            let moment = NaiveDateTime::from_timestamp(
                match_history.info.game_creation / 1000,
                (match_history.info.game_creation % 1000) as u32);
            if latest_chronological_date.is_none() {
                latest_chronological_date = Some(moment);
            }
            latest_processed_date = Some(moment);
            if i % 10 == 0 {
                println!("Analyzing file {} -> {}...", i, moment);
            }
            self.add_match_history(&match_history);
            count_of_processed_files += 1;
        };
        println!("Analysis complete. Total files: {}. Processed files: {}", files.len(), count_of_processed_files);
        Ok(())
    }

    fn add_match_history(&mut self, match_history: &riven::models::match_v5::Match) {
        for participant in &match_history.info.participants {
            if participant.summoner_id == self.summoner_id {
                let my_champion = participant.champion_name.clone();
                let champion_info = self.champion_infos.entry(my_champion).or_insert(ChampionInfo::new());
                champion_info.count_of_matches += 1;

                let enemies = find_participants_by_team_id(&match_history.info, participant.team_id, false);
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
                .add(champion_name).add("\n")
                .add(&indent_string(&champion_info.get_summary_text()))
                .add("\n");
        }
        return text;
    }
}

fn find_participants_by_team_id(info: &riven::models::match_v5::Info, team_id: riven::consts::Team, equal: bool)
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

#[derive(clap::Parser)]
struct CommandLineArguments {
    #[clap(short, default_value_t = 300)]
    days: i64
}

pub fn analyze() {
    let summoner_id = std::fs::read_to_string("./summoner-id.txt")
        .expect("summoner-id is required");
    let args = CommandLineArguments::parse_from(std::env::args().skip(1));
    let mut analyzer = Analyzer::new(summoner_id);
    println!("{}", args.days);
    analyzer.duration_limit = chrono::Duration::days(args.days);
    analyzer.analyze_files().unwrap();
    println!("Champion summary:\n{}", analyzer.get_summary_text());
}