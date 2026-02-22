use std::env;
use std::sync;

use serde::Deserialize;

pub static CONFIG: sync::OnceLock<ClaudeStatusLineConfig> = sync::OnceLock::new();

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ClaudeStatusLineConfig {
    pub cost: Option<ClaudeStatusLineComponentConfig>,
    pub duration: Option<ClaudeStatusLineComponentConfig>,
    pub model: Option<ClaudeStatusLineComponentConfig>,
    pub percentage: Option<ClaudeStatusLineComponentConfig>,
    pub tokens: Option<ClaudeStatusLineComponentConfig>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ClaudeStatusLineComponentConfig {
    color: Option<u8>,
    icon: Option<String>,
}

impl ClaudeStatusLineComponentConfig {
    pub fn get_color_or(&self, default: u8) -> u8 {
        self.color.unwrap_or(default)
    }

    pub fn get_icon_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.icon.as_deref().unwrap_or(default)
    }
}

pub fn get_config() -> &'static ClaudeStatusLineConfig {
    CONFIG.get_or_init(|| {
        if cfg!(test) {
            ClaudeStatusLineConfig::default()
        } else {
            load_config()
        }
    })
}

fn load_config() -> ClaudeStatusLineConfig {
    let home = env::var("HOME").unwrap_or_default();
    let config_path = format!("{home}/.claude/statusline.toml");
    load_config_from(&config_path)
}

fn load_config_from(config_path: &str) -> ClaudeStatusLineConfig {
    let settings = ::config::Config::builder()
        .add_source(::config::File::with_name(config_path).required(false))
        .add_source(::config::Environment::with_prefix("CLAUDE_STATUSLINE").separator("_"))
        .build()
        .unwrap();

    settings
        .try_deserialize::<ClaudeStatusLineConfig>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    // Serialize all tests that call load_config_from — it reads env vars, so parallel
    // execution risks flaky cross-test interference.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn env_overrides_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("statusline.toml");
        std::fs::write(&path, "[tokens]\nicon = \"toml_icon\"\n").unwrap();

        let _lock = ENV_MUTEX.lock().unwrap();
        let prev = env::var("CLAUDE_STATUSLINE_TOKENS_ICON").ok();
        // Safety: ENV_MUTEX serializes all env var access in tests.
        unsafe { env::set_var("CLAUDE_STATUSLINE_TOKENS_ICON", "env_icon") };

        let config = load_config_from(path.to_str().unwrap());

        // Safety: ENV_MUTEX serializes all env var access in tests.
        unsafe {
            match prev {
                Some(v) => env::set_var("CLAUDE_STATUSLINE_TOKENS_ICON", v),
                None => env::remove_var("CLAUDE_STATUSLINE_TOKENS_ICON"),
            }
        }

        assert_eq!(Some("env_icon".to_string()), config.tokens.unwrap().icon);
    }

    #[test]
    fn get_color_or_returns_configured_value() {
        let config = ClaudeStatusLineComponentConfig {
            color: Some(100),
            icon: None,
        };
        assert_eq!(100, config.get_color_or(42));
    }

    #[test]
    fn get_color_or_returns_default_when_unset() {
        let config = ClaudeStatusLineComponentConfig::default();
        assert_eq!(42, config.get_color_or(42));
    }

    #[test]
    fn get_icon_or_returns_configured_value() {
        let config = ClaudeStatusLineComponentConfig {
            color: None,
            icon: Some("+".to_string()),
        };
        assert_eq!("+", config.get_icon_or("🤖"));
    }

    #[test]
    fn get_icon_or_returns_default_when_unset() {
        let config = ClaudeStatusLineComponentConfig::default();
        assert_eq!("🤖", config.get_icon_or("🤖"));
    }

    #[test]
    fn load_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let prev = env::var("CLAUDE_STATUSLINE_COST_COLOR").ok();
        // Safety: ENV_MUTEX serializes all env var access in tests.
        unsafe { env::set_var("CLAUDE_STATUSLINE_COST_COLOR", "200") };

        let config = load_config_from("/nonexistent/statusline.toml");

        // Safety: ENV_MUTEX serializes all env var access in tests.
        unsafe {
            match prev {
                Some(v) => env::set_var("CLAUDE_STATUSLINE_COST_COLOR", v),
                None => env::remove_var("CLAUDE_STATUSLINE_COST_COLOR"),
            }
        }

        assert_eq!(Some(200), config.cost.unwrap().color);
    }

    #[test]
    fn load_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("statusline.toml");
        std::fs::write(&path, "[duration]\ncolor = 39\nicon = \"T\"\n").unwrap();

        let _lock = ENV_MUTEX.lock().unwrap();
        let config = load_config_from(path.to_str().unwrap());

        let duration = config.duration.unwrap();
        assert_eq!(Some(39), duration.color);
        assert_eq!(Some("T".to_string()), duration.icon);
    }
}
