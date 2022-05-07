use std::collections::HashMap;
use chrono::NaiveDateTime;
use clap::StructOpt;
use std::ops::Add;
use crate::champion_info::ChampionInfo;
use crate::string::{indent_string, format_percent};

const STATISTICAL_SATURATION_THRESHOLD: i32 = 12;

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
                        },  _ => {}
                    }
                },  _ => {}
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

                let allies = find_participants_by_team_id(&match_history.info, participant.team_id, true);
                for ally in allies {
                    let win_rate_info = champion_info.get_win_rate_with(&ally.champion_name);
                    if win_rate_info.get_count_of_matches() < STATISTICAL_SATURATION_THRESHOLD {
                        win_rate_info.add(participant.win);
                    }
                }
                let enemies = find_participants_by_team_id(&match_history.info, participant.team_id, false);
                for enemy in enemies {
                    let win_rate_info = champion_info.get_win_rate_vs(&enemy.champion_name);
                    if win_rate_info.get_count_of_matches() < STATISTICAL_SATURATION_THRESHOLD {
                        win_rate_info.add(participant.win);
                    }
                }
            }
        }
    }

    fn get_sorted_champions(&self) -> Vec<(&String, &ChampionInfo)> {
        let mut champions: Vec<(&String, &ChampionInfo)> = Vec::new();
        for (champion_name, champion_info) in &self.champion_infos {
            champions.push((champion_name, &champion_info));
        }
        champions.sort_by(|a, b|
            a.1.count_of_matches.partial_cmp(&b.1.count_of_matches).unwrap().reverse()
        );
        return champions;
    }

    fn get_summary_text(&self) -> String {
        let champions = self.get_sorted_champions();
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

    fn get_score_summary_text(&self, allies: Vec<&str>, enemies: Vec<&str>) -> String {
        let mut text = String::new();
        let champions = self.get_sorted_champions();
        for (champion_name, champion_info) in &champions {
            let mut ally_count: i32 = 0;
            let mut ally_score: f32 = 0.0;
            for info in champion_info.get_win_rates_with_champions() {
                if allies.contains(&info.0.as_str()) {
                    ally_count += 1;
                    ally_score += info.1.get_win_rate();
                }
            }
            let mut enemy_count: i32 = 0;
            let mut enemy_score: f32 = 0.0;
            for info in champion_info.get_win_rates_vs_champions() {
                if enemies.contains(&info.0.as_str()) {
                    enemy_count += 1;
                    enemy_score += info.1.get_win_rate();
                }
            }
            text = text.add(
                &format!("{}: ally strength {}, enemy weakness {}, summary chance {}",
                    champion_name,
                    Self::format_score(ally_count, ally_score),
                    Self::format_score(enemy_count, enemy_score),
                    Self::format_score(ally_count + enemy_count, ally_score + enemy_score)
                )
            );
            text.push('\n');
        };
        return text;
    }

    fn format_score(champion_count: i32, score: f32) -> String {
        if champion_count == 0 {
            return String::from("[?]")
        } else {
            let mut text = String::new();
            text = text.add(&format_percent(score / (champion_count as f32)));
            text = text.add(" of ");
            text = text.add(&champion_count.to_string());
            return text;
        }
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
    days: i64,

    #[clap(long, default_value_t = String::from(""))]
    allies: String,

    #[clap(long, default_value_t = String::from(""))]
    enemies: String,
}

pub fn analyze() {
    let summoner_id = std::fs::read_to_string("./summoner-id.txt")
        .expect("summoner-id is required");
    let args = CommandLineArguments::parse_from(std::env::args().skip(1));
    let mut analyzer = Analyzer::new(summoner_id);
    println!("{}", args.days);
    analyzer.duration_limit = chrono::Duration::days(args.days);
    analyzer.analyze_files().unwrap();

    if args.allies.len() > 0 || args.enemies.len() > 0 {
        let allies: Vec<&str> = args.allies.split(',').collect();
        let enemies: Vec<&str> = args.enemies.split(',').collect();
        println!("Champion chances:\n{}", analyzer.get_score_summary_text(allies, enemies));
    } else {
        println!("Champion summary:\n{}", analyzer.get_summary_text());
    }
}