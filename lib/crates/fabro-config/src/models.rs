use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};

const MODELS_FILE_NAME: &str = "models.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomProviderConfig {
    pub id: String,
    pub base_url: String,
    pub api_key_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomModelLimits {
    pub context_window: i64,
    #[serde(default)]
    pub max_output: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomModelFeatures {
    pub tools: bool,
    pub vision: bool,
    pub reasoning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomModelConfig {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub limits: CustomModelLimits,
    pub features: CustomModelFeatures,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomModelsConfig {
    #[serde(default)]
    pub providers: Vec<CustomProviderConfig>,
    #[serde(default)]
    pub models: Vec<CustomModelConfig>,
}

pub fn default_models_path() -> anyhow::Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;
    Ok(base.join("fabro").join(MODELS_FILE_NAME))
}

pub fn load_custom_models(path: Option<&Path>) -> anyhow::Result<CustomModelsConfig> {
    let path = match path {
        Some(p) => p.to_path_buf(),
        None => default_models_path()?,
    };

    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CustomModelsConfig::default())
        }
        Err(e) => {
            return Err(e).with_context(|| {
                format!("Failed reading custom models config at {}", path.display())
            })
        }
    };

    let config: CustomModelsConfig = serde_json::from_str(&contents)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;
    validate_custom_models(&config)?;
    Ok(config)
}

pub fn save_custom_models(
    config: &CustomModelsConfig,
    path: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    validate_custom_models(config)?;
    let path = match path {
        Some(p) => p.to_path_buf(),
        None => default_models_path()?,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let mut json = serde_json::to_string_pretty(config)?;
    json.push('\n');
    std::fs::write(&path, json).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(path)
}

pub fn validate_custom_models(config: &CustomModelsConfig) -> anyhow::Result<()> {
    let mut provider_ids = HashSet::new();
    for provider in &config.providers {
        validate_nonempty("provider id", &provider.id)?;
        validate_env_var_name(&provider.api_key_env)?;

        if provider.base_url.trim().is_empty() {
            bail!("Provider '{}' has an empty base_url", provider.id);
        }

        if !provider_ids.insert(provider.id.clone()) {
            bail!("Duplicate provider id '{}'", provider.id);
        }
    }

    let mut model_ids = HashSet::new();
    for model in &config.models {
        validate_nonempty("model id", &model.id)?;
        if !provider_ids.contains(&model.provider) {
            bail!(
                "Model '{}' references unknown provider '{}'",
                model.id,
                model.provider
            );
        }

        if model.display_name.trim().is_empty() {
            bail!("Model '{}' has an empty display_name", model.id);
        }

        if model.limits.context_window <= 0 {
            bail!(
                "Model '{}' must have a positive limits.context_window",
                model.id
            );
        }

        if !model_ids.insert(model.id.clone()) {
            bail!("Duplicate model id '{}'", model.id);
        }

        for alias in &model.aliases {
            validate_slug("model alias", alias)?;
        }
    }

    Ok(())
}

fn validate_slug(label: &str, value: &str) -> anyhow::Result<()> {
    validate_nonempty(label, value)?;
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!(
            "Invalid {label} '{}': use lowercase letters, numbers, and '-' only",
            value
        );
    }
    Ok(())
}

fn validate_nonempty(label: &str, value: &str) -> anyhow::Result<()> {
    if value.is_empty() {
        bail!("{label} cannot be empty");
    }
    Ok(())
}

fn validate_env_var_name(value: &str) -> anyhow::Result<()> {
    if value.is_empty() {
        bail!("api_key_env cannot be empty");
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        bail!("api_key_env cannot be empty");
    };

    if !first.is_ascii_uppercase() && first != '_' {
        bail!("Invalid api_key_env '{}': must start with A-Z or _", value);
    }

    if !chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
        bail!("Invalid api_key_env '{}': use A-Z, 0-9, and _ only", value);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> CustomModelsConfig {
        CustomModelsConfig {
            providers: vec![CustomProviderConfig {
                id: "my-provider".to_string(),
                base_url: "https://example.com/v1".to_string(),
                api_key_env: "MY_PROVIDER_API_KEY".to_string(),
            }],
            models: vec![CustomModelConfig {
                id: "my-model".to_string(),
                provider: "my-provider".to_string(),
                display_name: "My Model".to_string(),
                limits: CustomModelLimits {
                    context_window: 128000,
                    max_output: None,
                },
                features: CustomModelFeatures {
                    tools: true,
                    vision: false,
                    reasoning: false,
                },
                aliases: vec![],
            }],
        }
    }

    #[test]
    fn validates_sample_config() {
        let config = sample_config();
        validate_custom_models(&config).unwrap();
    }

    #[test]
    fn allows_provider_id_with_non_slug_characters() {
        let mut config = sample_config();
        config.providers[0].id = "My Provider_1".to_string();
        config.models[0].provider = "My Provider_1".to_string();
        validate_custom_models(&config).unwrap();
    }

    #[test]
    fn allows_model_id_with_provider_style_characters() {
        let mut config = sample_config();
        config.models[0].id = "hf:zai-org/GLM-4.7".to_string();
        validate_custom_models(&config).unwrap();
    }

    #[test]
    fn rejects_unknown_provider_reference() {
        let mut config = sample_config();
        config.models[0].provider = "missing".to_string();
        let err = validate_custom_models(&config).unwrap_err();
        assert!(err
            .to_string()
            .contains("references unknown provider 'missing'"));
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.json");
        let loaded = load_custom_models(Some(&path)).unwrap();
        assert_eq!(loaded, CustomModelsConfig::default());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("models.json");
        let config = sample_config();

        save_custom_models(&config, Some(&path)).unwrap();
        let loaded = load_custom_models(Some(&path)).unwrap();

        assert_eq!(loaded, config);
    }
}
