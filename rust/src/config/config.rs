use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

pub struct Config {
    pub utf8_lenient: bool,
    pub fts_path: String,
}

impl Config {
    fn default() -> Config {
        Config {
            utf8_lenient: false,
            fts_path: String::new(),
        }
    }

    pub fn update_utf8_lenient(&mut self, utf8_lenient: bool) {
        self.utf8_lenient = utf8_lenient;
    }

    pub fn update_fts_path(&mut self, fts_path: String) {
        self.fts_path = fts_path;
    }
}