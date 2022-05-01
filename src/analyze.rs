use chrono::NaiveDateTime;
use clap::StructOpt;
use std::ops::Add;
use std::collections::HashMap;
use crate::string::*;

const SUMMARY_LIMIT: usize = 6;
const STATISTICAL_SIGNIFICANCE_THRESHOLD: i32 = 5;

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

    pub fn add(&mut self, win: bool) {
        self.count_of_matches += 1;
        if win {
            self.count_of_wins += 1;
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
    win_rates_with_champions: HashMap<String, WinRateInfo>,
}

impl ChampionInfo {
    pub fn new() -> ChampionInfo {
        return ChampionInfo {
            count_of_matches: 0,
            win_rates_vs_champions: HashMap::new(),
            win_rates_with_champions: HashMap::new(),
        }
    }

    pub fn get_win_rate_vs(&mut self, champion_name: &String) -> &mut WinRateInfo {
        if !self.win_rates_vs_champions.contains_key(champion_name) {
            let info = WinRateInfo::new();
            self.win_rates_vs_champions.insert(champion_name.clone(), info);
        }
        return self.win_rates_vs_champions.get_mut(champion_name).unwrap();
    }

    pub fn get_win_rate_with(&mut self, champion_name: &String) -> &mut WinRateInfo {
        if !self.win_rates_with_champions.contains_key(champion_name) {
            let info = WinRateInfo::new();
            self.win_rates_with_champions.insert(champion_name.clone(), info);
        }
        return self.win_rates_with_champions.get_mut(champion_name).unwrap();
    }

    fn get_significant_list(source: &HashMap<String, WinRateInfo>) -> Vec<(&str, &WinRateInfo)> {
        let mut significant_champions: Vec<(&str, &WinRateInfo)> = Vec::new();
        for (champion_name, win_rate_info) in source {
            if win_rate_info.count_of_matches >= STATISTICAL_SIGNIFICANCE_THRESHOLD {
                significant_champions.push((&champion_name, &win_rate_info));
            }
        };
        significant_champions.sort_by(|a, b|
            a.1.get_win_rate().partial_cmp(&b.1.get_win_rate()).unwrap()
        );
        return significant_champions;
    }

    fn format_top_summary_list(title: &str, sorted_champions: &Vec<(&str, &WinRateInfo)>, reverse: bool) -> String {
        let mut relevant_champions: Vec<(&str, &WinRateInfo)> = Vec::new();
        if reverse {
            for champion in sorted_champions.iter().rev().take(SUMMARY_LIMIT) {
                relevant_champions.push(*champion);
            }
        } else {
            for champion in sorted_champions.iter().take(SUMMARY_LIMIT) {
                relevant_champions.push(*champion);
            }
        }
        let easiest_enemies = relevant_champions;
        let mut text = String::from(title);
        text = text.add(": ").add(&easiest_enemies.len().to_string());
        text.push('\n');
        text = text.add(&WinRateInfo::format_list_of_named(&easiest_enemies, INDENTATION_STRING));
        return text;
    }

    pub fn get_summary_text(&self) -> String {
        let mut text = String::new();
        text = text
            .add("count of matches: ")
            .add(&self.count_of_matches.to_string());
        text.push('\n');

        {
            let allies = ChampionInfo::get_significant_list(&self.win_rates_with_champions);
            text = text.add(&ChampionInfo::format_top_summary_list("best allies", &allies, true));
            text = text.add(&ChampionInfo::format_top_summary_list("worst allies", &allies, false));
        }
        {
            let enemies = ChampionInfo::get_significant_list(&self.win_rates_vs_champions);
            text = text.add(&ChampionInfo::format_top_summary_list("easiest enemies", &enemies, true));
            text = text.add(&ChampionInfo::format_top_summary_list("worst enemies", &enemies, false));
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
                    win_rate_info.add(participant.win);
                }
                let enemies = find_participants_by_team_id(&match_history.info, participant.team_id, false);
                for enemy in enemies {
                    let win_rate_info = champion_info.get_win_rate_vs(&enemy.champion_name);
                    win_rate_info.add(participant.win);
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
        let mut champions = self.get_sorted_champions();
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
        let mut champions = self.get_sorted_champions();
        for (champion_name, champion_info) in &champions {
            let mut ally_count: i32 = 0;
            let mut ally_score: f32 = 0.0;
            for info in &champion_info.win_rates_with_champions {
                if allies.contains(&info.0.as_str()) {
                    ally_count += 1;
                    ally_score += info.1.get_win_rate();
                }
            }
            let mut enemy_count: i32 = 0;
            let mut enemy_score: f32 = 0.0;
            for info in &champion_info.win_rates_vs_champions {
                if allies.contains(&info.0.as_str()) {
                    enemy_count += 1;
                    enemy_score += info.1.get_win_rate();
                }
            }
            println!("{}: ally strength {}, enemy weakness {}, summary chance {}",
                champion_name,
                Self::format_score(ally_count, ally_score),
                Self::format_score(enemy_count, enemy_score),
                Self::format_score(ally_count + enemy_count, ally_score + enemy_score)
            );
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