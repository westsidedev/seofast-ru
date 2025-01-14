pub mod task;

use std::{
    io::{stdout, Write},
    sync::atomic::Ordering,
    time::Duration,
};

use chrono::{TimeZone, Utc};
use chrono_tz::{America::Sao_Paulo, Tz};
use regex::Regex;
use task::ThreadTaskSeofast;
use tokio::time::sleep;

use crate::{modules::colors::Colors, GLOBAL_CONTROL};

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
            colors.WHITE, colors.GREEN, colors.WHITE
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
            colors.WHITE,
            colors.GREEN,
            &self.task,
            colors.WHITE,
            colors.GREEN,
            &self.username.to_uppercase(),
            colors.WHITE,
            colors.GREEN,
            &self.classification,
            colors.WHITE,
            colors.GREEN,
            &self.money,
            colors.WHITE,
            colors.GREEN,
            time_now(Sao_Paulo).await,
            colors.WHITE,
        );
        stdout().flush().unwrap();
    }

    pub async fn tmr(&self, mode: &str, tmr: &str) {
        let colors = Colors::new().await;
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{}{}|{}{}{}][{}{}{}]",
            colors.WHITE,
            colors.GREEN,
            &self.task,
            colors.WHITE,
            colors.GREEN,
            &self.username.to_uppercase(),
            colors.WHITE,
            colors.GREEN,
            &self.classification,
            colors.WHITE,
            colors.GREEN,
            mode,
            colors.WHITE,
            colors.GREEN,
            tmr,
            colors.WHITE,
            colors.GREEN,
            time_now(Sao_Paulo).await,
            colors.WHITE
        );
        stdout().flush().unwrap();
    }

    pub async fn earn(&self, earn: &str) {
        let colors = Colors::new().await;
        let re = Regex::new(r"(\d.\d\d\d)").unwrap();
        let earn = re.captures(earn).unwrap();
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}|{}{}{}][{}{}{}]\n",
            colors.WHITE,
            colors.GREEN,
            &self.task,
            colors.WHITE,
            colors.GREEN,
            &self.username.to_uppercase(),
            colors.WHITE,
            colors.GREEN,
            &self.classification,
            colors.WHITE,
            colors.GREEN,
            &self.money,
            colors.WHITE,
            colors.GREEN,
            &earn[1],
            colors.WHITE,
            colors.GREEN,
            time_now(Sao_Paulo).await,
            colors.WHITE
        );
        stdout().flush().unwrap();
    }

    pub async fn pause() {
        let colors = Colors::new().await;
        for i in (1..=600).rev() {
            if GLOBAL_CONTROL.load(Ordering::Relaxed) {
                break;
            }
            print!(
                "\r\x1b[K{}[{}PAUSED{}]({}{}{})",
                colors.WHITE, colors.YELLOW, colors.WHITE, colors.YELLOW, i, colors.WHITE
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
