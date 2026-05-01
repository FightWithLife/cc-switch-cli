use super::*;

#[derive(Debug, Clone)]
pub struct FilterState {
    pub active: bool,
    pub buffer: String,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            active: false,
            buffer: String::new(),
        }
    }

    pub fn query_lower(&self) -> Option<String> {
        let trimmed = self.buffer.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(trimmed.to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Nav,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub remaining_ticks: u16,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            remaining_ticks: 12,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Quit,
    ProviderDelete {
        id: String,
    },
    McpDelete {
        id: String,
    },
    PromptDelete {
        id: String,
    },
    SkillsUninstall {
        directory: String,
    },
    SkillsRepoRemove {
        owner: String,
        name: String,
    },
    ConfigImport {
        path: String,
    },
    ConfigRestoreBackup {
        id: String,
    },
    ConfigReset,
    SettingsSetSkipClaudeOnboarding {
        enabled: bool,
    },
    SettingsSetClaudePluginIntegration {
        enabled: bool,
    },
    ProviderApiFormatProxyNotice,
    ProviderSwitchSharedConfigNotice,
    OpenClawDailyMemoryDelete {
        filename: String,
    },
    OpenCodeModelDelete {
        provider_id: String,
        model_id: String,
    },
    FormSaveBeforeClose,
    EditorDiscard,
    EditorSaveBeforeClose,
    WebDavMigrateV1ToV2,
}

#[derive(Debug, Clone)]
pub struct ConfirmOverlay {
    pub title: String,
    pub message: String,
    pub action: ConfirmAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSubmit {
    ConfigExport,
    ConfigImport,
    ConfigBackupName,
    SettingsProxyListenAddress,
    SettingsProxyListenPort,
    SettingsOpenClawConfigDir,
    SkillsInstallSpec,
    SkillsDiscoverQuery,
    SkillsRepoAdd,
    OpenClawDailyMemoryFilename,
    OpenClawToolsRule {
        section: OpenClawToolsSection,
        row: Option<usize>,
    },
    OpenClawAgentsRuntimeField {
        field: OpenClawAgentsRuntimeField,
    },
    WebDavJianguoyunUsername,
    WebDavJianguoyunPassword,
}

#[derive(Debug, Clone)]
pub struct TextInputState {
    pub title: String,
    pub prompt: String,
    pub buffer: String,
    pub submit: TextSubmit,
    pub secret: bool,
}

#[derive(Debug, Clone)]
pub struct TextViewState {
    pub title: String,
    pub lines: Vec<String>,
    pub scroll: usize,
    pub action: Option<TextViewAction>,
}

#[derive(Debug, Clone)]
pub enum TextViewAction {
    ProxyToggleTakeover { app_type: AppType, enabled: bool },
}

impl TextViewAction {
    pub fn key_label(&self) -> &'static str {
        match self {
            TextViewAction::ProxyToggleTakeover { enabled: true, .. } => texts::tui_key_takeover(),
            TextViewAction::ProxyToggleTakeover { enabled: false, .. } => texts::tui_key_restore(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingKind {
    Generic,
    Proxy,
    WebDav,
    UpdateCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpEnvEditorField {
    Key,
    Value,
}

#[derive(Debug, Clone)]
pub struct McpEnvEntryEditorState {
    pub row: Option<usize>,
    pub return_selected: usize,
    pub field: McpEnvEditorField,
    pub key: crate::cli::tui::form::TextInput,
    pub value: crate::cli::tui::form::TextInput,
}

impl McpEnvEntryEditorState {
    pub fn key_active(&self) -> bool {
        matches!(self.field, McpEnvEditorField::Key)
    }

    pub fn value_active(&self) -> bool {
        matches!(self.field, McpEnvEditorField::Value)
    }
}

#[derive(Debug, Clone)]
pub struct OpenCodeProviderDraft {
    pub provider_id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub npm: String,
    pub models_by_id: indexmap::IndexMap<String, crate::provider::OpenCodeModelDraft>,
    pub current_model: String,
    pub dirty: bool,
    pub last_loaded_from_db: serde_json::Value,
}

impl OpenCodeProviderDraft {
    /// @brief 判断 OpenCode provider draft 是否有未保存改动。
    /// @return 有改动返回 true，否则返回 false。
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// @brief 设置 OpenCode provider draft 的 dirty 标记。
    /// @param dirty 新的 dirty 状态。
    /// @return 无。
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// @brief 返回当前 draft 中的 model 数量。
    /// @return model 数量。
    pub fn model_count(&self) -> usize {
        self.models_by_id.len()
    }

    /// @brief 检查 draft 中是否存在指定 model。
    /// @param id model ID。
    /// @return 存在返回 true，否则返回 false。
    pub fn has_model(&self, id: &str) -> bool {
        self.models_by_id.contains_key(id)
    }

    /// @brief 添加新的 OpenCode model draft。
    /// @param draft 待添加的 model draft。
    /// @return 添加成功返回 Ok，ID 重复时返回错误文案。
    pub fn add_model(&mut self, draft: crate::provider::OpenCodeModelDraft) -> Result<(), String> {
        if self.has_model(&draft.model_id) {
            return Err(format!("Duplicated model id `{}`", draft.model_id));
        }
        self.models_by_id.insert(draft.model_id.clone(), draft);
        if self.current_model.trim().is_empty() {
            self.current_model = self.models_by_id.keys().next().cloned().unwrap_or_default();
        }
        self.dirty = true;
        Ok(())
    }

    /// @brief 移除指定 model，并在必要时修复 currentModel。
    /// @param id 要移除的 model ID。
    /// @return 找到并移除时返回 model draft，否则返回 None。
    pub fn remove_model(&mut self, id: &str) -> Option<crate::provider::OpenCodeModelDraft> {
        let removed = self.models_by_id.shift_remove(id)?;
        if self.current_model == id {
            self.current_model = self.models_by_id.keys().min().cloned().unwrap_or_default();
        }
        self.dirty = true;
        Some(removed)
    }

    /// @brief 重命名 model ID，并保持 map key 与 currentModel 同步。
    /// @param old_id 原 model ID。
    /// @param new_id 新 model ID。
    /// @return 重命名成功返回 Ok，否则返回错误文案。
    pub fn rename_model(&mut self, old_id: &str, new_id: String) -> Result<(), String> {
        let new_id = new_id.trim().to_string();
        if new_id.is_empty() {
            return Err("Model ID is required".to_string());
        }
        if old_id != new_id && self.has_model(&new_id) {
            return Err(format!("Duplicated model id `{new_id}`"));
        }
        let mut draft = self
            .models_by_id
            .shift_remove(old_id)
            .ok_or_else(|| format!("Model `{old_id}` not found"))?;
        draft.original_model_id = draft.original_model_id.or_else(|| Some(old_id.to_string()));
        draft.model_id = new_id.clone();
        self.models_by_id.insert(new_id.clone(), draft);
        if self.current_model == old_id {
            self.current_model = new_id;
        }
        self.dirty = true;
        Ok(())
    }

    /// @brief 从已持久化的 provider 初始化 OpenCode 编辑 draft。
    /// @param provider Provider 数据。
    /// @return 初始化后的 OpenCodeProviderDraft。
    pub fn from_provider(provider: &crate::provider::Provider) -> Self {
        let settings = &provider.settings_config;
        let npm = settings
            .get("npm")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("@ai-sdk/openai-compatible")
            .to_string();
        let options = settings
            .get("options")
            .and_then(serde_json::Value::as_object);
        let base_url = options
            .and_then(|value| value.get("baseURL"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let api_key = options
            .and_then(|value| value.get("apiKey"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();

        let mut models_by_id = indexmap::IndexMap::new();
        if let Some(models) = settings
            .get("models")
            .and_then(serde_json::Value::as_object)
        {
            for (model_id, model_value) in models {
                let mut draft = crate::provider::OpenCodeModelDraft::new(model_id.clone());
                draft.original_model_id = Some(model_id.clone());
                draft.model_name = model_value
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(model_id)
                    .to_string();
                if let Some(limit) = model_value
                    .get("limit")
                    .and_then(serde_json::Value::as_object)
                {
                    draft.input_limit = limit.get("context").and_then(serde_json::Value::as_u64);
                    draft.output_limit = limit.get("output").and_then(serde_json::Value::as_u64);
                }
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
                models_by_id.insert(model_id.clone(), draft);
            }
        }

        let current_model = settings
            .get("currentModel")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .or_else(|| models_by_id.keys().next().cloned())
            .unwrap_or_default();

        Self {
            provider_id: provider.id.clone(),
            name: provider.name.clone(),
            base_url,
            api_key,
            npm,
            models_by_id,
            current_model,
            dirty: false,
            last_loaded_from_db: settings.clone(),
        }
    }

    /// @brief 从 OpenCode 表单基础字段创建新建 provider 的 draft。
    /// @param form Provider 表单状态。
    /// @return 初始化后的 OpenCodeProviderDraft。
    pub fn from_form(form: &crate::cli::tui::form::ProviderAddFormState) -> Self {
        Self {
            provider_id: form.id.value.trim().to_string(),
            name: form.name.value.trim().to_string(),
            base_url: form.opencode_base_url.value.trim().to_string(),
            api_key: form.opencode_api_key.value.trim().to_string(),
            npm: if form.opencode_npm_package.value.trim().is_empty() {
                "@ai-sdk/openai-compatible".to_string()
            } else {
                form.opencode_npm_package.value.trim().to_string()
            },
            models_by_id: indexmap::IndexMap::new(),
            current_model: String::new(),
            dirty: false,
            last_loaded_from_db: serde_json::json!({}),
        }
    }

    /// @brief 用表单中的 provider 基础字段刷新 draft。
    /// @param form Provider 表单状态。
    /// @return 无。
    pub fn apply_form_base_fields(&mut self, form: &crate::cli::tui::form::ProviderAddFormState) {
        let next_name = form.name.value.trim().to_string();
        let next_base_url = form.opencode_base_url.value.trim().to_string();
        let next_api_key = form.opencode_api_key.value.trim().to_string();
        let next_npm = if form.opencode_npm_package.value.trim().is_empty() {
            "@ai-sdk/openai-compatible".to_string()
        } else {
            form.opencode_npm_package.value.trim().to_string()
        };
        let next_provider_id = form.id.value.trim().to_string();
        if self.name != next_name
            || self.base_url != next_base_url
            || self.api_key != next_api_key
            || self.npm != next_npm
            || self.provider_id != next_provider_id
        {
            self.dirty = true;
        }
        self.name = next_name;
        self.base_url = next_base_url;
        self.api_key = next_api_key;
        self.npm = next_npm;
        self.provider_id = next_provider_id;
    }

    /// @brief 序列化为 OpenCode settingsConfig JSON。
    /// @return settingsConfig JSON。
    pub fn to_settings_config(&self) -> serde_json::Value {
        let mut settings_obj = self
            .last_loaded_from_db
            .as_object()
            .cloned()
            .unwrap_or_default();
        settings_obj.insert("npm".to_string(), serde_json::json!(self.npm));
        let mut options_obj = settings_obj
            .remove("options")
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        if !self.base_url.is_empty() {
            options_obj.insert("baseURL".to_string(), serde_json::json!(self.base_url));
        } else {
            options_obj.remove("baseURL");
        }
        if !self.api_key.is_empty() {
            options_obj.insert("apiKey".to_string(), serde_json::json!(self.api_key));
        } else {
            options_obj.remove("apiKey");
        }
        if !options_obj.is_empty() {
            settings_obj.insert(
                "options".to_string(),
                serde_json::Value::Object(options_obj),
            );
        }

        let mut models_obj = serde_json::Map::new();
        for draft in self.models_by_id.values() {
            let mut model_obj = draft.extra.as_object().cloned().unwrap_or_default();
            let model_name = if draft.model_name.trim().is_empty() {
                draft.model_id.to_uppercase()
            } else {
                draft.model_name.clone()
            };
            model_obj.insert("name".to_string(), serde_json::json!(model_name));
            if draft.input_limit.is_some() || draft.output_limit.is_some() {
                let mut limit_obj = serde_json::Map::new();
                if let Some(value) = draft.input_limit {
                    limit_obj.insert("context".to_string(), serde_json::json!(value));
                }
                if let Some(value) = draft.output_limit {
                    limit_obj.insert("output".to_string(), serde_json::json!(value));
                }
                model_obj.insert("limit".to_string(), serde_json::Value::Object(limit_obj));
            } else {
                model_obj.remove("limit");
            }
            models_obj.insert(draft.model_id.clone(), serde_json::Value::Object(model_obj));
        }
        settings_obj.insert("models".to_string(), serde_json::Value::Object(models_obj));
        if !self.current_model.trim().is_empty() {
            settings_obj.insert(
                "currentModel".to_string(),
                serde_json::json!(self.current_model),
            );
        } else {
            settings_obj.remove("currentModel");
        }
        serde_json::Value::Object(settings_obj)
    }

    /// @brief 校验 draft 是否可保存为 OpenCode provider。
    /// @return 可保存返回 Ok，否则返回错误文案。
    pub fn validate_for_save(&self) -> Result<(), String> {
        for model in self.models_by_id.values() {
            if model.model_id.trim().is_empty() {
                return Err("Model ID is required".to_string());
            }
            for value in [model.input_limit, model.output_limit]
                .into_iter()
                .flatten()
            {
                if value == 0 || value > u32::MAX as u64 {
                    return Err("Must be positive integer".to_string());
                }
            }
        }
        if !self.current_model.trim().is_empty() && !self.has_model(&self.current_model) {
            return Err(format!(
                "Current model `{}` is not configured",
                self.current_model
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Overlay {
    None,
    Help,
    Confirm(ConfirmOverlay),
    ProviderSwitchFirstUseConfirm {
        provider_id: String,
        title: String,
        message: String,
        selected: usize,
    },
    TextInput(TextInputState),
    BackupPicker {
        selected: usize,
    },
    TextView(TextViewState),
    CommonSnippetPicker {
        selected: usize,
    },
    CommonSnippetView {
        app_type: AppType,
        view: TextViewState,
    },
    ClaudeModelPicker {
        selected: usize,
        editing: bool,
    },
    ClaudeApiFormatPicker {
        selected: usize,
    },
    ModelFetchPicker {
        request_id: u64,
        field: ProviderAddField,
        claude_idx: Option<usize>,
        input: String,
        query: String,
        fetching: bool,
        models: Vec<String>,
        error: Option<String>,
        selected_idx: usize,
    },
    OpenClawToolsProfilePicker {
        selected: Option<usize>,
    },
    OpenClawAgentsFallbackPicker {
        insert_at: usize,
        selected: usize,
        options: Vec<OpenClawModelOption>,
    },
    McpAppsPicker {
        id: String,
        name: String,
        selected: usize,
        apps: crate::app_config::McpApps,
    },
    VisibleAppsPicker {
        selected: usize,
        apps: crate::settings::VisibleApps,
    },
    SkillsAppsPicker {
        directory: String,
        name: String,
        selected: usize,
        apps: crate::app_config::SkillApps,
    },
    SkillsImportPicker {
        skills: Vec<crate::services::skill::UnmanagedSkill>,
        selected_idx: usize,
        selected: HashSet<String>,
    },
    SkillsSyncMethodPicker {
        selected: usize,
    },
    McpEnvPicker {
        selected: usize,
    },
    McpEnvEntryEditor(McpEnvEntryEditorState),
    Loading {
        kind: LoadingKind,
        title: String,
        message: String,
    },
    SpeedtestRunning {
        url: String,
    },
    SpeedtestResult {
        url: String,
        lines: Vec<String>,
        scroll: usize,
    },
    StreamCheckRunning {
        provider_id: String,
        provider_name: String,
    },
    StreamCheckResult {
        provider_name: String,
        lines: Vec<String>,
        scroll: usize,
    },
    UpdateAvailable {
        current: String,
        latest: String,
        selected: usize,
    },
    UpdateDownloading {
        downloaded: u64,
        total: Option<u64>,
    },
    UpdateResult {
        success: bool,
        message: String,
    },
}

impl Overlay {
    pub fn is_active(&self) -> bool {
        !matches!(self, Overlay::None)
    }
}

#[cfg(test)]
mod opencode_draft_tests {
    use super::*;
    use crate::provider::OpenCodeModelDraft;
    use indexmap::IndexMap;
    use serde_json::json;

    fn draft() -> OpenCodeProviderDraft {
        let mut models_by_id = IndexMap::new();
        models_by_id.insert(
            "gpt-4.1".to_string(),
            OpenCodeModelDraft {
                model_id: "gpt-4.1".to_string(),
                model_name: "GPT 4.1".to_string(),
                input_limit: Some(128000),
                output_limit: Some(8192),
                original_model_id: Some("gpt-4.1".to_string()),
                extra: json!({}),
            },
        );
        OpenCodeProviderDraft {
            provider_id: "demo".to_string(),
            name: "Demo".to_string(),
            base_url: "https://api.example.com/v1".to_string(),
            api_key: "sk-demo".to_string(),
            npm: "@ai-sdk/openai-compatible".to_string(),
            models_by_id,
            current_model: "gpt-4.1".to_string(),
            dirty: false,
            last_loaded_from_db: json!({ "provider": "demo" }),
        }
    }

    #[test]
    fn opencode_provider_draft_tracks_model_crud_and_current_model_fallback() {
        let mut draft = draft();

        assert_eq!(draft.model_count(), 1);
        assert!(!draft.is_dirty());

        draft
            .add_model(OpenCodeModelDraft::new("gpt-4o".to_string()))
            .expect("unique model should be added");
        assert!(draft.has_model("gpt-4o"));
        assert!(draft.is_dirty());

        let err = draft
            .add_model(OpenCodeModelDraft::new("gpt-4o".to_string()))
            .expect_err("duplicate model id should be rejected");
        assert!(err.contains("Duplicated model id"));

        draft.set_dirty(false);
        let removed = draft
            .remove_model("gpt-4.1")
            .expect("model should be removed");
        assert_eq!(removed.model_id, "gpt-4.1");
        assert_eq!(draft.current_model, "gpt-4o");
        assert!(draft.is_dirty());
    }

    #[test]
    fn opencode_provider_draft_removing_current_model_uses_ascii_minimum_fallback() {
        let mut draft = draft();
        draft
            .add_model(OpenCodeModelDraft::new("z-model".to_string()))
            .expect("add later model");
        draft
            .add_model(OpenCodeModelDraft::new("a-model".to_string()))
            .expect("add ascii-first model");
        draft.set_dirty(false);

        draft
            .remove_model("gpt-4.1")
            .expect("current model should be removed");

        assert_eq!(draft.current_model, "a-model");
        assert!(draft.is_dirty());
    }

    #[test]
    fn opencode_provider_draft_serialization_removes_stale_current_model_when_empty() {
        let mut draft = draft();
        draft.last_loaded_from_db = json!({
            "currentModel": "gpt-4.1",
            "models": { "gpt-4.1": { "name": "GPT 4.1" } }
        });
        draft.remove_model("gpt-4.1").expect("remove only model");

        let settings = draft.to_settings_config();

        assert!(settings.get("currentModel").is_none());
        assert_eq!(settings["models"], json!({}));
    }

    #[test]
    fn opencode_provider_draft_rename_is_atomic_and_repairs_current_model() {
        let mut draft = draft();

        draft
            .rename_model("gpt-4.1", "gpt-5.4".to_string())
            .expect("rename should succeed");

        assert!(!draft.has_model("gpt-4.1"));
        assert!(draft.has_model("gpt-5.4"));
        assert_eq!(draft.current_model, "gpt-5.4");
        assert_eq!(
            draft
                .models_by_id
                .get("gpt-5.4")
                .and_then(|model| model.original_model_id.as_deref()),
            Some("gpt-4.1")
        );
    }

    #[test]
    fn opencode_provider_draft_serialization_preserves_unmanaged_settings() {
        let provider = crate::provider::Provider::with_id(
            "demo".to_string(),
            "Demo".to_string(),
            json!({
                "npm": "@ai-sdk/openai-compatible",
                "name": "upstream-name",
                "options": {
                    "baseURL": "https://api.example.com/v1",
                    "apiKey": "sk-old",
                    "headers": { "X-Test": "1" },
                    "timeout": 30
                },
                "models": {
                    "gpt-4.1": {
                        "name": "GPT 4.1",
                        "options": { "reasoningEffort": "medium" }
                    }
                },
                "currentModel": "gpt-4.1"
            }),
            None,
        );
        let mut draft = OpenCodeProviderDraft::from_provider(&provider);

        draft.api_key = "sk-new".to_string();
        let settings = draft.to_settings_config();

        assert_eq!(settings["name"], "upstream-name");
        assert_eq!(settings["options"]["apiKey"], "sk-new");
        assert_eq!(settings["options"]["headers"]["X-Test"], "1");
        assert_eq!(settings["options"]["timeout"], 30);
        assert_eq!(
            settings["models"]["gpt-4.1"]["options"]["reasoningEffort"],
            "medium"
        );
    }
}
