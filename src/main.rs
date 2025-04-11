mod guit;
mod json;
mod command_parser;
mod command_sender;

use std::sync::{LazyLock, Arc, Mutex};
use std::env::current_dir;

use json::{parse_json, Null, Value};
use json::Getter;

static LANG: LazyLock<Arc<Mutex<Lang>>> = LazyLock::new(|| Arc::new(Mutex::new(Lang::Français)));
static ERROR: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
static TEXT: LazyLock<Value> = LazyLock::new(|| {
    match parse_json(std::str::from_utf8(include_bytes!("../assets/text.json")).unwrap()) {
        Ok(v) => v,
        Err(e) => {
            ERROR.lock().unwrap().push_str(&e.to_string());
            Value::Null(Null)
        }
    }
});


#[derive(Debug, PartialEq)]
pub enum Lang {
    Français,
    English,
}

impl Lang {
    fn get_code(&self) -> String {
        match self {
            Lang::Français => "fr".to_string(),
            Lang::English => "en".to_string(),
        }
    }
}

const NOT_FOUND_INDEX: &str = "not_found";
/// # Panics
///
/// Can't panic.
#[must_use]
pub fn search_content(indexes: &[&str]) -> String {
    let lang_code = crate::LANG.lock().unwrap().get_code();
    let not_found_text = crate::TEXT[lang_code.as_str()][NOT_FOUND_INDEX].to_string();

    let not_found = crate::json::Value::String(not_found_text);
    let mut result = &crate::TEXT[lang_code];

    for index in indexes {
        if let Some(res) = result.get(*index) {
            result = res;
        } else {
            return not_found.to_string();
        }
    }

    result.to_string()
}

#[macro_export]
macro_rules! t {
    ($($args:tt),*) => {
        $crate::search_content(&[$($args),+])
    };
}

fn main() -> eframe::Result {
    let current_dir = current_dir().unwrap_or_default();
    env_logger::init();

    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([400.0, 300.0])
        .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Guit",
        native,
        Box::new(|cc| Ok(Box::new(crate::guit::Guit::new(cc, current_dir))))
    )
}
