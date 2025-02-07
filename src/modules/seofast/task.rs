use std::{
    io::{self, Write},
    path::Path,
    process::exit,
    sync::atomic::Ordering,
    time::Duration,
};

use regex::Regex;
use thirtyfour::prelude::*;
use tokio::{signal::ctrl_c, spawn, time::sleep};

use crate::{
    modules::{
        browser::{start_driver, Browser},
        config::{Log, UserData},
    },
    GLOBAL_CONTROL,
};

use super::print::{self, Info};

enum TaskResult {
    QUIT,
    CRITICAL,
    OK,
    PAUSE,
    CONTINUE,
}

#[derive(Debug, PartialEq)]
enum VideoType {
    NONE,
    VIP,
    PREMIUM,
    RARE,
}

#[derive(Clone)]
pub struct TaskDriverSeofast {
    headless: bool,
}

impl TaskDriverSeofast {
    async fn login(self) -> Result<WebDriver, WebDriverError> {
        let user = UserData::load().await;
        Log::debug("TaskDriverSeofast-LOGIN", &user.port).await;
        let browser = Browser {
            headless: self.headless,
            proxy: Some(user.proxy.clone()),
            port: user.port,
        };

        let driver = browser.new().await;

        sleep(Duration::from_secs(4)).await;

        let _ = driver.set_page_load_timeout(Duration::from_secs(15)).await;
        if let Err(e) = driver.get("https://seo-fast.ru/login").await {
            Log::error(
                "TaskDriverSeofast->LOGIN",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
        }

        sleep(Duration::from_secs(4)).await;
        if user.cookies.is_empty() {
            let logusername = driver.find(By::Id("logusername")).await;
            if let Err(e) = logusername {
                let _ = driver.quit().await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
            }
            let _ = logusername.unwrap().send_keys(&user.email).await?;

            sleep(Duration::from_secs(2)).await;

            let logpassw = driver.find(By::Id("logpassword")).await;
            if let Err(e) = logpassw {
                let _ = driver.quit().await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
            }
            let _ = logpassw.unwrap().send_keys(&user.password).await?;

            sleep(Duration::from_secs(2)).await;

            let sf_button = driver.find(By::ClassName("sf_button")).await;
            if let Err(e) = sf_button {
                let _ = driver.quit().await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
            }
            let _ = sf_button.unwrap().click().await?;

            let head_info = driver.wait_element(By::ClassName("head_info_b"), 15).await;
            if let Err(e) = head_info {
                let _ = driver.quit().await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
            }

            if let Ok(cookies) = driver.get_all_cookies().await {
                let mut cookie_format = String::new();
                for cookie in cookies {
                    if cookie.name.contains("googtrans") {
                        continue;
                    } else {
                        cookie_format
                            .push_str(format!("{}={}; ", cookie.name, cookie.value).as_str());
                    }
                }
                let _ = UserData::modify(&user.email, &user.password, &cookie_format, &user.proxy)
                    .await;
                return Ok(driver);
            }
        }

        for cookies in user.cookies.split("; ") {
            let cookie: Vec<&str> = cookies.split("=").collect();
            if cookie.len() <= 1 {
                continue;
            } else {
                let mut ck = Cookie::new(cookie[0], cookie[1]);
                ck.set_domain("seo-fast.ru");
                ck.set_same_site(SameSite::Lax);
                ck.set_path("/");
                if let Err(e) = driver.add_cookie(ck).await {
                    return Err(e);
                }
            }
        }
        if let Err(e) = driver.get("https://seo-fast.ru").await {
            let _ = driver.quit().await;
            return Err(e);
        }

        let head_info = driver.wait_element(By::ClassName("head_info_b"), 15).await;
        if let Err(e) = head_info {
            let _ = driver.quit().await;
            let _ = UserData::modify(&user.email, &user.password, "", &user.proxy).await;
            return Err(e);
        }
        return Ok(driver);
    }

    async fn youtube(driver: WebDriver, task_number: &str) -> TaskResult {
        let _ = driver.set_page_load_timeout(Duration::from_secs(20)).await;
        let goto = driver.goto("https://seo-fast.ru/work_youtube?all").await;
        if let Err(e) = goto {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::CONTINUE;
        };
        let mut username = String::new();
        let mut money = String::new();
        let mut classification = String::new();
        //
        let head_info_b = driver.find(By::ClassName("head_info_b")).await;
        if let Err(e) = head_info_b {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::CRITICAL;
        }
        let head_elem = head_info_b.unwrap();
        let a = head_elem.find(By::Tag("a")).await;
        let _ = username.push_str(&a.unwrap().text().await.unwrap());
        //
        let main_balance = driver.find(By::ClassName("main_balance")).await.unwrap();
        let balance_divs = main_balance.find_all(By::Tag("div")).await;
        let balance_span = balance_divs.unwrap()[2].find(By::Tag("span")).await;
        let _ = money.push_str(&balance_span.unwrap().text().await.unwrap());
        //
        let ratingcss2 = driver.find(By::ClassName("ratingcss2")).await.unwrap();
        let _ = classification.push_str(&ratingcss2.text().await.unwrap());
        //
        let re_classi = Regex::new(r"([0-9]+)").unwrap();
        let re_money = Regex::new(r"(\d\d\d.\d\d\d\d|\d\d.\d\d\d\d|\d.\d\d\d\d|\d)").unwrap();
        let money = re_money.captures(&money).unwrap();
        let money: f32 = money[1].parse().unwrap();
        let classification = re_classi.captures(&classification).unwrap();

        let info = Info {
            task: task_number.parse().unwrap(),
            username,
            classification: classification[1].to_string(),
            money,
        };
        print::user(&info).await;
        let tab_origin = driver.window().await.unwrap();

        let rek_table = driver.find_all(By::ClassName("list_rek_table")).await;
        if let Err(e) = rek_table {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::PAUSE;
        }

        if rek_table.as_ref().unwrap().len() < 3 {
            return TaskResult::CONTINUE;
        }

        let tbody = rek_table.unwrap()[2].find(By::Tag("tbody")).await;
        if let Err(e) = tbody {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::PAUSE;
        }

        let mut id_yt = String::new();
        let list_tr = tbody.unwrap().find_all(By::Tag("tr")).await;
        if let Err(e) = list_tr {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::PAUSE;
        }

        let mut list_elem_trash = Vec::new();

        for tr_elem in list_tr.as_ref().unwrap() {
            let tr_id = tr_elem.id().await;
            if let Some(id) = tr_id.unwrap() {
                if id.contains("youtube_v") {
                    list_elem_trash.push(tr_elem.to_owned());
                }
            }
        }

        for elem_trash in list_elem_trash {
            if let Ok(_) = elem_trash.find(By::ClassName("youtube_l")).await {
                if let Some(txt) = elem_trash.id().await.unwrap() {
                    let id_trash = txt.replace("youtube_v", "");
                    let _ = driver
                        .execute(
                            format!("st_task_youtube('{}', '2');", &id_trash),
                            Vec::new(),
                        )
                        .await;
                    sleep(Duration::from_secs(1)).await;
                }
            }

            if let Ok(_) = elem_trash.find(By::ClassName("youtube_s")).await {
                if let Some(txt) = elem_trash.id().await.unwrap() {
                    let id_trash = txt.replace("youtube_v", "");
                    let _ = driver
                        .execute(
                            format!("st_task_youtube('{}', '1');", &id_trash),
                            Vec::new(),
                        )
                        .await;
                    sleep(Duration::from_secs(1)).await;
                }
            }

            if let Ok(_) = elem_trash.find(By::ClassName("youtube_c")).await {
                if let Some(txt) = elem_trash.id().await.unwrap() {
                    let id_trash = txt.replace("youtube_v", "");
                    let _ = driver
                        .execute(
                            format!("st_task_youtube('{}', '3');", &id_trash),
                            Vec::new(),
                        )
                        .await;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        let tr_title = list_tr.as_ref().unwrap()[0].text().await.unwrap();

        let tr = list_tr.unwrap()[1].to_owned();

        if let Some(txt) = tr.id().await.unwrap() {
            let id = txt.replace("youtube_v", "");
            id_yt = id;
        }

        if tr_title.contains("Rutube") {
            let _ = driver
                .execute(format!("st_view_youtube('{}');", &id_yt), Vec::new())
                .await;
            sleep(Duration::from_secs(2)).await;
            return TaskResult::CONTINUE;
        }

        let tds = tr.find_all(By::Tag("td")).await;
        if let Err(e) = tds {
            Log::error(
                "TaskDriverSeofast->YOUTUBE",
                &format!("line:{}\n{}", line!(), e),
            )
            .await;
            return TaskResult::PAUSE;
        }

        let td_span = tds.unwrap()[2].find(By::Tag("span")).await;
        let video_type = match td_span.unwrap().text().await.unwrap().as_str() {
            "RARE" => VideoType::RARE,
            "PREMIUM" => VideoType::PREMIUM,
            "VIP" => VideoType::VIP,
            _ => VideoType::NONE,
        };
        Log::info("TaskDriverSeofast->YOUTUBE", &format!("{:?}", video_type)).await;
        let _ = driver.execute("window.scroll(0,400);", Vec::new()).await;
        let _ = driver
            .screenshot(&Path::new(&"config/seofast/screenshot/window_1.png"))
            .await;
        Log::debug(
            "TaskDriverSeofast->YOUTUBE",
            &format!("line:{}\n{}", line!(), &tr.outer_html().await.unwrap()),
        )
        .await;

        Log::debug("TaskDriverSeofast->YOUTUBE", &id_yt).await;

        //check new mode
        let mut new_mode = false;

        let surf_ckick = tr.find_all(By::ClassName("surf_ckick")).await;

        Log::debug("", &format!("surf-ckick\n{:#?}", surf_ckick)).await;

        if let Ok(elems) = surf_ckick.as_ref() {
            if elems.len() == 0 {
                new_mode = true;
            }
        }

        Log::debug(
            "TaskDriverSeofast->YOUTUBE",
            &format!("new_mode:{:#}", new_mode),
        )
        .await;

        if !new_mode {
            if surf_ckick.as_ref().unwrap().len() <= 1 {
                return TaskResult::CONTINUE;
            }
            let _ = surf_ckick.as_ref().unwrap()[1].click().await;
            if surf_ckick.as_ref().unwrap()[1]
                .attr("onclick")
                .await
                .unwrap()
                .unwrap()
                .contains("start_youtube_view_button")
            {
                let _ = surf_ckick.unwrap()[2].click().await;
            }

            if let Ok(elem) = tr
                .wait_element(By::ClassName("start_link_youtube"), 3)
                .await
            {
                let _ = elem.click().await;
            }

            let _ = driver
                .screenshot(&Path::new(&"config/seofast/screenshot/w_oldmode1-ck.png"))
                .await;
        }

        if new_mode {
            let res_views = driver.find(By::Id(format!("res_views{}", id_yt))).await;
            let div = res_views.unwrap().find(By::Tag("div")).await;
            let a = div.unwrap().find(By::Tag("a")).await;
            let _ = a.unwrap().click().await;
        }

        //check limit video
        let yt_error = driver.wait_element(By::ClassName("youtube_error"), 5).await;
        if let Ok(elem) = yt_error {
            if let Ok(a) = elem.find(By::Tag("a")).await {
                let _ = a.click().await;
            }
            let popup = match driver.wait_element(By::Id("popup_content_list"), 15).await {
                Ok(elem) => elem,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    return TaskResult::CONTINUE;
                }
            };

            let a = popup.find_all(By::Tag("a")).await.unwrap();
            if a.len() < 1 {
                return TaskResult::CONTINUE;
            }
            let _ = a[1].click().await;
            let sf_button_red = driver
                .wait_elements(By::ClassName("sf_button_red"), 15)
                .await;
            if let Ok(elms) = sf_button_red {
                for e in elms {
                    let code = e.attr("onclick").await.unwrap().unwrap();
                    let _ = driver.execute(code, Vec::new()).await;
                }
                return TaskResult::CONTINUE;
            }
        }

        let mut tabs = match driver.windows().await {
            Ok(windows) => windows,
            Err(_) => return TaskResult::QUIT,
        };

        for i in 0..11 {
            if tabs.len() > 1 {
                break;
            }
            tabs = match driver.windows().await {
                Ok(window) => window,
                Err(_) => return TaskResult::CONTINUE,
            };
            sleep(Duration::from_secs(1)).await;
            if i == 10 {
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }
        }
        let mut tab_video_txt = String::new();
        for i in &tabs {
            if *i != tab_origin {
                tab_video_txt = i.to_string();
            }
        }

        let tab_video = WindowHandle::from(tab_video_txt);

        let tmr_id = match new_mode {
            true => format!("timer_ads_{}", id_yt),
            false => "tmr".to_string(),
        };

        let mut earn = String::new();

        if new_mode {
            let _ = driver.switch_to_window(tab_origin.clone()).await;
            let _ = driver
                .screenshot(&Path::new(&"config/seofast/screenshot/w-newmode1.png"))
                .await;
            let res_views = driver
                .find(By::Id(format!("res_views{}", id_yt)))
                .await
                .unwrap();
            let tmr_elem = res_views.wait_element(By::Id(&tmr_id), 5).await;
            if let Err(_) = tmr_elem {
                let _ = driver.switch_to_window(tab_video.clone()).await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            let tmr: i32 = tmr_elem.unwrap().text().await.unwrap().parse().unwrap();
            let mut tmr = tmr * 4;

            while tmr > 0 {
                print::tmr(&info, "YOUTUBE", &tmr.to_string()).await;
                sleep(Duration::from_secs(1)).await;
                tmr -= 1;
                // let tmr_elem = res_views.find(By::Id(&tmr_id)).await;
                // if let Err(_) = tmr_elem.as_ref() {
                //     break;
                // }

                // if let Ok(txt) = tmr_elem.unwrap().text().await {
                //     if !txt.is_empty() {
                //         tmr = txt.parse().unwrap();
                //     } else {
                //         tmr = 0;
                //     }
                // }
            }

            let _ = driver
                .screenshot(&Path::new(&"config/seofast/screenshot/w-newmode2.png"))
                .await;

            let button_purple = res_views
                .wait_element(By::ClassName("sf_button_purple"), 5)
                .await;
            Log::debug(
                "TaskDriverSeofast->YOUTUBE",
                &format!("button_purple\n{:#?}", button_purple),
            )
            .await;
            if let Err(e) = button_purple.as_ref() {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                let _ = driver.switch_to_window(tab_video.clone()).await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            let _ = button_purple.unwrap().click().await;

            let _ = driver
                .screenshot(&Path::new(&"config/seofast/screenshot/w-newmode3.png"))
                .await;

            let span = res_views.wait_element(By::Tag("span"), 10).await;
            Log::debug("TaskDriverSeofast->YOUTUBE", &format!("span\n{:#?}", span)).await;
            if let Err(e) = span {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                let _ = driver.switch_to_window(tab_video.clone()).await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            let _ = driver
                .screenshot(&Path::new(&"config/seofast/screenshot/w-newmode4.png"))
                .await;

            earn = span.unwrap().text().await.unwrap();

            let _ = driver.switch_to_window(tab_video.clone()).await;
            let _ = driver.close_window().await;
            let _ = driver.switch_to_window(tab_origin.clone()).await;
        }

        if !new_mode {
            let _ = driver.switch_to_window(tab_video.clone()).await;

            let url = driver.current_url().await;
            if let Err(_) = url {
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                return TaskResult::CONTINUE;
            }

            if url.unwrap().as_str().contains("video") {
                let _ = driver
                    .screenshot(&Path::new(&"config/seofast/screenshot/window_2.png"))
                    .await;
                let src = driver.source().await;
                if let Err(_) = src {
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    return TaskResult::CONTINUE;
                }
                Log::debug(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), src.unwrap()),
                )
                .await;
            }

            if let Err(_) = driver.wait_element(By::Id("timer-tr-block"), 10).await {
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            };

            //stage 1
            let iframe = driver.wait_element(By::Tag("iframe"), 5).await;
            if let Err(e) = iframe {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                return TaskResult::CONTINUE;
            }

            let _ = iframe.unwrap().enter_frame().await;

            let ytplay = driver
                .wait_element(By::ClassName("ytp-large-play-button"), 5)
                .await;
            if let Err(e) = ytplay.as_ref() {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return TaskResult::CRITICAL;
            }

            if let Ok(b) = ytplay.as_ref().unwrap().is_displayed().await {
                if !b {
                    Log::info("TaskDriverSeofast->YOUTUBE", "ytbutton not displayed").await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
                let _ = ytplay.unwrap().click().await;
                let _ = driver.enter_default_frame().await;
            }

            let tmr = driver.find(By::Id(&tmr_id)).await;
            if let Err(_) = tmr {
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            let mut tmr: i32 = tmr.unwrap().text().await.unwrap().parse().unwrap();

            if tmr > 600 {
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            let mut tmr_error = 0;
            let mut tmr_old = 0;

            while tmr > 0 {
                print::tmr(&info, "YOUTUBE", &tmr.to_string()).await;
                if let Ok(txt_task) = driver.find(By::Id("text_work")).await {
                    if let Ok(txt) = txt_task.text().await {
                        if txt.contains("Перезапустите воспроизведения видео ↺")
                        {
                            if let Ok(iframe2) = driver.find(By::Tag("iframe")).await {
                                let _ = iframe2.enter_frame().await;
                                if let Ok(btn) =
                                    driver.find(By::ClassName("ytp-large-play-button")).await
                                {
                                    let _ = btn.click().await;
                                    let _ = driver.enter_default_frame().await;
                                }
                            }
                        }
                    }
                }

                if tmr != tmr_old {
                    tmr_old = tmr;
                    tmr_error = 0;
                }
                if tmr == tmr_old {
                    tmr_error += 1;
                    if tmr_error == 2000 {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CRITICAL;
                    }
                }

                let tmr_elem = driver.find(By::Id(&tmr_id)).await;
                if let Err(_) = tmr_elem {
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
                tmr = match tmr_elem.as_ref().unwrap().text().await.unwrap().is_empty() {
                    true => 0,
                    false => tmr_elem.unwrap().text().await.unwrap().parse().unwrap(),
                };
            }

            let succes_error = driver.wait_element(By::Id("succes-error"), 5).await;
            if let Err(e) = succes_error {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            Log::debug(
                "TaskDriverSeofast->YOUTUBE",
                &format!(
                    "line:{}\n{}",
                    line!(),
                    succes_error.as_ref().unwrap().outer_html().await.unwrap()
                ),
            )
            .await;

            let span = succes_error
                .unwrap()
                .wait_element(By::Tag("span"), 10)
                .await;
            if let Err(e) = span {
                Log::error(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                let _ = driver
                    .screenshot(&Path::new(&"config/seofast/screenshot/w-oldmode2F.png"))
                    .await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                let _ = driver
                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                    .await;
                return TaskResult::CONTINUE;
            }

            if let Ok(txt) = span.unwrap().text().await {
                earn = txt;
            }

            if earn.contains("YouTube") {
                //stage 2
                let mut stage_error = 0;
                let mut stage_old = 0;
                loop {
                    if let Ok(stage) = driver.execute("return stage;", Vec::new()).await {
                        let stage: String = stage.convert().unwrap();
                        let stage: i32 = stage.parse().unwrap();
                        if stage != stage_old {
                            stage_old = stage;
                            stage_error = 0;
                        }
                        if stage == stage_old {
                            stage_error += 1;
                            if stage_error == 2000 {
                                let _ = driver.close_window().await;
                                let _ = driver.switch_to_window(tab_origin).await;
                                let _ = driver
                                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                                    .await;
                                return TaskResult::CRITICAL;
                            }
                        }
                        if stage != 1 {
                            break;
                        }
                    }
                }
                let iframe = driver.wait_element(By::Tag("iframe"), 10).await;
                if let Err(e) = iframe {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }

                let _ = iframe.unwrap().enter_frame().await;

                let ytplay2 = driver
                    .wait_element(By::ClassName("ytp-large-play-button"), 5)
                    .await;
                if let Err(e) = ytplay2 {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }

                let _ = ytplay2.unwrap().click().await;
                let _ = driver.enter_default_frame().await;

                let tmr_elem = driver.find(By::Id(&tmr_id)).await;
                if let Err(e) = tmr_elem {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }

                let mut tmr: i32 = tmr_elem.unwrap().text().await.unwrap().parse().unwrap();

                let mut tmr_error = 0;
                let mut tmr_old = 0;
                while tmr > 0 {
                    print::tmr(&info, "YOUTUBE", &tmr.to_string()).await;
                    io::stdout().flush().unwrap();
                    if tmr != tmr_old {
                        tmr_old = tmr;
                        tmr_error = 0;
                    }
                    if tmr == tmr_old {
                        tmr_error += 1;
                        if tmr_error == 2000 {
                            let _ = driver.close_window().await;
                            let _ = driver.switch_to_window(tab_origin).await;
                            let _ = driver
                                .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                                .await;
                            return TaskResult::CRITICAL;
                        }
                    }
                    let tmr_elem = driver.find(By::Id(&tmr_id)).await;
                    if let Err(_) = tmr_elem {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CONTINUE;
                    }

                    tmr = match tmr_elem.as_ref().unwrap().text().await.unwrap().is_empty() {
                        true => 0,
                        false => tmr_elem.unwrap().text().await.unwrap().parse().unwrap(),
                    };
                }

                let succes_error = driver.wait_element(By::Id("succes-error"), 5).await;
                if let Err(e) = succes_error {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }

                let span2 = succes_error
                    .unwrap()
                    .wait_element(By::Tag("span"), 10)
                    .await;
                if let Err(e) = span2 {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
                earn = span2.unwrap().text().await.unwrap();
            }
            let _ = driver.close_window().await;
            let _ = driver.switch_to_window(tab_origin).await;
        }

        if earn.is_empty() {
            return TaskResult::CONTINUE;
        }

        print::earn(&info, &earn).await;
        return TaskResult::OK;
    }

    #[allow(dead_code)]
    async fn surfing() -> TaskResult {
        todo!()
    }
}

struct TaskControlSeofast;

impl TaskControlSeofast {
    async fn youtube(headless: bool) -> () {
        let _ = spawn(async move {
            let mut cmdriver = start_driver().await;
            loop {
                if GLOBAL_CONTROL.load(Ordering::Relaxed) {
                    let _ = cmdriver.kill().unwrap();
                    break;
                }
                sleep(Duration::from_secs(1)).await;
            }
        });

        sleep(Duration::from_millis(500)).await;

        let user_task = TaskDriverSeofast { headless };

        let mut task_number = 1;
        loop {
            if GLOBAL_CONTROL.load(Ordering::Relaxed) {
                return ();
            }

            let result_user_task = user_task.clone().login().await;
            if let Err(e) = result_user_task {
                Log::error(
                    "TaskControlSeofast->YOUTUBE",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                continue;
            }

            let driver = result_user_task.unwrap();

            loop {
                if GLOBAL_CONTROL.load(Ordering::Relaxed) {
                    return ();
                }
                let youtube =
                    TaskDriverSeofast::youtube(driver.clone(), &task_number.to_string()).await;
                match youtube {
                    TaskResult::CONTINUE => continue,
                    TaskResult::PAUSE => {
                        let _ = driver.clone().quit().await;
                        print::pause().await;
                        break;
                    }
                    TaskResult::OK => {
                        task_number += 1;
                        continue;
                    }
                    TaskResult::CRITICAL => {
                        let _ = driver.clone().quit().await;
                        break;
                    }
                    TaskResult::QUIT => break,
                }
            }
        }
    }

    #[allow(dead_code)]
    async fn surfing() {
        todo!()
    }

    #[allow(dead_code)]
    async fn all() {
        todo!()
    }
}

pub struct ThreadTaskSeofast {
    pub headless: bool,
}

impl ThreadTaskSeofast {
    pub async fn youtube(self) {
        spawn(async move {
            if let Ok(_) = ctrl_c().await {
                GLOBAL_CONTROL.store(true, Ordering::SeqCst)
            }
        });
        let th = spawn(async move { TaskControlSeofast::youtube(self.headless).await });

        loop {
            if th.is_finished() {
                exit(0);
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn surfing(self) {
        todo!()
    }

    pub async fn all(self) {
        todo!()
    }
}
