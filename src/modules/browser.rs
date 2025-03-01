use std::{
    collections::HashMap,
    env,
    process::{Child, Command, Stdio},
};
use thirtyfour::prelude::*;

use super::config::UserData;

pub struct Browser {
    pub headless: bool,
    pub proxy: Option<String>,
    pub port: String,
}

impl Browser {
    pub async fn new(&self) -> WebDriver {
        let path = env::current_dir().unwrap();
        let user_data_dir = format!("--user-data-dir={}/config/seofast/brave", path.display());

        let mut args = vec![
            "--ignore-certificate-errors-spki-list",
            "--ignore-certificate-errors",
            "--ignore-ssl-errors",
            "--start-maximized",
            "--log-level=OFF",
            "--silent",
            //"--disable-web-security",
            "--allow-running-insecure-content",
            "--mute-audio",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--enable-low-end-device-mode",
            "--disable-low-res-tiling",
            "--disable-background-timer-throttling",
            //"--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            //"--disable-client-side-phishing-detection",
            "--disable-accelerated-2d-canvas",
            //"--disable-crash-reporter",
            //"--disable-oopr-debug-crash-dump",
            //"--disable-2d-canvas-image-chromium",
            //"--disable-2d-canvas-clip-aa",
            "--disable-notifications",
            "--disable-stack-profiler",
            "--disable-gl-drawing-for-tests",
            "--disable-setuid-sandbox",
            "--disable-blink-features=AutomationControlled",
            "--disable-logging",
            "--disable-gpu",
            //"--disable-webgl",
            //"--disable-webgl2",
            "--no-first-run",
            //"--arc-disable-media-store-maintenance",
            "--no-zygote",
            "--no-crash-upload",
            //"--aggressive-cache-discard",
            //"--single-process",
            "--enable-high-efficiency-mode",
            //"--ash-no-nudges",
            "--hide-scrollbars",
            "--incognito",
            &user_data_dir,
        ];

        let mut proxy_arg = String::from("--proxy-server=");
        if let Some(px) = &self.proxy {
            if !px.is_empty() {
                proxy_arg.push_str(&px);
                args.push(&proxy_arg);
            }
        }

        if self.headless.eq(&true) {
            args.push("--headless=new");
        }

        let mut caps = DesiredCapabilities::chrome();

        for arg in args {
            let _ = caps.add_arg(arg);
        }

        let mut prefs = HashMap::new();
        prefs.insert(
            String::from("profile.default_content_setting_values.notifications"),
            2,
        );
        let _ = caps.add_experimental_option("excludeSwitches", vec!["enable-automation"]);
        //let _ = caps.add_experimental_option("useAutomationExtension", false);
        let _ = caps.add_experimental_option("prefs", prefs);
        let _ = caps.add_experimental_option("w3c", true);

        let mut uname = Command::new("uname");
        let output = uname.arg("-o").output();
        if let Ok(out) = output {
            let result = out.stdout;
            let result = std::str::from_utf8(&result).unwrap();
            if result.contains("Linux") {
                let _ = caps.set_binary("/bin/brave");
            }
            if result.contains("Android") {
                let _ = caps.set_binary("/data/data/com.termux/files/usr/bin/chromium-browser");
            }
        }

        if std::env::consts::OS.contains("windows") {
            let _ = caps.set_binary(&format!(
                "{}/config/browser/brave/brave.exe",
                path.display()
            ));
        }

        let driver = WebDriver::new(format!("http://localhost:{}", &self.port), caps).await;
        driver.unwrap()
    }
}

pub async fn start_driver() -> Child {
    let port = UserData::load().await.port;
    if std::env::consts::OS.contains("windows") {
        return Command::new(format!(
            "{}/config/driver/chromedriver.exe",
            env::current_dir().unwrap().display()
        ))
        .args(&[&format!("--port={}", port), "--silent"])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    }
    Command::new("chromedriver")
        .args(&[&format!("--port={}", port), "--silent"])
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
}
