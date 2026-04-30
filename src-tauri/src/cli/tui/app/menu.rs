use super::*;

const PROXY_ACTIVITY_WINDOW: usize = 48;
const PROXY_ACTIVITY_POLL_INTERVAL_TICKS: u64 = 5;

impl App {
    pub(crate) fn clear_openclaw_daily_memory_search_state(&mut self) {
        self.filter.active = false;
        self.filter.buffer.clear();
        self.openclaw_daily_memory_search_query.clear();
        self.openclaw_daily_memory_search_results.clear();
        self.daily_memory_idx = 0;
    }

    pub fn new(app_override: Option<AppType>) -> Self {
        let app_type = app_override.unwrap_or(AppType::Claude);
        Self {
            app_type,
            route: Route::Main,
            route_stack: Vec::new(),
            focus: Focus::Nav,
            nav_idx: 0,
            filter: FilterState::new(),
            editor: None,
            form: None,
            pending_overlay: None,
            overlay: Overlay::None,
            toast: None,
            should_quit: false,
            last_size: Size::new(0, 0),
            tick: 0,
            proxy_input_activity_samples: Vec::new(),
            proxy_output_activity_samples: Vec::new(),
            proxy_activity_last_input_tokens: None,
            proxy_activity_last_output_tokens: None,
            proxy_visual_state: None,
            proxy_visual_transition: None,
            local_env_results: Vec::new(),
            local_env_loading: true,
            provider_idx: 0,
            mcp_idx: 0,
            prompt_idx: 0,
            skills_idx: 0,
            skills_discover_idx: 0,
            skills_repo_idx: 0,
            skills_unmanaged_idx: 0,
            skills_discover_results: Vec::new(),
            skills_discover_query: String::new(),
            skills_unmanaged_results: Vec::new(),
            skills_unmanaged_selected: HashSet::new(),
            config_idx: 0,
            workspace_idx: 0,
            daily_memory_idx: 0,
            openclaw_tools_form: None,
            openclaw_agents_form: None,
            openclaw_daily_memory_search_query: String::new(),
            openclaw_daily_memory_search_results: Vec::new(),
            config_webdav_idx: 0,
            webdav_quick_setup_username: None,
            language_idx: 0,
            settings_idx: 0,
            settings_proxy_idx: 0,
        }
    }

    pub fn nav_item(&self) -> NavItem {
        self.nav_items()
            .get(self.nav_idx)
            .copied()
            .unwrap_or(NavItem::Main)
    }

    pub(crate) fn nav_items(&self) -> &'static [NavItem] {
        NavItem::all_for_app(&self.app_type)
    }

    pub(crate) fn nav_item_for_route(app_type: &AppType, route: &Route) -> NavItem {
        match route {
            Route::Main => NavItem::Main,
            Route::Providers
            | Route::ProviderDetail { .. }
            | Route::OpenCodeModelConfigList { .. }
            | Route::OpenCodeModelConfigDetail { .. } => NavItem::Providers,
            Route::Mcp => NavItem::Mcp,
            Route::Prompts => NavItem::Prompts,
            Route::Config => NavItem::Config,
            Route::ConfigOpenClawWorkspace | Route::ConfigOpenClawDailyMemory => {
                if matches!(app_type, AppType::OpenClaw) {
                    NavItem::OpenClawWorkspace
                } else {
                    NavItem::Config
                }
            }
            Route::ConfigOpenClawEnv => {
                if matches!(app_type, AppType::OpenClaw) {
                    NavItem::OpenClawEnv
                } else {
                    NavItem::Config
                }
            }
            Route::ConfigOpenClawTools => {
                if matches!(app_type, AppType::OpenClaw) {
                    NavItem::OpenClawTools
                } else {
                    NavItem::Config
                }
            }
            Route::ConfigOpenClawAgents => {
                if matches!(app_type, AppType::OpenClaw) {
                    NavItem::OpenClawAgents
                } else {
                    NavItem::Config
                }
            }
            Route::ConfigWebDav => NavItem::Config,
            Route::Skills
            | Route::SkillsDiscover
            | Route::SkillsRepos
            | Route::SkillDetail { .. } => NavItem::Skills,
            Route::Settings | Route::SettingsProxy => NavItem::Settings,
        }
    }

    pub(crate) fn set_route_no_history(&mut self, route: Route) -> Action {
        if route == self.route {
            return Action::None;
        }

        let was_daily_memory = matches!(self.route, Route::ConfigOpenClawDailyMemory);
        let is_daily_memory = matches!(route, Route::ConfigOpenClawDailyMemory);
        if was_daily_memory != is_daily_memory {
            self.clear_openclaw_daily_memory_search_state();
        }
        if !matches!(route, Route::ConfigOpenClawTools) {
            self.openclaw_tools_form = None;
        }
        if !matches!(route, Route::ConfigOpenClawAgents) {
            self.openclaw_agents_form = None;
        }

        self.route = route.clone();
        self.focus = route_default_focus(&route);

        let nav_item = Self::nav_item_for_route(&self.app_type, &route);
        if let Some(idx) = self.nav_items().iter().position(|item| *item == nav_item) {
            self.nav_idx = idx;
        }

        if matches!(route, Route::Main) {
            self.route_stack.clear();
            self.focus = Focus::Nav;
        }

        Action::SwitchRoute(route)
    }

    pub(crate) fn push_route_and_switch(&mut self, route: Route) -> Action {
        if route == self.route {
            return Action::None;
        }
        self.route_stack.push(self.route.clone());
        self.set_route_no_history(route)
    }

    pub(crate) fn pop_route_and_switch(&mut self) -> Action {
        if let Some(prev) = self.route_stack.pop() {
            self.set_route_no_history(prev)
        } else {
            self.set_route_no_history(Route::Main)
        }
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if let Some(toast) = &mut self.toast {
            if toast.remaining_ticks > 0 {
                toast.remaining_ticks -= 1;
            }
            if toast.remaining_ticks == 0 {
                self.toast = None;
            }
        }

        if let Some(transition) = self.proxy_visual_transition {
            if self.tick.saturating_sub(transition.started_tick) >= PROXY_HERO_TRANSITION_TICKS {
                self.proxy_visual_transition = None;
            }
        }
    }

    pub(crate) fn observe_proxy_visual_state(&mut self, data: &UiData) {
        let current_on = data.proxy.running;

        match self.proxy_visual_state.replace(current_on) {
            None => {}
            Some(previous_on) if previous_on != current_on => {
                self.proxy_visual_transition = Some(ProxyVisualTransition {
                    from_on: previous_on,
                    to_on: current_on,
                    started_tick: self.tick,
                });
            }
            Some(_) => {}
        }
    }

    pub(crate) fn should_poll_proxy_activity(&self) -> bool {
        matches!(self.route, Route::Main) && self.tick % PROXY_ACTIVITY_POLL_INTERVAL_TICKS == 0
    }

    pub(crate) fn reset_proxy_activity(&mut self, input_tokens: u64, output_tokens: u64) {
        self.proxy_input_activity_samples.clear();
        self.proxy_output_activity_samples.clear();
        self.proxy_activity_last_input_tokens = Some(input_tokens);
        self.proxy_activity_last_output_tokens = Some(output_tokens);
    }

    pub(crate) fn observe_proxy_token_activity(&mut self, input_tokens: u64, output_tokens: u64) {
        let Some(previous_input) = self.proxy_activity_last_input_tokens.replace(input_tokens)
        else {
            return;
        };
        let Some(previous_output) = self
            .proxy_activity_last_output_tokens
            .replace(output_tokens)
        else {
            return;
        };

        let (input_delta, output_delta) =
            if input_tokens < previous_input || output_tokens < previous_output {
                self.proxy_input_activity_samples.clear();
                self.proxy_output_activity_samples.clear();
                (0, 0)
            } else {
                (
                    input_tokens.saturating_sub(previous_input),
                    output_tokens.saturating_sub(previous_output),
                )
            };

        self.proxy_input_activity_samples.push(input_delta);
        self.proxy_output_activity_samples.push(output_delta);

        if self.proxy_input_activity_samples.len() > PROXY_ACTIVITY_WINDOW {
            let overflow = self.proxy_input_activity_samples.len() - PROXY_ACTIVITY_WINDOW;
            self.proxy_input_activity_samples.drain(0..overflow);
        }
        if self.proxy_output_activity_samples.len() > PROXY_ACTIVITY_WINDOW {
            let overflow = self.proxy_output_activity_samples.len() - PROXY_ACTIVITY_WINDOW;
            self.proxy_output_activity_samples.drain(0..overflow);
        }
    }

    pub fn push_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toast = Some(Toast::new(message, kind));
    }

    pub fn open_help(&mut self) {
        self.overlay = Overlay::Help;
    }

    pub fn close_overlay(&mut self) {
        self.overlay = self.pending_overlay.take().unwrap_or(Overlay::None);
    }

    fn structured_form_is_editing_text_field(&self) -> bool {
        match self.route {
            Route::ConfigOpenClawTools => false,
            Route::ConfigOpenClawAgents => false,
            _ => false,
        }
    }

    fn should_route_printable_content_input_before_globals(&self, key: &KeyEvent) -> bool {
        matches!(self.focus, Focus::Content)
            && self.structured_form_is_editing_text_field()
            && matches!(key.code, KeyCode::Char(c) if !c.is_control())
            && !key.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn on_key(&mut self, key: KeyEvent, data: &UiData) -> Action {
        self.clamp_selections(data);
        if !self.overlay.is_active() {
            self.pending_overlay = None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.should_quit = true;
            return Action::Quit;
        }

        if self.overlay.is_active() {
            return self.on_overlay_key(key, data);
        }

        if self.editor.is_some() {
            return self.on_editor_key(key);
        }

        // ModelConfigList/Detail 页面优先处理自身按键（form 数据仍保留但不拦截按键）
        if matches!(
            self.route,
            Route::OpenCodeModelConfigList { .. } | Route::OpenCodeModelConfigDetail { .. }
        ) {
            // Global actions.
            match key.code {
                KeyCode::Char('?') => {
                    self.open_help();
                    return Action::None;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    return self.on_back_key();
                }
                _ => {}
            }
            return self.on_content_key(key, data);
        }

        if self.form.is_some() {
            return self.on_form_key(key, data);
        }

        if self.filter.active {
            return self.on_filter_key(key);
        }

        // Vim-style hjkl navigation
        let key = if matches!(self.focus, Focus::Content)
            && self.structured_form_is_editing_text_field()
        {
            key
        } else {
            match key.code {
                KeyCode::Char('h') => KeyEvent::new(KeyCode::Left, key.modifiers),
                KeyCode::Char('j') => KeyEvent::new(KeyCode::Down, key.modifiers),
                KeyCode::Char('k') => KeyEvent::new(KeyCode::Up, key.modifiers),
                KeyCode::Char('l') => KeyEvent::new(KeyCode::Right, key.modifiers),
                _ => key,
            }
        };

        if self.should_route_printable_content_input_before_globals(&key) {
            return self.on_content_key(key, data);
        }

        // Global actions.
        match key.code {
            KeyCode::Char('?') => {
                self.open_help();
                return Action::None;
            }
            KeyCode::Char('/') => {
                self.filter.active = true;
                return Action::None;
            }
            KeyCode::Char('[') => {
                return cycle_app_type(&self.app_type, -1)
                    .map(Action::SetAppType)
                    .unwrap_or(Action::None);
            }
            KeyCode::Char(']') => {
                return cycle_app_type(&self.app_type, 1)
                    .map(Action::SetAppType)
                    .unwrap_or(Action::None);
            }
            KeyCode::Left => {
                self.focus = Focus::Nav;
                return Action::None;
            }
            KeyCode::Right => {
                if route_has_content_list(&self.route) {
                    self.focus = Focus::Content;
                } else {
                    self.focus = Focus::Nav;
                }
                return Action::None;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                return self.on_back_key();
            }
            _ => {}
        }

        if matches!(self.route, Route::Main)
            && matches!(key.code, KeyCode::Char('p') | KeyCode::Char('P'))
        {
            return self.main_proxy_action(data);
        }

        // Navigation + route-specific actions.
        match self.focus {
            Focus::Nav => self.on_nav_key(key),
            Focus::Content => self.on_content_key(key, data),
        }
    }

    pub(crate) fn on_back_key(&mut self) -> Action {
        match self.route {
            Route::Main => {
                self.overlay = Overlay::Confirm(ConfirmOverlay {
                    title: crate::cli::i18n::texts::tui_confirm_exit_title().to_string(),
                    message: crate::cli::i18n::texts::tui_confirm_exit_message().to_string(),
                    action: ConfirmAction::Quit,
                });
                Action::None
            }
            _ => self.pop_route_and_switch(),
        }
    }

    pub(crate) fn on_filter_key(&mut self, key: KeyEvent) -> Action {
        let is_daily_memory = matches!(self.route, Route::ConfigOpenClawDailyMemory);
        match key.code {
            KeyCode::Esc => {
                self.filter.active = false;
                self.filter.buffer.clear();
                if is_daily_memory {
                    self.openclaw_daily_memory_search_results.clear();
                    self.daily_memory_idx = 0;
                    return Action::OpenClawDailyMemorySearch {
                        query: String::new(),
                    };
                }
            }
            KeyCode::Enter => {
                self.filter.active = false;
                if is_daily_memory {
                    return Action::OpenClawDailyMemorySearch {
                        query: self.filter.buffer.clone(),
                    };
                }
            }
            KeyCode::Backspace => {
                self.filter.buffer.pop();
                if is_daily_memory && self.filter.buffer.is_empty() {
                    return Action::OpenClawDailyMemorySearch {
                        query: String::new(),
                    };
                }
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    self.filter.buffer.push(c);
                }
            }
            _ => {}
        }
        Action::None
    }

    pub(crate) fn on_nav_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up => {
                self.nav_idx = self.nav_idx.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                self.nav_idx = (self.nav_idx + 1).min(self.nav_items().len() - 1);
                Action::None
            }
            KeyCode::Enter => {
                if let Some(route) = self.nav_item().to_route() {
                    self.push_route_and_switch(route)
                } else {
                    self.overlay = Overlay::Confirm(ConfirmOverlay {
                        title: crate::cli::i18n::texts::tui_confirm_exit_title().to_string(),
                        message: crate::cli::i18n::texts::tui_confirm_exit_message().to_string(),
                        action: ConfirmAction::Quit,
                    });
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    pub(crate) fn on_content_key(&mut self, key: KeyEvent, data: &UiData) -> Action {
        match self.route.clone() {
            Route::Providers => self.on_providers_key(key, data),
            Route::ProviderDetail { id } => self.on_provider_detail_key(key, data, &id),
            Route::Mcp => self.on_mcp_key(key, data),
            Route::Prompts => self.on_prompts_key(key, data),
            Route::Config => self.on_config_key(key, data),
            Route::ConfigOpenClawWorkspace => self.on_config_openclaw_workspace_key(key, data),
            Route::ConfigOpenClawDailyMemory => self.on_config_openclaw_daily_memory_key(key, data),
            Route::ConfigOpenClawEnv => self.on_config_openclaw_env_key(key, data),
            Route::ConfigOpenClawTools => self.on_config_openclaw_tools_key(key, data),
            Route::ConfigOpenClawAgents => self.on_config_openclaw_agents_key(key, data),
            Route::ConfigWebDav => self.on_config_webdav_key(key, data),
            Route::Skills => self.on_skills_installed_key(key, data),
            Route::SkillsDiscover => self.on_skills_discover_key(key),
            Route::SkillsRepos => self.on_skills_repos_key(key, data),
            Route::SkillDetail { directory } => self.on_skill_detail_key(key, data, &directory),
            Route::Settings => self.on_settings_key(key, data),
            Route::SettingsProxy => self.on_settings_proxy_key(key, data),
            Route::Main => match key.code {
                KeyCode::Char('r') => Action::LocalEnvRefresh,
                KeyCode::Char('p') | KeyCode::Char('P') => self.main_proxy_action(data),
                _ => Action::None,
            },
            Route::OpenCodeModelConfigList { provider_id } => {
                self.on_opencode_model_list_key(key, data, &provider_id)
            }
            Route::OpenCodeModelConfigDetail {
                provider_id,
                model_idx,
            } => self.on_opencode_model_detail_key(key, data, &provider_id, model_idx),
        }
    }
    pub(crate) fn clamp_selections(&mut self, data: &UiData) {
        let providers_len = visible_providers(&self.app_type, &self.filter, data).len();
        if providers_len == 0 {
            self.provider_idx = 0;
        } else {
            self.provider_idx = self.provider_idx.min(providers_len - 1);
        }

        let mcp_len = visible_mcp(&self.filter, data).len();
        if mcp_len == 0 {
            self.mcp_idx = 0;
        } else {
            self.mcp_idx = self.mcp_idx.min(mcp_len - 1);
        }

        let prompt_len = visible_prompts(&self.filter, data).len();
        if prompt_len == 0 {
            self.prompt_idx = 0;
        } else {
            self.prompt_idx = self.prompt_idx.min(prompt_len - 1);
        }

        let skills_len = visible_skills_installed(&self.filter, data).len();
        if skills_len == 0 {
            self.skills_idx = 0;
        } else {
            self.skills_idx = self.skills_idx.min(skills_len - 1);
        }

        let discover_len =
            visible_skills_discover(&self.filter, &self.skills_discover_results).len();
        if discover_len == 0 {
            self.skills_discover_idx = 0;
        } else {
            self.skills_discover_idx = self.skills_discover_idx.min(discover_len - 1);
        }

        let repos_len = visible_skills_repos(&self.filter, data).len();
        if repos_len == 0 {
            self.skills_repo_idx = 0;
        } else {
            self.skills_repo_idx = self.skills_repo_idx.min(repos_len - 1);
        }

        let unmanaged_len =
            visible_skills_unmanaged(&self.filter, &self.skills_unmanaged_results).len();
        if unmanaged_len == 0 {
            self.skills_unmanaged_idx = 0;
        } else {
            self.skills_unmanaged_idx = self.skills_unmanaged_idx.min(unmanaged_len - 1);
        }

        let config_len = visible_config_items(&self.filter, &self.app_type).len();
        if config_len == 0 {
            self.config_idx = 0;
        } else {
            self.config_idx = self.config_idx.min(config_len - 1);
        }

        let workspace_len = openclaw_workspace_entry_count();
        if workspace_len == 0 {
            self.workspace_idx = 0;
        } else {
            self.workspace_idx = self.workspace_idx.min(workspace_len - 1);
        }

        let daily_memory_len = visible_openclaw_daily_memory(self, data).len();
        if daily_memory_len == 0 {
            self.daily_memory_idx = 0;
        } else {
            self.daily_memory_idx = self.daily_memory_idx.min(daily_memory_len - 1);
        }

        let config_webdav_len = visible_webdav_config_items(&self.filter).len();
        if config_webdav_len == 0 {
            self.config_webdav_idx = 0;
        } else {
            self.config_webdav_idx = self.config_webdav_idx.min(config_webdav_len - 1);
        }
    }

    /// OpenCode ModelConfigList 页面按键处理
    fn on_opencode_model_list_key(
        &mut self,
        key: KeyEvent,
        _data: &UiData,
        provider_id: &str,
    ) -> Action {
        let model_count = self
            .form
            .as_ref()
            .and_then(|f| match f {
                FormState::ProviderAdd(form) => Some(form.opencode_model_count()),
                _ => None,
            })
            .unwrap_or(0);

        match key.code {
            // Esc 由全局 on_back_key 处理（pop_route_and_switch）
            KeyCode::Up => {
                if self.provider_idx > 0 {
                    self.provider_idx -= 1;
                }
                Action::None
            }
            KeyCode::Down => {
                if model_count > 0 && self.provider_idx < model_count - 1 {
                    self.provider_idx += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                // 编辑选中的 model
                if model_count > 0 {
                    let idx = self.provider_idx.min(model_count - 1);
                    self.route = Route::OpenCodeModelConfigDetail {
                        provider_id: provider_id.to_string(),
                        model_idx: idx,
                    };
                }
                Action::None
            }
            KeyCode::Char('n') => {
                // 新建 model → 先添加空 draft，再进入 detail 编辑
                if let Some(FormState::ProviderAdd(form)) = &mut self.form {
                    let new_id = format!("model-{}", form.opencode_model_count() + 1);
                    let draft = crate::provider::OpenCodeModelDraft::new(new_id);
                    let _ = form.opencode_add_model(draft);
                }
                let new_count = self
                    .form
                    .as_ref()
                    .and_then(|f| match f {
                        FormState::ProviderAdd(form) => Some(form.opencode_model_count()),
                        _ => None,
                    })
                    .unwrap_or(0);
                self.route = Route::OpenCodeModelConfigDetail {
                    provider_id: provider_id.to_string(),
                    model_idx: new_count.saturating_sub(1), // 进入新建的 model
                };
                self.provider_idx = 0; // 聚焦到第一个字段（Model Name）
                Action::None
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                // 删除选中的 model
                if model_count > 0 {
                    let idx = self.provider_idx.min(model_count - 1);
                    if let Some(FormState::ProviderAdd(form)) = &mut self.form {
                        if let Some(removed) = form.opencode_remove_model(idx) {
                            let msg = if crate::cli::i18n::is_chinese() {
                                format!("已删除模型 `{}`", removed.model_id)
                            } else {
                                format!("Deleted model `{}`", removed.model_id)
                            };
                            self.push_toast(msg, ToastKind::Success);
                        }
                    }
                    // 修正索引
                    let new_count = self
                        .form
                        .as_ref()
                        .and_then(|f| match f {
                            FormState::ProviderAdd(form) => Some(form.opencode_model_count()),
                            _ => None,
                        })
                        .unwrap_or(0);
                    if new_count == 0 {
                        self.provider_idx = 0;
                    } else if self.provider_idx >= new_count {
                        self.provider_idx = new_count - 1;
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    /// OpenCode ModelConfigDetail 页面按键处理
    fn on_opencode_model_detail_key(
        &mut self,
        key: KeyEvent,
        _data: &UiData,
        provider_id: &str,
        model_idx: usize,
    ) -> Action {
        // 4 个字段：0=model_name, 1=model_id, 2=input_limit, 3=output_limit
        const DETAIL_FIELD_COUNT: usize = 4;

        match key.code {
            // Esc 由全局 on_back_key 处理（pop_route_and_switch）
            KeyCode::Up => {
                if self.provider_idx > 0 {
                    self.provider_idx -= 1;
                }
                Action::None
            }
            KeyCode::Down => {
                if self.provider_idx < DETAIL_FIELD_COUNT - 1 {
                    self.provider_idx += 1;
                }
                Action::None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // 编辑当前字段
                let field = match self.provider_idx {
                    0 => "model_name",
                    1 => "model_id",
                    2 => "input_limit",
                    3 => "output_limit",
                    _ => return Action::None,
                };

                // 获取当前值
                let current_value =
                    self.form
                        .as_ref()
                        .and_then(|f| match f {
                            FormState::ProviderAdd(form) => form
                                .opencode_models
                                .get(model_idx)
                                .map(|draft| match field {
                                    "model_name" => draft.model_name.clone(),
                                    "model_id" => draft.model_id.clone(),
                                    "input_limit" => {
                                        draft.input_limit.map(|v| v.to_string()).unwrap_or_default()
                                    }
                                    "output_limit" => draft
                                        .output_limit
                                        .map(|v| v.to_string())
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                }),
                            _ => None,
                        })
                        .unwrap_or_default();

                let title = match field {
                    "model_name" => {
                        if crate::cli::i18n::is_chinese() {
                            "编辑模型名称"
                        } else {
                            "Edit Model Name"
                        }
                    }
                    "model_id" => {
                        if crate::cli::i18n::is_chinese() {
                            "编辑模型 ID"
                        } else {
                            "Edit Model ID"
                        }
                    }
                    "input_limit" => {
                        if crate::cli::i18n::is_chinese() {
                            "编辑输入限制"
                        } else {
                            "Edit Input Limit"
                        }
                    }
                    "output_limit" => {
                        if crate::cli::i18n::is_chinese() {
                            "编辑输出限制"
                        } else {
                            "Edit Output Limit"
                        }
                    }
                    _ => "",
                };

                self.open_editor(
                    title.to_string(),
                    crate::cli::tui::app::EditorKind::Plain,
                    current_value,
                    crate::cli::tui::app::EditorSubmit::OpenCodeModelFieldEdit { model_idx, field },
                );
                if let Some(editor) = self.editor.as_mut() {
                    editor.mode = crate::cli::tui::app::EditorMode::Edit;
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}
