use std::{env::args, process::exit, sync::atomic::AtomicBool};

use modules::{
    config::UserData,
    seofast::{Mode, Seofast},
};

mod modules;

pub static GLOBAL_CONTROL: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    let mut arg: Vec<String> = args().collect();
    if arg.len() == 1 {
        msg_help().await;
    }
    let _ = arg.remove(0);
    match arg[0].as_str() {
        "--email" => {
            match arg[2].as_str() {
                "--passw" => (),
                _ => msg_help().await,
            }
            match arg[4].as_str() {
                "--browser" => (),
                _ => msg_help().await,
            }
            let mode = match arg[6].as_str() {
                "--YT" | "--yt" => Mode::YOUTUBE,
                "--SF" | "--sf" => Mode::SURFING,
                "--All" => Mode::ALL,
                _ => todo!(),
            };
            let _ = UserData::create(&arg[1], &arg[3], "", "", &arg[5]).await;
            let mut headless = false;
            if arg.len() == 8 {
                if arg[7].contains("--headless") {
                    headless = true;
                }
            }
            Seofast::start(mode, headless).await;
        }
        "--start" => {
            let mode = match arg[1].as_str() {
                "--YT" | "--yt" => Mode::YOUTUBE,
                "--SF" | "--sf" => Mode::SURFING,
                "--All" => Mode::ALL,
                _ => todo!(),
            };
            let mut headless = false;
            if arg.len() == 3 {
                if arg[2].contains("--headless") {
                    headless = true;
                }
            }
            Seofast::start(mode, headless).await;
        }
        "--help" => msg_help().await,
        _ => msg_help().await,
    }
}

async fn msg_help() {
    print!("\x1bc");
    println!("Options:");
    println!(" --email      Email used for login in seofast");
    println!(" --passw      Password used for login in seofast");
    println!(" --browser    Chrome,Chromium,Brave");
    println!(" --start      Start software after first execution");
    println!(" --headless   Active headless mode (OPTIONAL)");
    println!(" --help       Show this message");
    println!(" --YT         Youtube mode");
    println!(" --SF         Surfing mode");
    println!(" --All        Youtube and surfing mode\n");
    println!("Example:");
    println!("STEP 1 [FIRST EXEC]:");
    println!(" ./seofast-ru --email xxxx@xxxx --passw 123456 --browser brave --YT --headless");
    println!("STEP 2 [START]:");
    println!(" ./seofast-ru --start --YT --headless");
    exit(0);
}
