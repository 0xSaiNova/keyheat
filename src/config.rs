use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_idle_threshold_secs")]
    pub idle_threshold_secs: u64,

    #[serde(default = "default_wpm_window_secs")]
    pub wpm_window_secs: u64,

    #[serde(default = "default_wpm_sample_interval_secs")]
    pub wpm_sample_interval_secs: u64,
}

fn default_idle_threshold_secs() -> u64 {
    30
}

fn default_wpm_window_secs() -> u64 {
    30
}

fn default_wpm_sample_interval_secs() -> u64 {
    10
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idle_threshold_secs: default_idle_threshold_secs(),
            wpm_window_secs: default_wpm_window_secs(),
            wpm_sample_interval_secs: default_wpm_sample_interval_secs(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();

        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }

        Self::default()
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyheat")
            .join("config.toml")
    }

    #[allow(dead_code)]
    pub fn create_default_if_missing() -> Result<(), std::io::Error> {
        let path = Self::config_path();

        if path.exists() {
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;

            // Set directory permissions to owner-only on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(parent)?.permissions();
                perms.set_mode(0o700);
                fs::set_permissions(parent, perms)?;
            }
        }

        let default_config = Self::default();
        let toml_string = toml::to_string_pretty(&default_config).unwrap();
        fs::write(&path, toml_string)?;

        Ok(())
    }
}
