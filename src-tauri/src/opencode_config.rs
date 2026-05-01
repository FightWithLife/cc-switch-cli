use crate::config::write_json_file;
use crate::error::AppError;
use crate::provider::OpenCodeProviderConfig;
use crate::settings::get_opencode_override_dir;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
fn test_opencode_dir() -> &'static Mutex<Option<PathBuf>> {
    static DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    DIR.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn set_test_opencode_dir(path: Option<PathBuf>) {
    *test_opencode_dir()
        .lock()
        .expect("测试 OpenCode 目录锁失败") = path;
}

pub fn get_opencode_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(dir) = test_opencode_dir()
        .lock()
        .expect("测试 OpenCode 目录锁失败")
        .clone()
    {
        return dir;
    }

    if let Some(override_dir) = get_opencode_override_dir() {
        return override_dir;
    }

    crate::config::home_dir()
        .map(|home| home.join(".config").join("opencode"))
        .unwrap_or_else(|| PathBuf::from(".config").join("opencode"))
}

pub fn get_opencode_config_path() -> PathBuf {
    get_opencode_dir().join("opencode.json")
}

pub fn read_opencode_config() -> Result<Value, AppError> {
    let path = get_opencode_config_path();
    if !path.exists() {
        return Ok(json!({ "$schema": "https://opencode.ai/config.json" }));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
    serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))
}

pub fn write_opencode_config(config: &Value) -> Result<(), AppError> {
    let path = get_opencode_config_path();
    write_json_file(&path, config)
}

pub fn get_providers() -> Result<Map<String, Value>, AppError> {
    let config = read_opencode_config()?;
    Ok(config
        .get("provider")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

pub fn set_provider(id: &str, provider: Value) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;

    if full_config.get("provider").is_none() {
        full_config["provider"] = json!({});
    }

    if let Some(providers) = full_config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
    {
        providers.insert(id.to_string(), provider);
    }

    write_opencode_config(&full_config)
}

pub fn remove_provider(id: &str) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;
    if let Some(providers) = full_config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
    {
        providers.remove(id);
    }
    write_opencode_config(&full_config)
}

pub fn get_typed_providers() -> Result<IndexMap<String, OpenCodeProviderConfig>, AppError> {
    let mut result = IndexMap::new();
    for (id, value) in get_providers()? {
        match serde_json::from_value::<OpenCodeProviderConfig>(value) {
            Ok(config) => {
                result.insert(id, config);
            }
            Err(err) => {
                log::warn!("Failed to parse OpenCode provider '{id}': {err}");
            }
        }
    }
    Ok(result)
}

pub fn set_typed_provider(id: &str, config: &OpenCodeProviderConfig) -> Result<(), AppError> {
    let value =
        serde_json::to_value(config).map_err(|source| AppError::JsonSerialize { source })?;
    set_provider(id, value)
}

/// @brief 读取指定 OpenCode provider 的 currentModel。
/// @param provider_id provider ID。
/// @return 找到时返回 currentModel，否则返回 None。
pub fn get_current_model(provider_id: &str) -> Result<Option<String>, AppError> {
    Ok(get_providers()?
        .get(provider_id)
        .and_then(|provider| provider.get("currentModel"))
        .and_then(Value::as_str)
        .map(str::to_string))
}

/// @brief 更新指定 OpenCode provider 的 currentModel。
/// @param provider_id provider ID。
/// @param model_id 新的 model ID。
/// @return 更新成功返回 Ok。
pub fn set_current_model(provider_id: &str, model_id: &str) -> Result<(), AppError> {
    let mut config = read_opencode_config()?;
    if config.get("provider").is_none() {
        config["provider"] = json!({});
    }
    let providers = config
        .get_mut("provider")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| AppError::Config("OpenCode provider map is not an object".into()))?;
    let provider = providers
        .entry(provider_id.to_string())
        .or_insert_with(|| json!({}));
    if !provider.is_object() {
        *provider = json!({});
    }
    if let Some(provider_obj) = provider.as_object_mut() {
        provider_obj.insert("currentModel".to_string(), json!(model_id));
    }
    write_opencode_config(&config)
}

pub fn get_mcp_servers() -> Result<Map<String, Value>, AppError> {
    let config = read_opencode_config()?;
    Ok(config
        .get("mcp")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

pub fn set_mcp_server(id: &str, server: Value) -> Result<(), AppError> {
    let mut full_config = read_opencode_config()?;

    if full_config.get("mcp").is_none() {
        full_config["mcp"] = json!({});
    }

    if let Some(mcp) = full_config.get_mut("mcp").and_then(Value::as_object_mut) {
        mcp.insert(id.to_string(), server);
    }

    write_opencode_config(&full_config)
}

pub fn remove_mcp_server(id: &str) -> Result<(), AppError> {
    let mut config = read_opencode_config()?;

    if let Some(mcp) = config.get_mut("mcp").and_then(Value::as_object_mut) {
        mcp.remove(id);
    }

    write_opencode_config(&config)
}

#[cfg(test)]
mod current_model_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn current_model_helpers_read_and_update_only_target_provider() {
        let temp = tempfile::tempdir().expect("tempdir");
        set_test_opencode_dir(Some(temp.path().join("opencode")));
        write_opencode_config(&json!({
            "provider": {
                "demo": { "npm": "@ai-sdk/openai-compatible", "currentModel": "old" },
                "other": { "npm": "@ai-sdk/openai-compatible", "currentModel": "kept" }
            }
        }))
        .expect("seed config");

        assert_eq!(
            get_current_model("demo").expect("read"),
            Some("old".to_string())
        );
        set_current_model("demo", "new").expect("update");

        let config = read_opencode_config().expect("read config");
        assert_eq!(config["provider"]["demo"]["currentModel"], "new");
        assert_eq!(config["provider"]["other"]["currentModel"], "kept");
        set_test_opencode_dir(None);
    }
}
