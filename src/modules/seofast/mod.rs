pub mod task;

use std::{
    io::{stdout, Write},
    time::Duration,
};

use chrono::{TimeZone, Utc};
use chrono_tz::{America::Sao_Paulo, Tz};
use regex::Regex;
use task::ThreadTaskSeofast;
use tokio::time::sleep;

use crate::modules::colors::Colors;

#[allow(dead_code)]
pub enum Mode {
    YOUTUBE,
    SURFING,
    ALL,
}

pub struct Seofast;

impl Seofast {
    pub async fn start(mode: Mode, headless: bool) {
        let colors = Colors::new().await;
        print!("\x1bc");
        print!(
            "{}[{}SEOFAST{}]\n",
            colors.GREEN, colors.WHITE, colors.GREEN
        );
        stdout().flush().unwrap();
        let thread = ThreadTaskSeofast { headless };
        match mode {
            Mode::YOUTUBE => thread.youtube().await,
            Mode::SURFING => thread.surfing().await,
            Mode::ALL => thread.all().await,
        }
    }
}

pub struct Print {
    task: i32,
    username: String,
    classification: String,
    money: f32,
}

impl Print {
    pub async fn user(&self) {
        let colors = Colors::new().await;
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}][{}{}{}]",
            colors.GREEN,
            colors.WHITE,
            &self.task,
            colors.GREEN,
            colors.WHITE,
            &self.username.to_uppercase(),
            colors.GREEN,
            colors.WHITE,
            &self.classification,
            colors.GREEN,
            colors.WHITE,
            &self.money,
            colors.GREEN,
            colors.WHITE,
            time_now(Sao_Paulo).await,
            colors.GREEN,
        );
        stdout().flush().unwrap();
    }

    pub async fn tmr(&self, mode: &str, tmr: &str) {
        let colors = Colors::new().await;
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{}{}|{}{}{}][{}{}{}]",
            colors.GREEN,
            colors.WHITE,
            &self.task,
            colors.GREEN,
            colors.WHITE,
            &self.username.to_uppercase(),
            colors.GREEN,
            colors.WHITE,
            &self.classification,
            colors.GREEN,
            colors.WHITE,
            mode,
            colors.GREEN,
            colors.WHITE,
            tmr,
            colors.GREEN,
            colors.WHITE,
            time_now(Sao_Paulo).await,
            colors.GREEN
        );
        stdout().flush().unwrap();
    }

    pub async fn earn(&self, earn: &str) {
        let colors = Colors::new().await;
        let re = Regex::new(r"(\d.\d\d\d)").unwrap();
        let earn = re.captures(earn).unwrap();
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}|{}{}{}][{}{}{}]\n",
            colors.GREEN,
            colors.WHITE,
            &self.task,
            colors.GREEN,
            colors.WHITE,
            &self.username.to_uppercase(),
            colors.GREEN,
            colors.WHITE,
            &self.classification,
            colors.GREEN,
            colors.WHITE,
            &self.money,
            colors.GREEN,
            colors.WHITE,
            &earn[1],
            colors.GREEN,
            colors.WHITE,
            time_now(Sao_Paulo).await,
            colors.GREEN
        );
        stdout().flush().unwrap();
    }

    pub async fn pause() {
        let colors = Colors::new().await;
        for i in (1..=900).rev() {
            print!(
                "\r\x1b[K{}[{}PAUSED{}]({}{}{})",
                colors.GREEN, colors.YELLOW, colors.GREEN, colors.YELLOW, i, colors.GREEN
            );
            stdout().flush().unwrap();
            sleep(Duration::from_secs(1)).await;
        }
    }
}

pub async fn time_now(state: Tz) -> String {
    let time_now = Utc::now();

    //convert time_now to NaiveDateTime
    let naive_date_time = time_now.naive_local();
    let hour_sp = state.from_utc_datetime(&naive_date_time).format("%H:%M");
    hour_sp.to_string()
}
