use crate::app_config::AppType;
use crate::provider::Provider;
use serde_json::Value;

use super::codex_config::parse_codex_config_snippet;
use super::{ClaudeApiFormat, ProviderAddFormState, OPENCLAW_DEFAULT_API_PROTOCOL};

pub(super) fn populate_form_from_provider(
    form: &mut ProviderAddFormState,
    app_type: &AppType,
    provider: &Provider,
) {
    match app_type {
        AppType::Claude => populate_claude_form(form, provider),
        AppType::Codex => populate_codex_form(form, provider),
        AppType::Gemini => populate_gemini_form(form, provider),
        AppType::OpenCode => populate_opencode_form(form, provider),
        AppType::OpenClaw => populate_openclaw_form(form, provider),
    }
}

fn populate_claude_form(form: &mut ProviderAddFormState, provider: &Provider) {
    form.claude_api_format = parse_claude_api_format(provider);
    if let Some(env) = provider
        .settings_config
        .get("env")
        .and_then(|value| value.as_object())
    {
        if let Some(token) = env
            .get("ANTHROPIC_AUTH_TOKEN")
            .and_then(|value| value.as_str())
        {
            form.claude_api_key.set(token);
        }
        if let Some(url) = env
            .get("ANTHROPIC_BASE_URL")
            .and_then(|value| value.as_str())
        {
            form.claude_base_url.set(url);
        }
        if let Some(model) = env.get("ANTHROPIC_MODEL").and_then(|value| value.as_str()) {
            form.claude_model.set(model);
        }
        if let Some(reasoning) = env
            .get("ANTHROPIC_REASONING_MODEL")
            .and_then(|value| value.as_str())
        {
            form.claude_reasoning_model.set(reasoning);
        }

        let model = env.get("ANTHROPIC_MODEL").and_then(|value| value.as_str());
        let small_fast = env
            .get("ANTHROPIC_SMALL_FAST_MODEL")
            .and_then(|value| value.as_str());

        if let Some(haiku) = env
            .get("ANTHROPIC_DEFAULT_HAIKU_MODEL")
            .and_then(|value| value.as_str())
            .or(small_fast)
            .or(model)
        {
            form.claude_haiku_model.set(haiku);
        }
        if let Some(sonnet) = env
            .get("ANTHROPIC_DEFAULT_SONNET_MODEL")
            .and_then(|value| value.as_str())
            .or(model)
            .or(small_fast)
        {
            form.claude_sonnet_model.set(sonnet);
        }
        if let Some(opus) = env
            .get("ANTHROPIC_DEFAULT_OPUS_MODEL")
            .and_then(|value| value.as_str())
            .or(model)
            .or(small_fast)
        {
            form.claude_opus_model.set(opus);
        }
    }
}

fn populate_codex_form(form: &mut ProviderAddFormState, provider: &Provider) {
    if let Some(config) = provider
        .settings_config
        .get("config")
        .and_then(|value| value.as_str())
    {
        let parsed = parse_codex_config_snippet(config);
        if let Some(base_url) = parsed.base_url {
            form.codex_base_url.set(base_url);
        }
        if let Some(model) = parsed.model {
            form.codex_model.set(model);
        }
        if let Some(wire_api) = parsed.wire_api {
            form.codex_wire_api = wire_api;
        }
        if let Some(requires_openai_auth) = parsed.requires_openai_auth {
            form.codex_requires_openai_auth = requires_openai_auth;
        }
        if let Some(env_key) = parsed.env_key {
            form.codex_env_key.set(env_key);
        }
    }
    if let Some(auth) = provider
        .settings_config
        .get("auth")
        .and_then(|value| value.as_object())
    {
        if let Some(key) = auth.get("OPENAI_API_KEY").and_then(|value| value.as_str()) {
            form.codex_api_key.set(key);
        }
    }
}

fn populate_gemini_form(form: &mut ProviderAddFormState, provider: &Provider) {
    if let Some(env) = provider
        .settings_config
        .get("env")
        .and_then(|value| value.as_object())
    {
        if let Some(key) = env.get("GEMINI_API_KEY").and_then(|value| value.as_str()) {
            form.gemini_auth_type = super::GeminiAuthType::ApiKey;
            form.gemini_api_key.set(key);
        } else {
            form.gemini_auth_type = super::GeminiAuthType::OAuth;
        }

        if let Some(url) = env
            .get("GOOGLE_GEMINI_BASE_URL")
            .or_else(|| env.get("GEMINI_BASE_URL"))
            .and_then(|value| value.as_str())
        {
            form.gemini_base_url.set(url);
        }

        if let Some(model) = env.get("GEMINI_MODEL").and_then(|value| value.as_str()) {
            form.gemini_model.set(model);
        }
    } else {
        form.gemini_auth_type = super::GeminiAuthType::OAuth;
    }
}

fn populate_opencode_form(form: &mut ProviderAddFormState, provider: &Provider) {
    if let Some(npm) = provider
        .settings_config
        .get("npm")
        .and_then(|value| value.as_str())
    {
        form.opencode_npm_package.set(npm);
    }
    if let Some(options) = provider
        .settings_config
        .get("options")
        .and_then(|value| value.as_object())
    {
        if let Some(api_key) = options.get("apiKey").and_then(|value| value.as_str()) {
            form.opencode_api_key.set(api_key);
        }
        if let Some(base_url) = options.get("baseURL").and_then(|value| value.as_str()) {
            form.opencode_base_url.set(base_url);
        }
    }

    // 读取 currentModel（位于 settingsConfig.currentModel）
    let current_model = provider
        .settings_config
        .get("currentModel")
        .and_then(|value| value.as_str())
        .map(|s| s.to_string());

    // 加载所有 models 到 opencode_models 列表
    if let Some(models) = provider
        .settings_config
        .get("models")
        .and_then(|value| value.as_object())
    {
        let mut loaded_models = Vec::new();
        let mut model_values: Vec<(String, &Value)> = Vec::new();
        for (model_id, model_value) in models {
            let mut draft = crate::provider::OpenCodeModelDraft::new(model_id.clone());
            draft.original_model_id = Some(model_id.clone());
            if let Some(name) = model_value.get("name").and_then(|value| value.as_str()) {
                draft.model_name = name.to_string();
            }
            if let Some(limit) = model_value.get("limit").and_then(|value| value.as_object()) {
                draft.input_limit = limit.get("context").and_then(|value| value.as_u64());
                draft.output_limit = limit.get("output").and_then(|value| value.as_u64());
            }
            // 保留额外字段（如 options、userAgent 等）
            if let Some(obj) = model_value.as_object() {
                let mut extra = serde_json::Map::new();
                for (key, value) in obj {
                    if key != "name" && key != "limit" {
                        extra.insert(key.clone(), value.clone());
                    }
                }
                if !extra.is_empty() {
                    draft.extra = serde_json::Value::Object(extra);
                }
            }
            model_values.push((model_id.clone(), model_value));
            loaded_models.push(draft);
        }
        // 使用与原代码一致的排序逻辑：rank 降序，然后 model_id 降序
        // 这与原代码的 max_by 逻辑一致（rank 高的排在前面）
        loaded_models.sort_by(|a, b| {
            let rank_a = model_values
                .iter()
                .find(|(id, _)| id == &a.model_id)
                .map(|(_, v)| opencode_model_rank(v))
                .unwrap_or(0);
            let rank_b = model_values
                .iter()
                .find(|(id, _)| id == &b.model_id)
                .map(|(_, v)| opencode_model_rank(v))
                .unwrap_or(0);
            rank_b
                .cmp(&rank_a)
                .then_with(|| b.model_id.cmp(&a.model_id))
        });
        form.opencode_models = loaded_models;
    }

    // 设置 flat fields（用于 ProviderForm 中的 ModelConfig 入口行显示，以及向后兼容）
    // 优先使用 currentModel，否则使用排序后的第一个 model
    let active_model_id =
        current_model.or_else(|| form.opencode_models.first().map(|m| m.model_id.clone()));

    if let Some(ref model_id) = active_model_id {
        form.opencode_model_original_id = Some(model_id.clone());
        form.opencode_model_id.set(model_id);
        if let Some(draft) = form
            .opencode_models
            .iter()
            .find(|m| &m.model_id == model_id)
        {
            if !draft.model_name.is_empty() {
                form.opencode_model_name.set(&draft.model_name);
            } else {
                form.opencode_model_name.set(model_id);
            }
            if let Some(context) = draft.input_limit {
                form.opencode_model_context_limit.set(context.to_string());
            }
            if let Some(output) = draft.output_limit {
                form.opencode_model_output_limit.set(output.to_string());
            }
        }
    }
}

fn populate_openclaw_form(form: &mut ProviderAddFormState, provider: &Provider) {
    if let Some(api_key) = provider
        .settings_config
        .get("apiKey")
        .and_then(|value| value.as_str())
    {
        form.opencode_api_key.set(api_key);
    }
    if let Some(base_url) = provider
        .settings_config
        .get("baseUrl")
        .and_then(|value| value.as_str())
    {
        form.opencode_base_url.set(base_url);
    }
    if let Some(api) = provider
        .settings_config
        .get("api")
        .and_then(|value| value.as_str())
    {
        form.opencode_npm_package.set(api);
    } else {
        form.opencode_npm_package.set(OPENCLAW_DEFAULT_API_PROTOCOL);
    }
    if provider
        .settings_config
        .get("headers")
        .and_then(|value| value.as_object())
        .is_some_and(|headers| headers.contains_key("User-Agent"))
    {
        form.openclaw_user_agent = true;
    }
    if let Some(models) = provider
        .settings_config
        .get("models")
        .and_then(|value| value.as_array())
    {
        form.openclaw_models = models.clone();
    }
    if let Some(model) = form.openclaw_models.first() {
        if let Some(id) = model.get("id").and_then(|value| value.as_str()) {
            form.opencode_model_original_id = Some(id.to_string());
            form.opencode_model_id.set(id);
        }
        if let Some(name) = model.get("name").and_then(|value| value.as_str()) {
            form.opencode_model_name.set(name);
        }
        if let Some(context_window) = model.get("contextWindow").and_then(|value| value.as_u64()) {
            form.opencode_model_context_limit
                .set(context_window.to_string());
        }
    }
}

fn parse_claude_api_format(provider: &Provider) -> ClaudeApiFormat {
    if let Some(api_format) = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.api_format.as_deref())
    {
        return ClaudeApiFormat::from_raw(api_format);
    }

    if let Some(api_format) = provider
        .settings_config
        .get("api_format")
        .and_then(|value| value.as_str())
    {
        return ClaudeApiFormat::from_raw(api_format);
    }

    let compat_enabled = match provider.settings_config.get("openrouter_compat_mode") {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_i64().unwrap_or(0) != 0,
        Some(Value::String(value)) => {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "true" || normalized == "1"
        }
        _ => false,
    };

    if compat_enabled {
        ClaudeApiFormat::OpenAiChat
    } else {
        ClaudeApiFormat::Anthropic
    }
}

fn opencode_model_rank(model: &Value) -> usize {
    let mut score = 0;
    if model
        .get("limit")
        .and_then(|value| value.as_object())
        .map(|limit| !limit.is_empty())
        .unwrap_or(false)
    {
        score += 1;
    }
    if model
        .get("options")
        .and_then(|value| value.as_object())
        .map(|options| !options.is_empty())
        .unwrap_or(false)
    {
        score += 1;
    }
    score
}
