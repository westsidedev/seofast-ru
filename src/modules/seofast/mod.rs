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
        print!("{}[{}SEOFAST{}]\n", colors.BLUE, colors.YELLOW, colors.BLUE);
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
            colors.BLUE,
            colors.GREEN,
            &self.task,
            colors.BLUE,
            colors.YELLOW,
            &self.username.to_uppercase(),
            colors.BLUE,
            colors.YELLOW,
            &self.classification,
            colors.BLUE,
            colors.YELLOW,
            &self.money,
            colors.BLUE,
            colors.CIAN,
            time_now(Sao_Paulo).await,
            colors.BLUE,
        );
        stdout().flush().unwrap();
    }

    pub async fn tmr(&self, mode: &str, tmr: &str) {
        let colors = Colors::new().await;
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{}{}|{}{}{}][{}{}{}]",
            colors.BLUE,
            colors.GREEN,
            &self.task,
            colors.BLUE,
            colors.YELLOW,
            &self.username.to_uppercase(),
            colors.BLUE,
            colors.YELLOW,
            &self.classification,
            colors.BLUE,
            colors.YELLOW,
            mode,
            colors.BLUE,
            colors.YELLOW,
            tmr,
            colors.BLUE,
            colors.CIAN,
            time_now(Sao_Paulo).await,
            colors.BLUE
        );
        stdout().flush().unwrap();
    }

    pub async fn earn(&self, earn: &str) {
        let colors = Colors::new().await;
        let re = Regex::new(r"(\d.\d\d\d)").unwrap();
        let earn = re.captures(earn).unwrap();
        print!(
            "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}|{}{}{}][{}{}{}]\n",
            colors.BLUE,
            colors.GREEN,
            &self.task,
            colors.BLUE,
            colors.YELLOW,
            &self.username.to_uppercase(),
            colors.BLUE,
            colors.YELLOW,
            &self.classification,
            colors.BLUE,
            colors.YELLOW,
            &self.money,
            colors.BLUE,
            colors.YELLOW,
            &earn[1],
            colors.BLUE,
            colors.CIAN,
            time_now(Sao_Paulo).await,
            colors.BLUE
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
                colors.BLUE, colors.YELLOW, colors.BLUE, colors.YELLOW, i, colors.BLUE
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
