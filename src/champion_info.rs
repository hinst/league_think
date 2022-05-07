use std::ops::Add;
use std::collections::HashMap;
use crate::string::*;
use crate::win_rate_info::WinRateInfo;

const SUMMARY_LIMIT: usize = 6;

pub struct ChampionInfo {
    pub count_of_matches: i32,
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
            significant_champions.push((&champion_name, &win_rate_info));
        }
        significant_champions.sort_by(|a, b|
            a.1.get_win_chance().partial_cmp(&b.1.get_win_chance()).unwrap()
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

    pub fn get_win_rates_vs_champions(&self) -> &HashMap<String, WinRateInfo> {
        return &self.win_rates_vs_champions;
    }

    pub fn get_win_rates_with_champions(&self) -> &HashMap<String, WinRateInfo> {
        return &self.win_rates_with_champions;
    }
}
