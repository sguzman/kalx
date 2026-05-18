use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::kalshi::Signer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub profile: String,
    pub profiles: BTreeMap<String, ProfileConfig>,
    pub output: OutputConfig,
    pub logging: LoggingConfig,
    pub api_key_id: Option<String>,
    pub private_key_path: Option<String>,
    pub loaded_config_path: Option<String>,
    pub loaded_env_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub rest_base_url: String,
    pub ws_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthState {
    pub api_key_present: bool,
    pub private_key_present: bool,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidationReport {
    pub profile_exists: bool,
    pub rest_base_url_valid: bool,
    pub ws_base_url_valid: bool,
    pub api_key_present: bool,
    pub private_key_present: bool,
    pub private_key_exists: bool,
    pub private_key_parse_ok: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    profile: Option<String>,
    profiles: Option<BTreeMap<String, ProfileConfig>>,
    output: Option<OutputConfig>,
    logging: Option<LoggingConfig>,
}

impl AppConfig {
    pub fn load(config_path: Option<&str>, env_path: Option<&str>, cli_profile: Option<&str>) -> Result<Self> {
        let loaded_env_path = load_env_file(env_path)?;

        let mut config = Self::defaults();
        let file_path = resolve_config_path(config_path);
        let loaded_config_path = if let Some(path) = file_path.as_ref().filter(|path| path.exists()) {
            let text = fs::read_to_string(path)
                .with_context(|| format!("failed to read config file {}", path.display()))?;
            let partial: PartialConfig = toml::from_str(&text)
                .with_context(|| format!("failed to parse config file {}", path.display()))?;
            if let Some(profile) = partial.profile {
                config.profile = profile;
            }
            if let Some(profiles) = partial.profiles {
                config.profiles.extend(profiles);
            }
            if let Some(output) = partial.output {
                config.output = output;
            }
            if let Some(logging) = partial.logging {
                config.logging = logging;
            }
            Some(path.display().to_string())
        } else {
            None
        };

        if let Some(profile) = env::var("KALSHI_ENV").ok().filter(|value| !value.is_empty()) {
            config.profile = profile;
        }
        if let Some(level) = env::var("KALX_LOG").ok().filter(|value| !value.is_empty()) {
            config.logging.level = level;
        }
        if let Some(output) = env::var("KALX_OUTPUT").ok().filter(|value| !value.is_empty()) {
            config.output.format = output;
        }
        config.api_key_id = env::var("KALSHI_API_KEY_ID").ok().filter(|value| !value.is_empty());
        config.private_key_path = env::var("KALSHI_PRIVATE_KEY_PATH").ok().filter(|value| !value.is_empty());

        if let Some(profile) = cli_profile {
            config.profile = profile.to_string();
        }

        config.loaded_config_path = loaded_config_path;
        config.loaded_env_path = loaded_env_path;
        Ok(config)
    }

    pub fn defaults() -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "demo".to_string(),
            ProfileConfig {
                rest_base_url: "https://external-api.demo.kalshi.co/trade-api/v2".to_string(),
                ws_base_url: "wss://external-api-ws.demo.kalshi.co/trade-api/ws/v2".to_string(),
            },
        );
        profiles.insert(
            "prod".to_string(),
            ProfileConfig {
                rest_base_url: "https://external-api.kalshi.com/trade-api/v2".to_string(),
                ws_base_url: "wss://external-api-ws.kalshi.com/trade-api/ws/v2".to_string(),
            },
        );

        Self {
            profile: "demo".to_string(),
            profiles,
            output: OutputConfig { format: "table".to_string() },
            logging: LoggingConfig { level: "info".to_string(), json: false },
            api_key_id: None,
            private_key_path: None,
            loaded_config_path: None,
            loaded_env_path: None,
        }
    }

    pub fn active_profile(&self) -> &ProfileConfig {
        self.profiles
            .get(&self.profile)
            .or_else(|| self.profiles.get("demo"))
            .expect("default demo profile must exist")
    }

    pub fn auth_state(&self) -> AuthState {
        let api_key_present = self.api_key_id.as_ref().is_some_and(|value| !value.is_empty());
        let private_key_present = self.private_key_path.as_ref().is_some_and(|value| !value.is_empty());
        AuthState { api_key_present, private_key_present, ready: api_key_present && private_key_present }
    }

    pub fn validate(&self) -> ConfigValidationReport {
        let mut errors = Vec::new();
        let profile_exists = self.profiles.contains_key(&self.profile);
        if !profile_exists {
            errors.push(format!("unknown profile '{}'", self.profile));
        }

        let active = self.active_profile();
        let rest_base_url_valid = reqwest::Url::parse(&active.rest_base_url).is_ok();
        if !rest_base_url_valid {
            errors.push("invalid rest_base_url".to_string());
        }
        let ws_base_url_valid = reqwest::Url::parse(&active.ws_base_url).is_ok();
        if !ws_base_url_valid {
            errors.push("invalid ws_base_url".to_string());
        }

        let auth = self.auth_state();
        let private_key_exists = self.private_key_path.as_ref().is_some_and(|path| Path::new(path).exists());
        if auth.private_key_present && !private_key_exists {
            errors.push("private key path does not exist".to_string());
        }

        let private_key_parse_ok = match (&self.api_key_id, &self.private_key_path) {
            (Some(api_key_id), Some(private_key_path)) => Signer::new(api_key_id, private_key_path).is_ok(),
            _ => false,
        };
        if auth.ready && !private_key_parse_ok {
            errors.push("private key could not be parsed".to_string());
        }

        ConfigValidationReport {
            profile_exists,
            rest_base_url_valid,
            ws_base_url_valid,
            api_key_present: auth.api_key_present,
            private_key_present: auth.private_key_present,
            private_key_exists,
            private_key_parse_ok,
            errors,
        }
    }

    pub fn ensure_mutation_allowed(&self, live: bool, yes: bool) -> Result<()> {
        if !live {
            return Ok(());
        }
        if self.profile == "prod" && !yes {
            return Err(anyhow!("prod mutations require both --live and --yes"));
        }
        Ok(())
    }

    pub fn default_config_toml() -> &'static str {
        include_str!("../config/kalx.example.toml")
    }
}

fn load_env_file(env_path: Option<&str>) -> Result<Option<String>> {
    if let Some(path) = env_path {
        dotenvy::from_path_override(path).with_context(|| format!("failed to load env file {path}"))?;
        return Ok(Some(path.to_string()));
    }

    let candidate = Path::new(".env");
    if candidate.exists() {
        dotenvy::from_path_override(candidate)
            .with_context(|| format!("failed to load env file {}", candidate.display()))?;
        return Ok(Some(candidate.display().to_string()));
    }

    Ok(None)
}

fn resolve_config_path(config_path: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = config_path {
        return Some(PathBuf::from(path));
    }

    ProjectDirs::from("dev", "openai", "kalx").map(|dirs| dirs.config_dir().join("kalx.toml"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::AppConfig;

    #[test]
    fn parses_default_config_template() {
        let _: toml::Value = toml::from_str(AppConfig::default_config_toml()).unwrap();
    }

    #[test]
    fn validate_flags_missing_key_file() {
        let mut config = AppConfig::defaults();
        let dir = tempdir().unwrap();
        let fake_key = dir.path().join("missing.pem");
        config.api_key_id = Some("abc".into());
        config.private_key_path = Some(fake_key.display().to_string());
        let report = config.validate();
        assert!(!report.private_key_exists);
    }

    #[test]
    fn load_env_file_from_repo_dotenv() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".env"), "KALSHI_ENV=demo\n").unwrap();
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let config = AppConfig::load(None, None, None).unwrap();
        std::env::set_current_dir(cwd).unwrap();
        assert_eq!(config.profile, "demo");
    }
}
