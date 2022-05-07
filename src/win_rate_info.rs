use std::ops::Add;
use crate::string::{ format_percent, format_ratio, format_ratio_detailed };

const STATISTICAL_SIGNIFICANCE_THRESHOLD: i32 = 5;

pub struct WinRateInfo {
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

    pub fn get_win_rate(&self) -> f32 {
        if self.count_of_matches > 0 {
            return (self.count_of_wins as f32) / (self.count_of_matches as f32);
        } else {
            return 0.0;
        }
    }

    pub fn get_win_chance(&self) -> f32 {
        if self.count_of_matches == 0 {
            return 0.5
        } else if self.count_of_matches < STATISTICAL_SIGNIFICANCE_THRESHOLD {
            let lack = STATISTICAL_SIGNIFICANCE_THRESHOLD - self.count_of_matches;
            let lack = if lack == 1 { 1.3 }
                else if lack == 2 { 1.6 }
                else { lack as f32 };
            let win_rate = self.get_win_rate();
            let delta = win_rate - 0.5;
            return 0.5 + delta / lack;
        } else {
            return self.get_win_rate();
        }
    }

    pub fn format_list_of_named(list: &[(&str, &WinRateInfo)], indentation: &str) -> String {
        let mut text = String::new();
        for (champion_name, win_rate_info) in list {
            text = text
                .add(indentation)
                .add(champion_name)
                .add(" ")
                .add(" chance ")
                .add(&format_percent(win_rate_info.get_win_chance()))
                .add(" ratio ")
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

    pub fn get_count_of_matches(&self) -> i32 {
        return self.count_of_matches;
    }
}

impl ToString for WinRateInfo {
    fn to_string(&self) -> String {
        return format_ratio_detailed(self.count_of_wins, self.count_of_matches);
    }
}
