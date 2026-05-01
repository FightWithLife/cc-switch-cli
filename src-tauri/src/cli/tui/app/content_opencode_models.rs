use super::*;

const OPENCODE_MODEL_DETAIL_FIELDS: [ProviderAddField; 4] = [
    ProviderAddField::OpenCodeModelName,
    ProviderAddField::OpenCodeModelId,
    ProviderAddField::OpenCodeModelContextLimit,
    ProviderAddField::OpenCodeModelOutputLimit,
];

impl App {
    pub(crate) fn on_opencode_model_list_key(
        &mut self,
        key: KeyEvent,
        data: &UiData,
        provider_id: &str,
    ) -> Action {
        if is_save_shortcut(key) {
            return self.handle_form_save_shortcut(data);
        }

        let model_count = self
            .form
            .as_ref()
            .and_then(|form| match form {
                FormState::ProviderAdd(form) => Some(form.opencode_models.len()),
                _ => None,
            })
            .unwrap_or(0);

        match key.code {
            KeyCode::Up => {
                self.provider_idx = self.provider_idx.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                if model_count > 0 {
                    self.provider_idx = (self.provider_idx + 1).min(model_count - 1);
                }
                Action::None
            }
            KeyCode::Enter => {
                let next_route = {
                    let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
                        return Action::None;
                    };
                    if form.opencode_models.is_empty() {
                        return Action::None;
                    }
                    if let Err(err) = form.sync_current_opencode_model() {
                        self.push_toast(err, ToastKind::Error);
                        return Action::None;
                    }
                    let model_idx = self.provider_idx.min(form.opencode_models.len() - 1);
                    form.opencode_model_idx = model_idx;
                    form.load_current_opencode_model_fields();
                    form.field_idx = 0;
                    form.editing = false;
                    Route::OpenCodeModelConfigDetail {
                        provider_id: provider_id.to_string(),
                        model_idx,
                    }
                };
                self.push_route_and_switch(next_route)
            }
            KeyCode::Char('n') => {
                let next_route = {
                    let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
                        return Action::None;
                    };
                    match form.add_opencode_model() {
                        Ok(()) => {
                            let model_idx = form.opencode_model_idx;
                            form.field_idx = 0;
                            form.editing = true;
                            self.provider_idx = model_idx;
                            Some(Route::OpenCodeModelConfigDetail {
                                provider_id: provider_id.to_string(),
                                model_idx,
                            })
                        }
                        Err(err) => {
                            self.push_toast(err, ToastKind::Warning);
                            None
                        }
                    }
                };

                if let Some(route) = next_route {
                    self.push_route_and_switch(route)
                } else {
                    Action::None
                }
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                let model_id = self.form.as_ref().and_then(|form| match form {
                    FormState::ProviderAdd(form) => form
                        .opencode_models
                        .get(
                            self.provider_idx
                                .min(form.opencode_models.len().saturating_sub(1)),
                        )
                        .map(|draft| draft.model_id.clone()),
                    _ => None,
                });

                if let Some(model_id) = model_id {
                    self.overlay = Overlay::Confirm(ConfirmOverlay {
                        title: texts::tui_opencode_model_delete_title().to_string(),
                        message: texts::tui_opencode_model_delete_message(&model_id),
                        action: ConfirmAction::OpenCodeModelDelete {
                            provider_id: provider_id.to_string(),
                            model_id,
                        },
                    });
                }
                Action::None
            }
            KeyCode::Esc | KeyCode::Char('q') => self.pop_route_and_switch(),
            _ => Action::None,
        }
    }

    pub(crate) fn on_opencode_model_detail_key(
        &mut self,
        key: KeyEvent,
        data: &UiData,
        provider_id: &str,
        model_idx: usize,
    ) -> Action {
        if is_save_shortcut(key) {
            return self.handle_form_save_shortcut(data);
        }

        let maybe_fetch = {
            let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
                return Action::None;
            };

            if form.opencode_models.is_empty() {
                form.editing = false;
                return self.pop_route_and_switch();
            }

            if form.opencode_model_idx != model_idx.min(form.opencode_models.len() - 1) {
                if let Err(err) = form.sync_current_opencode_model() {
                    self.push_toast(err, ToastKind::Error);
                    return Action::None;
                }
                form.opencode_model_idx = model_idx.min(form.opencode_models.len() - 1);
                form.load_current_opencode_model_fields();
            }

            form.field_idx = form
                .field_idx
                .min(OPENCODE_MODEL_DETAIL_FIELDS.len().saturating_sub(1));

            if form.editing {
                self.handle_opencode_model_detail_editing_key(key)
            } else {
                self.handle_opencode_model_detail_navigation_key(key, provider_id)
            }
        };

        maybe_fetch.unwrap_or(Action::None)
    }

    fn handle_opencode_model_detail_editing_key(&mut self, key: KeyEvent) -> Option<Action> {
        let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
            return Some(Action::None);
        };
        let field = OPENCODE_MODEL_DETAIL_FIELDS[form.field_idx];

        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                if let Err(err) = form.sync_current_opencode_model() {
                    self.push_toast(err, ToastKind::Error);
                } else {
                    form.editing = false;
                }
                Some(Action::None)
            }
            KeyCode::Left => {
                if let Some(input) = form.input_mut(field) {
                    input.move_left();
                }
                Some(Action::None)
            }
            KeyCode::Right => {
                if let Some(input) = form.input_mut(field) {
                    input.move_right();
                }
                Some(Action::None)
            }
            KeyCode::Home => {
                if let Some(input) = form.input_mut(field) {
                    input.move_home();
                }
                Some(Action::None)
            }
            KeyCode::End => {
                if let Some(input) = form.input_mut(field) {
                    input.move_end();
                }
                Some(Action::None)
            }
            KeyCode::Backspace => {
                let changed = form
                    .input_mut(field)
                    .map(|input| input.backspace())
                    .unwrap_or(false);
                self.finish_opencode_model_detail_input_change(field, changed);
                Some(Action::None)
            }
            KeyCode::Delete => {
                let changed = form
                    .input_mut(field)
                    .map(|input| input.delete())
                    .unwrap_or(false);
                self.finish_opencode_model_detail_input_change(field, changed);
                Some(Action::None)
            }
            KeyCode::Char(c) => {
                if c.is_control() || key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Some(Action::None);
                }
                let changed = form
                    .input_mut(field)
                    .map(|input| input.insert_char(c))
                    .unwrap_or(false);
                self.finish_opencode_model_detail_input_change(field, changed);
                Some(Action::None)
            }
            _ => Some(Action::None),
        }
    }

    fn handle_opencode_model_detail_navigation_key(
        &mut self,
        key: KeyEvent,
        provider_id: &str,
    ) -> Option<Action> {
        let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
            return Some(Action::None);
        };
        let selected = OPENCODE_MODEL_DETAIL_FIELDS[form.field_idx];

        match key.code {
            KeyCode::Up => {
                form.field_idx = form.field_idx.saturating_sub(1);
                Some(Action::None)
            }
            KeyCode::Down => {
                form.field_idx = (form.field_idx + 1).min(OPENCODE_MODEL_DETAIL_FIELDS.len() - 1);
                Some(Action::None)
            }
            KeyCode::Enter if selected == ProviderAddField::OpenCodeModelId => {
                let base_url = form.opencode_base_url.value.trim().to_string();
                if base_url.is_empty() {
                    self.push_toast(
                        texts::tui_opencode_model_fetch_base_url_required(),
                        ToastKind::Warning,
                    );
                    return Some(Action::None);
                }
                let api_key = (!form.opencode_api_key.value.trim().is_empty())
                    .then(|| form.opencode_api_key.value.clone());
                Some(Action::ProviderModelFetch {
                    base_url,
                    api_key,
                    field: ProviderAddField::OpenCodeModelId,
                    claude_idx: None,
                })
            }
            KeyCode::Enter => {
                form.editing = true;
                Some(Action::None)
            }
            KeyCode::Char('n') => {
                let next_route = match form.add_opencode_model() {
                    Ok(()) => {
                        let model_idx = form.opencode_model_idx;
                        form.field_idx = 0;
                        form.editing = true;
                        Some(Route::OpenCodeModelConfigDetail {
                            provider_id: provider_id.to_string(),
                            model_idx,
                        })
                    }
                    Err(err) => {
                        self.push_toast(err, ToastKind::Warning);
                        None
                    }
                };
                Some(if let Some(route) = next_route {
                    self.push_route_and_switch(route)
                } else {
                    Action::None
                })
            }
            KeyCode::Delete | KeyCode::Char('d') => {
                let model_id = form
                    .opencode_models
                    .get(form.opencode_model_idx)
                    .map(|draft| draft.model_id.clone())
                    .unwrap_or_default();
                if !model_id.is_empty() {
                    self.overlay = Overlay::Confirm(ConfirmOverlay {
                        title: texts::tui_opencode_model_delete_title().to_string(),
                        message: texts::tui_opencode_model_delete_message(&model_id),
                        action: ConfirmAction::OpenCodeModelDelete {
                            provider_id: provider_id.to_string(),
                            model_id,
                        },
                    });
                }
                Some(Action::None)
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.provider_idx = form.opencode_model_idx;
                form.editing = false;
                Some(self.pop_route_and_switch())
            }
            _ => Some(Action::None),
        }
    }

    fn finish_opencode_model_detail_input_change(
        &mut self,
        field: ProviderAddField,
        changed: bool,
    ) {
        if !changed {
            return;
        }

        let Some(FormState::ProviderAdd(form)) = self.form.as_mut() else {
            return;
        };
        if let Err(err) = form.sync_current_opencode_model() {
            self.push_toast(err, ToastKind::Error);
            return;
        }

        if field == ProviderAddField::OpenCodeModelId && form.opencode_model_name.is_blank() {
            let _ = form.sync_current_opencode_model();
        }
    }
}
