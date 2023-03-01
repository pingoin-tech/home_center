use fireplace::config::ConfigFile;
use std::{fs, sync::Mutex};
use toml;

type PublicStore = Mutex<Option<Store>>;
pub static STORE: PublicStore = Mutex::new(None);

const CONFIG_FILE: &str = "setup.toml";
fn read_config() -> ConfigFile {
    let mut fileok = false;
    let mut result: ConfigFile = ConfigFile::default();
    if let Ok(file) = fs::read_to_string(CONFIG_FILE) {
        if let Ok(config) = toml::from_str(&file) {
            result = config;
            fileok = false;
        }
    }

    if !fileok {
        let data = toml::to_string_pretty(&result);
        if let Ok(content) = data {
            _ = fs::write(CONFIG_FILE, content);
        }
    }
    result
}

pub fn init_store() {
    let config = read_config();

    STORE
        .lock()
        .expect("could not lock")
        .get_or_insert(Store { config: config });
}

pub struct Store {
    pub config: ConfigFile,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            config: Default::default(),
        }
    }
}
