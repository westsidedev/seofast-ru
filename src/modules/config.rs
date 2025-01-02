use chrono::{TimeZone, Utc};
use chrono_tz::America::Sao_Paulo;
use config::{ext::*, *};
use serde::Deserialize;
use std::{fs, io::Write, path::Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct UserData {
    pub email: String,
    pub password: String,
    pub cookies: String,
    pub proxy: String,
    pub port: String,
}

impl UserData {
    pub async fn load() -> UserData {
        let file = Path::new("config/seofast/userdata.json");
        let file_json = DefaultConfigurationBuilder::new()
            .add_json_file(file)
            .build()
            .unwrap();
        let user: UserData = file_json.reify();
        user
    }

    pub async fn create(email: &str, senha: &str, cookies: &str, proxy: &str) -> UserData {
        if Path::new("config/seofast/userdata.json").exists() {
            let _ = fs::remove_dir_all("config/seofast");
        }
        let path = "config/seofast";
        let _ = fs::create_dir_all(path);
        let _ = fs::create_dir_all("config/seofast/screenshot");
        let mut opt = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/userdata.json", path))
            .unwrap();
        let port = "9001";
        let _ = opt.write_all(format!(
                "{{\n\t\"email\":\"{}\",\n\t\"password\": \"{}\",\n\t\"cookies\": \"{}\",\n\t\"proxy\": \"{}\",\n\t\"port\": \"{}\"\n}}",
                    email, senha, cookies, proxy, port
                ).as_bytes(),
            );
        return UserData::load().await;
    }

    pub async fn modify(email: &str, senha: &str, cookies: &str, proxy: &str) -> UserData {
        let path = "config/seofast";
        let _ = fs::remove_file("config/seofast/userdata.json");
        let _ = fs::create_dir(path);
        let mut opt = fs::OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(format!("{}/userdata.json", path))
            .unwrap();
        let port = "9001";
        let _ = opt.write_all(format!(
                "{{\n\t\"email\":\"{}\",\n\t\"password\": \"{}\",\n\t\"cookies\": \"{}\",\n\t\"proxy\": \"{}\",\n\t\"port\": \"{}\"\n}}",
                email, senha, cookies, proxy, port
            ).as_bytes(),
        );
        UserData::load().await
    }

    #[allow(dead_code)]
    pub async fn delete() -> () {
        fs::remove_dir_all("config/seofast").unwrap();
    }
}

#[allow(dead_code)]
pub enum TypeLog {
    INFO,
    ERROR,
    WARN,
    DEBUG,
}

#[derive(Clone)]
pub struct Log;

impl Log {
    pub async fn info(struct_name: &str, msg: &str) -> () {
        let _ = log_user(TypeLog::INFO, struct_name, msg);
    }

    pub async fn error(struct_name: &str, msg: &str) -> () {
        let _ = log_user(TypeLog::ERROR, struct_name, msg);
    }

    #[allow(dead_code)]
    pub async fn warn(struct_name: &str, msg: &str) -> () {
        let _ = log_user(TypeLog::WARN, struct_name, msg);
    }

    pub async fn debug(struct_name: &str, msg: &str) -> () {
        let _ = log_user(TypeLog::DEBUG, struct_name, msg);
    }
}

fn log_user(type_log: TypeLog, struct_name: &str, msg_log: &str) -> Result<(), std::io::Error> {
    let path = "config/seofast/account.log";

    let time_now = Utc::now();

    //convert time_now to NaiveDateTime
    let naive_date_time = time_now.naive_local();
    let hour_sp = Sao_Paulo
        .from_utc_datetime(&naive_date_time)
        .format("%d/%m/%Y %H:%M");
    let mut log = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .write(true)
        .open(&path)?;
    if log.metadata().unwrap().len() >= 5000000 {
        fs::remove_file(&path)?;
        fs::File::create(&path)?;
    }
    match type_log {
        TypeLog::INFO => log.write_all(
            format!("[INFO]({})[{}]:\n{}\n\n", hour_sp, struct_name, msg_log).as_bytes(),
        )?,
        TypeLog::ERROR => log.write_all(
            format!("[ERROR]({})[{}]:\n{}\n\n", hour_sp, struct_name, msg_log).as_bytes(),
        )?,
        TypeLog::WARN => log.write_all(
            format!("[WARN]({})[{}]:\n{}\n\n", hour_sp, struct_name, msg_log).as_bytes(),
        )?,
        TypeLog::DEBUG => log.write_all(
            format!("[DEBUG]({})[{}]:\n{}\n\n", hour_sp, struct_name, msg_log).as_bytes(),
        )?,
    }
    Ok(())
}
