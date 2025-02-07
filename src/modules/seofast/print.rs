use std::{
    io::{stdout, Write},
    sync::atomic::Ordering,
    time::Duration,
};

use chrono::{TimeZone, Utc};
use chrono_tz::{America::Sao_Paulo, Tz};
use regex::Regex;
use tokio::time::sleep;

use crate::{modules::colors::Colors, GLOBAL_CONTROL};

#[derive(Clone)]
pub struct Info {
    pub task: i32,
    pub username: String,
    pub classification: String,
    pub money: f32,
}

pub async fn user(info: &Info) {
    let c = Colors::new().await;
    print!(
        "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}][{}{}{}]",
        c.BLUE,
        c.GREEN,
        info.task,
        c.BLUE,
        c.YELLOW,
        info.username.to_uppercase(),
        c.BLUE,
        c.YELLOW,
        info.classification,
        c.BLUE,
        c.YELLOW,
        info.money,
        c.BLUE,
        c.CIAN,
        time_now(Sao_Paulo).await,
        c.BLUE,
    );
    stdout().flush().unwrap();
}

pub async fn tmr(info: &Info, mode: &str, tmr: &str) {
    let c = Colors::new().await;
    print!(
        "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{}{}|{}{}{}][{}{}{}]",
        c.BLUE,
        c.GREEN,
        info.task,
        c.BLUE,
        c.YELLOW,
        info.username.to_uppercase(),
        c.BLUE,
        c.YELLOW,
        info.classification,
        c.BLUE,
        c.YELLOW,
        mode,
        c.BLUE,
        c.YELLOW,
        tmr,
        c.BLUE,
        c.CIAN,
        time_now(Sao_Paulo).await,
        c.BLUE
    );
    stdout().flush().unwrap();
}

pub async fn earn(info: &Info, earn: &str) {
    let c = Colors::new().await;
    let re = Regex::new(r"(\d.\d\d\d)").unwrap();
    let earn = re.captures(earn).unwrap();
    print!(
        "\r\x1b[K{}[{}{}{}][{}{}{}|{}{}{}|{}{:.2}{}|{}{}{}][{}{}{}]\n",
        c.BLUE,
        c.GREEN,
        info.task,
        c.BLUE,
        c.YELLOW,
        info.username.to_uppercase(),
        c.BLUE,
        c.YELLOW,
        info.classification,
        c.BLUE,
        c.YELLOW,
        info.money,
        c.BLUE,
        c.YELLOW,
        &earn[1],
        c.BLUE,
        c.CIAN,
        time_now(Sao_Paulo).await,
        c.BLUE
    );
    stdout().flush().unwrap();
}

pub async fn pause() {
    let c = Colors::new().await;
    for i in (1..=600).rev() {
        if GLOBAL_CONTROL.load(Ordering::Relaxed) {
            break;
        }
        print!(
            "\r\x1b[K{}[{}PAUSED{}]({}{}{})",
            c.BLUE, c.YELLOW, c.BLUE, c.YELLOW, i, c.BLUE
        );
        stdout().flush().unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn time_now(state: Tz) -> String {
    let time_now = Utc::now();

    //convert time_now to NaiveDateTime
    let naive_date_time = time_now.naive_local();
    let hour_sp = state.from_utc_datetime(&naive_date_time).format("%H:%M");
    hour_sp.to_string()
}
