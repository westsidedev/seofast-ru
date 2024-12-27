use chrono::{TimeZone, Utc};
use chrono_tz::America::Sao_Paulo;
use config::{ext::JsonConfigurationExtensions, ConfigurationBuilder, DefaultConfigurationBuilder};
use std::{fs, io::Write, path::Path};

use super::browser::BrowserName;

#[allow(dead_code)]
#[derive(Clone)]
pub struct UserData {
    pub email: String,
    pub password: String,
    pub cookies: String,
    pub proxy: String,
    pub port: String,
    pub browser: BrowserName,
}

impl UserData {
    pub async fn load() -> UserData {
        let file = Path::new("config/seofast/userdata.json");
        let file_json = DefaultConfigurationBuilder::new()
            .add_json_file(file)
            .build()
            .unwrap();
        let email = file_json.get("email").unwrap().as_str().to_string();
        let password = file_json.get("password").unwrap().as_str().to_string();
        let cookies = file_json.get("cookies").unwrap().as_str().to_string();
        let proxy = file_json.get("proxy").unwrap().as_str().to_string();
        let port = file_json.get("port").unwrap().as_str().to_string();
        let browser = file_json.get("browser").unwrap();
        let browser = match browser.as_str() {
            "brave" => BrowserName::Brave,
            "chrome" => BrowserName::Chrome,
            "chromium" => BrowserName::Chromium,
            _ => unimplemented!(),
        };
        let user = UserData {
            email,
            password,
            cookies,
            proxy,
            port,
            browser,
        };
        user
    }

    pub async fn create(
        email: &str,
        senha: &str,
        cookies: &str,
        proxy: &str,
        browser: &str,
    ) -> UserData {
        if Path::new("config/seofast/userdata.json").exists() {
            let _ = fs::remove_dir_all("config/seofast");
        }
        let path = "config/seofast";
        let _ = fs::create_dir_all(path);
        let mut opt = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/userdata.json", path))
            .unwrap();
        let port = "9001";
        let _ = opt.write_all(format!(
                "{{\n\t\"email\":\"{}\",\n\t\"password\": \"{}\",\n\t\"cookies\": \"{}\",\n\t\"proxy\": \"{}\",\n\t\"port\": \"{}\",\n\t\"browser\": \"{}\"\n}}",
                    email, senha, cookies, proxy, port, browser
                ).as_bytes(),
            );
        return UserData::load().await;
    }

    pub async fn modify(
        email: &str,
        senha: &str,
        cookies: &str,
        proxy: &str,
        browser: &str,
    ) -> UserData {
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
                "{{\n\t\"email\":\"{}\",\n\t\"password\": \"{}\",\n\t\"cookies\": \"{}\",\n\t\"proxy\": \"{}\",\n\t\"port\": \"{}\",\n\t\"browser\": \"{}\"\n}}",
                email, senha, cookies, proxy, port, browser
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
