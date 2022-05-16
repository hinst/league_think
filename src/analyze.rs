use std::collections::{HashMap, HashSet};
use chrono::NaiveDateTime;
use clap::StructOpt;
use std::ops::Add;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::champion_info::ChampionInfo;
use crate::string::{ indent_string, format_percent, INDENTATION_STRING };
use crate::win_rate_info::WinRateInfo;

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
            let (matched_ally_count, ally_score, ally_breakdown_text) = Self::get_win_chance_summary(
                champion_info.get_win_rates_with_champions(), &allies, 2
            );
            let (matched_enemy_count, enemy_score, enemy_breakdown_text) = Self::get_win_chance_summary(
                champion_info.get_win_rates_vs_champions(), &enemies, 2
            );
            text.push_str(
                format!("{}: ally strength {}, enemy weakness {}, summary chance {}",
                    champion_name,
                    Self::format_chance(matched_ally_count, ally_score),
                    Self::format_chance(matched_enemy_count, enemy_score),
                    Self::format_chance(
                        matched_ally_count + matched_enemy_count,
                        ally_score + enemy_score
                    )
                ).as_str()
            );
            text.push('\n');
            text.push_str(INDENTATION_STRING);
            text.push_str("Allies:\n");
            text.push_str(ally_breakdown_text.as_str());
            text.push_str(INDENTATION_STRING);
            text.push_str("Enemies:\n");
            text.push_str(enemy_breakdown_text.as_str());
        };
        return text;
    }

    fn get_win_chance_summary(champion_infos: &HashMap<String, WinRateInfo>, champions: &Vec<&str>, 
            indentation_level: i32) -> (i32, f32, String) {
        let mut matched_count: i32 = 0;
        let mut combined_score: f32 = 0.0;
        let mut breakdown_text = String::new();
        for info in champion_infos {
            if champions.contains(&info.0.as_str()) {
                matched_count += 1;
                combined_score += info.1.get_win_rate();
                for _ in 0..indentation_level {
                    breakdown_text.push_str(INDENTATION_STRING);
                }
                breakdown_text.push_str(info.0);
                breakdown_text.push_str(" ");
                breakdown_text.push_str(info.1.to_string().as_str());
                breakdown_text.push('\n');
            }
        }
        return (matched_count, combined_score, breakdown_text);
    }

    fn format_chance(champion_count: i32, combined_score: f32) -> String {
        if champion_count == 0 {
            return String::from("[?]")
        } else {
            let mut text = String::new();
            text = text.add(&format_percent(combined_score / (champion_count as f32)));
            text = text.add(" of ");
            text = text.add(&champion_count.to_string());
            return text;
        }
    }

    fn get_all_champion_names(&self) -> Vec<String> {
        let mut name_set: HashSet<String> = HashSet::new();
        for (champion_name, info) in &self.champion_infos {
            name_set.insert(champion_name.clone());
            for (champion_name, _) in info.get_win_rates_vs_champions() {
                name_set.insert(champion_name.clone());
            }
            for (champion_name, _) in info.get_win_rates_with_champions() {
                name_set.insert(champion_name.clone());
            }
        }
        let mut names: Vec<String> = Vec::with_capacity(name_set.len());
        for name in name_set {
            names.push(name);
        }
        return names;
    }

    fn guess_champion_names(&self, names: Vec<&str>) -> Vec<String> {
        let matcher = SkimMatcherV2::default();
        let mut corrected_names: Vec<String> = Vec::new();
        let champion_names = self.get_all_champion_names();
        for name in &names {
            let mut best_score = 0;
            let mut best_match: Option<&String> = None;
            for actual_name in &champion_names {
                let score = matcher.fuzzy_match(actual_name, *name).unwrap();
                if score > best_score {
                    best_match = Some(actual_name);
                    best_score = score;
                }
            }
            match best_match {
                Some(best_match) => corrected_names.push(best_match.clone()),
                None => corrected_names.push(String::from(*name))
            }
        };
        return corrected_names;
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