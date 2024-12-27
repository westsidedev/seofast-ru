#[allow(non_snake_case, dead_code)]
pub struct Colors {
    pub CLOSE: String,
    pub RED: String,
    pub BLUE: String,
    pub CIAN: String,
    pub YELLOW: String,
    pub GREEN: String,
    pub WHITE: String,
    pub PINK: String,
}

impl Colors {
    pub async fn new() -> Colors {
        Colors {
            CLOSE: "\x1b[m".to_string(),
            CIAN: "\x1b[01;36m".to_string(),
            BLUE: "\x1b[01;34m".to_string(),
            YELLOW: "\x1b[01;33m".to_string(),
            GREEN: "\x1b[01;32m".to_string(),
            WHITE: "\x1b[01;37m".to_string(),
            RED: "\x1b[01;31m".to_string(),
            PINK: "\x1b[01;35m".to_string(),
        }
    }
}
