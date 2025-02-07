pub mod print;
pub mod task;

use std::io::{stdout, Write};
use task::ThreadTaskSeofast;

use crate::modules::colors::Colors;

#[allow(dead_code)]
pub enum Mode {
    YOUTUBE,
    SURFING,
    ALL,
}

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
