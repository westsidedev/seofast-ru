use std::{
    io::{self, Write},
    path::Path,
    process::exit,
    sync::atomic::Ordering,
    time::Duration,
};

use regex::Regex;
use thirtyfour::{error::WebDriverErrorInner, prelude::*};
use tokio::{signal::ctrl_c, spawn, time::sleep};

use crate::{
    modules::{
        browser::{start_driver, Browser, BrowserName},
        config::{Log, UserData},
    },
    GLOBAL_CONTROL,
};

use super::Print;

#[allow(non_camel_case_types)]
enum TaskResult {
    LOGIN_ERROR,
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
            name: user.browser.clone(),
            headless: self.headless,
            proxy: Some(user.proxy.clone()),
            port: user.port,
        };

        let browser_name = match user.browser {
            BrowserName::Brave => "brave",
            BrowserName::Chrome => "chrome",
            BrowserName::Chromium => "chromium",
        };

        let driver = browser.new().await;
        sleep(Duration::from_secs(4)).await;

        let _ = driver.set_page_load_timeout(Duration::from_secs(15)).await;
        let _ = match driver.get("https://seo-fast.ru/login").await {
            Ok(_) => (),
            Err(e) => {
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                ()
            }
        };

        sleep(Duration::from_secs(4)).await;
        if user.cookies.is_empty() {
            match driver.find(By::Id("logusername")).await {
                Ok(elm_username) => elm_username.send_keys(&user.email).await?,
                Err(e) => {
                    let _ = driver.quit().await;
                    Log::error(
                        "TaskDriverSeofast->LOGIN",
                        &format!("element logusername not found\n{}", e),
                    )
                    .await;
                    return Err(e);
                }
            }

            sleep(Duration::from_secs(2)).await;
            match driver.find(By::Id("logpassword")).await {
                Ok(elm_passw) => elm_passw.send_keys(&user.password).await?,
                Err(e) => {
                    let _ = driver.quit().await;
                    Log::error(
                        "TaskDriverSeofast->LOGIN",
                        &format!("element logpassword not found\n{}", e),
                    )
                    .await;
                    return Err(e);
                }
            }

            sleep(Duration::from_secs(2)).await;
            match driver.find(By::ClassName("sf_button")).await {
                Ok(elm_login) => elm_login.click().await?,
                Err(e) => {
                    let _ = driver.quit().await;
                    Log::error(
                        "TaskDriverSeofast->LOGIN",
                        &format!("element sf_button not found\n{}", e),
                    )
                    .await;
                    return Err(e);
                }
            }

            let _ = match driver.wait_element(By::ClassName("head_info_b"), 15).await {
                Ok(_) => {
                    if let Ok(cookies) = driver.get_all_cookies().await {
                        let mut cookie_format = String::new();
                        for cookie in cookies {
                            if cookie.name.contains("googtrans") {
                                continue;
                            } else {
                                cookie_format.push_str(
                                    format!("{}={}; ", cookie.name, cookie.value).as_str(),
                                );
                            }
                        }
                        let _ = UserData::modify(
                            &user.email,
                            &user.password,
                            &cookie_format,
                            &user.proxy,
                            &browser_name,
                        )
                        .await;
                        return Ok(driver);
                    }
                }
                Err(e) => {
                    let _ = driver.quit().await;
                    Log::error(
                        "TaskDriverSeofast->LOGIN",
                        &format!("element head_info_b not found\n{}", e),
                    )
                    .await;
                    return Err(e);
                }
            };
        }
        for cookies in user.cookies.split("; ") {
            let cookie: Vec<&str> = cookies.split("=").collect();
            //println!("{}", cookies);
            if cookie.len() <= 1 {
                continue;
            } else {
                let mut ck = Cookie::new(cookie[0], cookie[1]);
                ck.set_domain("seo-fast.ru");
                ck.set_same_site(SameSite::Lax);
                ck.set_path("/");
                match driver.add_cookie(ck).await {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }
        }
        let _ = match driver.get("https://seo-fast.ru").await {
            Ok(_) => (),
            Err(e) => {
                let _ = driver.quit().await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
                //continue;
            }
        };

        match driver.wait_element(By::ClassName("head_info_b"), 15).await {
            Ok(_) => return Ok(driver),
            Err(e) => {
                let _ = driver.quit().await;
                let _ =
                    UserData::modify(&user.email, &user.password, "", &user.proxy, &browser_name)
                        .await;
                Log::error(
                    "TaskDriverSeofast->LOGIN",
                    &format!("line:{}\n{}", line!(), e),
                )
                .await;
                return Err(e);
            }
        }
    }

    async fn youtube(driver: WebDriver, task_number: &str) -> TaskResult {
        let _ = driver.set_page_load_timeout(Duration::from_secs(10)).await;
        let _ = match driver.goto("https://seo-fast.ru/work_youtube?all").await {
            Ok(_) => (),
            Err(e) => match e.as_inner() {
                WebDriverErrorInner::Timeout(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await
                }
                WebDriverErrorInner::WebDriverTimeout(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await
                }
                _ => Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await,
            },
        };
        let username = match driver.find(By::ClassName("head_info_b")).await {
            Ok(head_info) => match head_info.find(By::Tag("a")).await {
                Ok(a) => match a.text().await {
                    Ok(txt) => txt,
                    Err(_) => return TaskResult::LOGIN_ERROR,
                },
                Err(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                    return TaskResult::CONTINUE;
                }
            },
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::LOGIN_ERROR;
            }
        };
        let money = match driver.find(By::ClassName("main_balance")).await {
            Ok(balance) => match balance.find_all(By::Tag("div")).await {
                Ok(list_div) => match list_div[2].find(By::Tag("span")).await {
                    Ok(span) => match span.text().await {
                        Ok(span_txt) => span_txt,
                        Err(_) => return TaskResult::CONTINUE,
                    },
                    Err(e) => {
                        Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                        return TaskResult::CONTINUE;
                    }
                },
                Err(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                    return TaskResult::CONTINUE;
                }
            },
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::LOGIN_ERROR;
            }
        };
        let classification = match driver.find(By::ClassName("ratingcss2")).await {
            Ok(rating) => rating.text().await.unwrap(),
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::LOGIN_ERROR;
            }
        };
        let re_classi = Regex::new(r"([0-9]+)").unwrap();
        let re_money = Regex::new(r"(\d\d\d.\d\d\d\d|\d\d.\d\d\d\d|\d.\d\d\d\d|\d)").unwrap();
        let money = re_money.captures(&money).unwrap();
        let money: f32 = money[1].parse().unwrap();
        let classification = re_classi.captures(&classification).unwrap();

        let p = Print {
            task: task_number.parse().unwrap(),
            username,
            classification: classification[1].to_string(),
            money,
        };
        p.user().await;
        let tab_origin = match driver.window().await {
            Ok(tab) => tab,
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::CONTINUE;
            }
        };

        let rek_table = match driver.find_all(By::ClassName("list_rek_table")).await {
            Ok(elm) => {
                if elm.len() < 3 {
                    return TaskResult::CONTINUE;
                }
                match elm[2].find(By::Tag("tbody")).await {
                    Ok(tbody) => tbody,
                    Err(e) => {
                        Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                        return TaskResult::PAUSE;
                    }
                }
            }
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::PAUSE;
            }
        };
        let mut id_yt = String::new();
        let task = match rek_table.find_all(By::Tag("tr")).await {
            Ok(list_tr) => {
                Log::info(
                    "TaskDriverSeofast->YOUTUBE",
                    &format!(
                        "line:{}\ntxt tag tr[0]: {}",
                        line!(),
                        list_tr[0].text().await.unwrap()
                    ),
                )
                .await;
                let txt_tag0 = list_tr[0].text().await.unwrap();

                //task de comentar e etc...
                if txt_tag0.contains("Оставить комментарий") {
                    let _ = driver.quit().await;
                    return TaskResult::PAUSE;
                }

                //paga pouco
                if txt_tag0.contains("Rutube") || txt_tag0.contains("Подписаться на канал")
                {
                    match list_tr[1].attr("id").await {
                        Ok(id) => {
                            let id = id.unwrap();
                            let re = Regex::new(r"([0-9]+)").unwrap();
                            let id_trash = re.captures(&id).unwrap();
                            Log::info(
                                "TaskDriverSeofast->YOUTUBE",
                                &format!("idtrash: {}", &id_trash[1]),
                            )
                            .await;
                            let _ = driver
                                .execute(
                                    format!("st_view_youtube('{}');", &id_trash[1]),
                                    Vec::new(),
                                )
                                .await;
                            sleep(Duration::from_secs(2)).await;
                            return TaskResult::CONTINUE;
                        }
                        Err(e) => {
                            Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                            return TaskResult::PAUSE;
                        }
                    }
                }
                if txt_tag0.contains("Поставить Лайк") {
                    match list_tr[1].attr("id").await {
                        Ok(id) => {
                            let id = id.unwrap();
                            let re = Regex::new(r"([0-9]+)").unwrap();
                            let id_trash = re.captures(&id).unwrap();
                            Log::info(
                                "TaskDriverSeofast->YOUTUBE",
                                &format!("idtrash: {}", &id_trash[1]),
                            )
                            .await;
                            let _ = driver
                                .execute(
                                    format!("st_task_youtube('{}',2);", &id_trash[1]),
                                    Vec::new(),
                                )
                                .await;
                            sleep(Duration::from_secs(2)).await;
                            return TaskResult::CONTINUE;
                        }
                        Err(e) => {
                            Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                            return TaskResult::PAUSE;
                        }
                    }
                }
                match list_tr[1].attr("id").await {
                    Ok(id) => {
                        if let Some(id) = id {
                            if id.contains("youtube") {
                                let re = Regex::new(r"([0-9]+)").unwrap();
                                let id_reg = re.captures(&id).unwrap();
                                id_yt.push_str(&id_reg[1]);
                                list_tr[1].to_owned()
                            } else {
                                Log::info("TaskDriverSeofast->YOUTUBE", "id not contains youtube")
                                    .await;
                                return TaskResult::PAUSE;
                            }
                        } else {
                            Log::info("TaskDriverSeofast->YOUTUBE", "id none").await;
                            return TaskResult::PAUSE;
                        }
                    }
                    Err(e) => {
                        Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                        return TaskResult::PAUSE;
                    }
                }
            }
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::PAUSE;
            }
        };
        let task_type = match task.find_all(By::Tag("td")).await {
            Ok(list_td) => match list_td[2].find(By::Tag("span")).await {
                Ok(span) => match span.text().await.unwrap().as_str() {
                    "RARE" => VideoType::RARE,
                    "PREMIUM" => VideoType::PREMIUM,
                    "VIP" => VideoType::VIP,
                    _ => VideoType::NONE,
                },
                Err(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                    return TaskResult::PAUSE;
                }
            },
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                return TaskResult::PAUSE;
            }
        };
        Log::info("TaskDriverSeofast->YOUTUBE", &format!("{:?}", task_type)).await;
        let _ = driver.execute("window.scroll(0,400);", Vec::new()).await;
        let _ = driver
            .screenshot(&Path::new(&"config/seofast/window_1.png"))
            .await;
        Log::debug(
            "TaskDriverSeofast->YOUTUBE",
            &format!("line:{}\n{}", line!(), &task.outer_html().await.unwrap()),
        )
        .await;

        //check old mode
        let old_mode = match task.find_all(By::ClassName("surf_ckick")).await {
            Ok(surf_click) => {
                match surf_click.len().ge(&2) {
                    true => {
                        match surf_click[1].click().await {
                            Ok(_) => {
                                let _ = driver
                                    .screenshot(&Path::new(&"config/seofast/window_1-click.png"))
                                    .await;
                                true
                            }
                            Err(e) => {
                                Log::error(
                                    "TaskDriverSeofast->YOUTUBE",
                                    &format!("surf_click\n{}", e),
                                )
                                .await;
                                //return TaskResult::PAUSE;
                                false
                            }
                        }
                    }
                    false => {
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CONTINUE;
                    }
                }
            }
            Err(e) => {
                Log::error("TaskDriverSeofast->YOUTUBE", &format!("surf_click\n{}", e)).await;
                //return TaskResult::PAUSE;
                false
            }
        };

        //check limit video
        let yt_error: Result<WebElement, ()> =
            match driver.wait_element(By::ClassName("youtube_error"), 5).await {
                Ok(yt_error) => Ok(yt_error),
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\n{}", line!(), e),
                    )
                    .await;
                    Err(())
                }
            };
        if let Ok(elem) = yt_error {
            let _ = match elem.find(By::Tag("a")).await {
                Ok(elem) => elem.click().await,
                Err(_) => Ok(()),
            };
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
        let _ = driver
            .screenshot(&Path::new(&"config/seofast/window_1-click2.png"))
            .await;
        //panic!();
        if !old_mode {
            //check new mode
            match task.find_all(By::Tag("td")).await {
                Ok(list_td) => match list_td[1].find(By::Tag("div")).await {
                    Ok(div) => match div.attr("id").await {
                        Ok(_) => true,
                        Err(e) => {
                            Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return TaskResult::PAUSE;
                            false
                        }
                    },
                    Err(e) => {
                        Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                        //return TaskResult::PAUSE;
                        false
                    }
                },
                Err(e) => {
                    Log::error("TaskDriverSeofast->YOUTUBE", &format!("{}", e)).await;
                    //return TaskResult::PAUSE;
                    false
                }
            };
        }
        let mut tabs = match driver.windows().await {
            Ok(windows) => windows,
            Err(_) => return TaskResult::QUIT,
        };
        for i in 0..11 {
            let _ = Log::debug(
                "TaskDriverSeofast->YOUTUBE",
                &format!("tabs:{}", tabs.len()),
            )
            .await;
            if tabs.len() > 1 {
                break;
            }
            tabs = match driver.windows().await {
                Ok(window) => window,
                //Err(WebDriverError::HttpError(_)) => return TaskResult::QUIT,
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
        for i in &tabs {
            driver.switch_to_window(i.to_owned()).await.unwrap();
            match driver.current_url().await {
                Ok(url) => {
                    if url.as_str().contains("video") {
                        driver.switch_to_window(i.to_owned()).await.unwrap();
                        let _ = driver
                            .screenshot(&Path::new(&"config/seofast/window_2.png"))
                            .await;
                        let src = match driver.source().await {
                            Ok(src) => src,
                            Err(e) => match e.as_inner() {
                                WebDriverErrorInner::WebDriverTimeout(_) => {
                                    let _ = driver.close_window().await;
                                    let _ = driver.switch_to_window(tab_origin).await;
                                    return TaskResult::CONTINUE;
                                }
                                _ => {
                                    let _ = driver.close_window().await;
                                    let _ = driver.switch_to_window(tab_origin).await;
                                    return TaskResult::CONTINUE;
                                }
                            },
                        };
                        Log::debug(
                            "TaskDriverSeofast->YOUTUBE",
                            &format!("line:{}\n{}", line!(), src),
                        )
                        .await;
                    }
                }
                Err(_) => {
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    return TaskResult::CONTINUE;
                }
            }
        }

        sleep(Duration::from_secs(3)).await;

        //old mode exec
        if old_mode {
            let _ = match driver.wait_element(By::Id("timer-tr-block"), 10).await {
                Ok(_) => (),
                Err(e) => {
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
            };

            //stage 1
            let iframe = match driver.wait_element(By::Tag("iframe"), 5).await {
                Ok(elem) => elem,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("iframe youtube not found\n{}", e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    return TaskResult::CONTINUE;
                }
            };

            let _ = iframe.enter_frame().await;

            let ytplay = match driver
                .wait_element(By::ClassName("ytp-large-play-button"), 5)
                .await
            {
                Ok(elem) => elem,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("ytbutton not found\n{}", e),
                    )
                    .await;
                    return TaskResult::CRITICAL;
                }
            };

            if let Ok(b) = ytplay.is_displayed().await {
                if !b {
                    Log::info("TaskDriverSeofast->YOUTUBE", "ytbutton not displayed").await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
                let _ = ytplay.click().await;
                let _ = driver.enter_default_frame().await;
            }

            let mut tmr = match driver.find(By::Id("timer-tr-block")).await {
                Ok(time) => match time.find(By::Id("tmr")).await {
                    Ok(tmr) => match tmr.text().await {
                        Ok(txt) => txt.parse::<i32>().unwrap(),
                        Err(_) => {
                            let _ = driver.close_window().await;
                            let _ = driver.switch_to_window(tab_origin).await;
                            return TaskResult::CONTINUE;
                        }
                    },
                    Err(_) => {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        return TaskResult::CONTINUE;
                    }
                },
                Err(_) => {
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };

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
                p.tmr("YOUTUBE", &tmr.to_string()).await;
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

                tmr = match driver.find(By::Id("timer-tr-block")).await {
                    Ok(time) => match time.find(By::Id("tmr")).await {
                        Ok(tmr) => match tmr.text().await {
                            Ok(txt) => {
                                if !txt.is_empty() {
                                    let n = txt.parse::<i32>().unwrap();
                                    n
                                } else {
                                    0
                                }
                            }
                            Err(_) => {
                                let _ = driver.close_window().await;
                                let _ = driver.switch_to_window(tab_origin).await;
                                return TaskResult::CONTINUE;
                            }
                        },
                        Err(_) => {
                            let _ = driver.close_window().await;
                            let _ = driver.switch_to_window(tab_origin).await;
                            return TaskResult::CONTINUE;
                        }
                    },
                    Err(_) => {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CONTINUE;
                    }
                };
            }

            // if task_type == VideoType::RARE {
            //     let succes_error = match driver.wait_element(By::Id("succes-error"), 5).await {
            //         Ok(elem) => elem,
            //         Err(e) => {
            //             Log::error(
            //                 "TaskDriverSeofast->YOUTUBE",
            //                 &format!("RARE\nsucces-error not found\n{}", e),
            //             )
            //             .await;
            //             let _ = driver.close_window().await;
            //             let _ = driver.switch_to_window(tab_origin).await;
            //             let _ = driver
            //                 .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
            //                 .await;
            //             return TaskResult::CONTINUE;
            //         }
            //     };

            //     let _ = match succes_error.wait_element(By::Tag("a"), 10).await {
            //         Ok(elem) => elem.click().await,
            //         Err(_) => {
            //             Log::error("TaskDriverSeofast->YOUTUBE", "tag a not found").await;
            //             let _ = driver.close_window().await;
            //             let _ = driver.switch_to_window(tab_origin).await;
            //             return TaskResult::CONTINUE;
            //         }
            //     };
            // }

            let succes_error = match driver.wait_element(By::Id("succes-error"), 5).await {
                Ok(elem) => elem,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("line:{}\nsuccess-error not found\n{}", line!(), e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };

            let span = match succes_error.wait_element(By::Tag("span"), 10).await {
                Ok(elem) => elem,
                Err(_) => {
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };

            let money = span.text().await.unwrap();
            if !money.contains("YouTube") {
                p.earn(&money).await;
                let _ = driver.close_window().await;
                let _ = driver.switch_to_window(tab_origin).await;
                return TaskResult::OK;
            }
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
            let iframe = match driver.wait_element(By::Tag("iframe"), 10).await {
                Ok(iframe) => iframe,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("iframe youtube not found\n{}", e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };
            let _ = iframe.enter_frame().await;

            let ytplay2 = match driver
                .wait_element(By::ClassName("ytp-large-play-button"), 5)
                .await
            {
                Ok(elem) => elem,
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("stage 2\nytbutton not clicked\n{}", e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };
            let _ = ytplay2.click().await;
            let _ = driver.enter_default_frame().await;

            let mut tmr = match driver.find(By::Id("timer-tr-block")).await {
                Ok(time) => match time.find(By::Id("tmr")).await {
                    Ok(tmr) => match tmr.text().await {
                        Ok(txt) => txt.parse::<i32>().unwrap(),
                        Err(_) => {
                            let _ = driver.close_window().await;
                            let _ = driver.switch_to_window(tab_origin).await;
                            return TaskResult::CONTINUE;
                        }
                    },
                    Err(_) => {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CONTINUE;
                    }
                },
                Err(e) => {
                    Log::error(
                        "TaskDriverSeofast->YOUTUBE",
                        &format!("tmr not found\n{}", e),
                    )
                    .await;
                    let _ = driver.close_window().await;
                    let _ = driver.switch_to_window(tab_origin).await;
                    let _ = driver
                        .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                        .await;
                    return TaskResult::CONTINUE;
                }
            };
            let mut tmr_error = 0;
            let mut tmr_old = 0;
            while tmr > 0 {
                p.tmr("YOUTUBE", &tmr.to_string()).await;
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
                tmr = match driver.find(By::Id("timer-tr-block")).await {
                    Ok(time) => match time.find(By::Id("tmr")).await {
                        Ok(tmr) => match tmr.text().await {
                            Ok(txt) => {
                                if !txt.is_empty() {
                                    txt.parse::<i32>().unwrap()
                                } else {
                                    0
                                }
                            }
                            Err(_) => {
                                let _ = driver.close_window().await;
                                let _ = driver.switch_to_window(tab_origin).await;
                                let _ = driver
                                    .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                                    .await;
                                return TaskResult::CONTINUE;
                            }
                        },
                        Err(_) => {
                            let _ = driver.close_window().await;
                            let _ = driver.switch_to_window(tab_origin).await;
                            let _ = driver
                                .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                                .await;
                            return TaskResult::CONTINUE;
                        }
                    },
                    Err(_) => {
                        let _ = driver.close_window().await;
                        let _ = driver.switch_to_window(tab_origin).await;
                        let _ = driver
                            .execute(format!("st_view_youtube('{}');", id_yt), Vec::new())
                            .await;
                        return TaskResult::CONTINUE;
                    }
                };
            }

            let succes_error = match driver.wait_element(By::Id("succes-error"), 5).await {
                Ok(elem) => elem,
                Err(e) => {
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
            };

            let span2 = match succes_error.wait_element(By::Tag("span"), 10).await {
                Ok(elem) => elem,
                Err(e) => {
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
            };
            let earn = span2.text().await.unwrap();
            p.earn(&earn).await;
            let _ = driver.close_window().await;
            let _ = driver.switch_to_window(tab_origin).await;
            return TaskResult::OK;
        } else {
            //new mode exec
            todo!()
        }
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
                    //software_status(Status::STOP, Mode::SOFTWARE).await;
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
                //exit(0);
                return ();
            }

            match user_task.clone().login().await {
                Ok(driver) => loop {
                    if GLOBAL_CONTROL.load(Ordering::Relaxed) {
                        //exit(0);
                        return ();
                    }
                    let youtube =
                        TaskDriverSeofast::youtube(driver.clone(), &task_number.to_string()).await;
                    match youtube {
                        TaskResult::CONTINUE => continue,
                        TaskResult::PAUSE => {
                            Print::pause().await;
                            break;
                        }
                        TaskResult::OK => {
                            task_number += 1;
                            continue;
                        }
                        TaskResult::LOGIN_ERROR => {
                            let _ = driver.clone().quit().await;
                            break;
                        }
                        TaskResult::CRITICAL => {
                            let _ = driver.clone().quit().await;
                            break;
                        }
                        TaskResult::QUIT => break,
                    }
                },
                Err(e) => {
                    match e.as_inner() {
                        WebDriverErrorInner::NoSuchElement(e) => {
                            Log::error("TaskControlSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return ();
                            continue;
                        }
                        WebDriverErrorInner::Timeout(e) => {
                            Log::error("TaskControlSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return ();
                            continue;
                        }
                        WebDriverErrorInner::WebDriverTimeout(e) => {
                            Log::error("TaskControlSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return ();
                            continue;
                        }
                        WebDriverErrorInner::InvalidCookieDomain(e) => {
                            Log::error("TaskControlSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return ();
                            continue;
                        }
                        _ => {
                            Log::error("TaskControlSeofast->YOUTUBE", &format!("{}", e)).await;
                            //return ();
                            break;
                        }
                    }
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
            match ctrl_c().await {
                Ok(()) => GLOBAL_CONTROL.store(true, Ordering::SeqCst),
                Err(e) => eprintln!("{}", e),
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
