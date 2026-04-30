use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::*;

/// 渲染 OpenCode Model Config List 页面
pub(super) fn render_opencode_model_list(
    frame: &mut Frame<'_>,
    app: &App,
    _data: &UiData,
    area: Rect,
    theme: &super::super::theme::Theme,
    _provider_id: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let footer_area = chunks[1];

    let block = Block::default()
        .title(if i18n::is_chinese() {
            " 模型配置 "
        } else {
            " Model Config "
        })
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim));

    // 从 form 中获取 opencode_models（如果存在）
    let Some(FormState::ProviderAdd(form)) = &app.form else {
        let empty_msg = if i18n::is_chinese() {
            "无可用数据。请先打开供应商编辑页面。"
        } else {
            "No data available. Open the provider edit form first."
        };
        let paragraph = Paragraph::new(empty_msg)
            .block(block)
            .style(Style::default().fg(theme.dim));
        frame.render_widget(paragraph, content_area);
        return;
    };

    let inner = block.inner(content_area);
    frame.render_widget(block, content_area);

    if form.opencode_models.is_empty() {
        let empty_msg = if i18n::is_chinese() {
            "未配置模型。按 [n] 添加新模型。"
        } else {
            "No models configured. Press [n] to add one."
        };
        let paragraph = Paragraph::new(empty_msg).style(Style::default().fg(theme.dim));
        frame.render_widget(paragraph, inner);
    } else {
        // 渲染模型列表
        let items: Vec<ListItem> = form
            .opencode_models
            .iter()
            .map(|draft| {
                let display_name = if draft.model_name.is_empty() {
                    draft.model_id.to_uppercase()
                } else {
                    draft.model_name.clone()
                };
                let model_id_hint = if draft.model_name.is_empty() {
                    String::new()
                } else {
                    format!("  ({})", draft.model_id)
                };
                ListItem::new(Line::from(vec![
                    Span::raw(display_name),
                    Span::styled(model_id_hint, Style::default().fg(theme.dim)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(
            app.provider_idx
                .min(form.opencode_models.len().saturating_sub(1)),
        ));

        frame.render_stateful_widget(list, inner, &mut state);
    }

    // 底部快捷键提示
    let footer_text = if i18n::is_chinese() {
        "[Enter] 编辑模型  [n] 新建模型  [Del] 删除  [Esc] 返回"
    } else {
        "[Enter] edit model  [n] new model  [Del] delete  [Esc] back"
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.dim));
    frame.render_widget(footer, footer_area);
}

/// 渲染 OpenCode Model Config Detail 页面
pub(super) fn render_opencode_model_detail(
    frame: &mut Frame<'_>,
    app: &App,
    _data: &UiData,
    area: Rect,
    theme: &super::super::theme::Theme,
    _provider_id: &str,
    model_idx: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let footer_area = chunks[1];

    let is_new = model_idx
        >= app
            .form
            .as_ref()
            .and_then(|f| match f {
                FormState::ProviderAdd(form) => Some(form.opencode_model_count()),
                _ => None,
            })
            .unwrap_or(0);

    let title = if is_new {
        if i18n::is_chinese() {
            " 新建模型 ".to_string()
        } else {
            " New Model ".to_string()
        }
    } else if i18n::is_chinese() {
        format!(" 编辑模型 #{} ", model_idx + 1)
    } else {
        format!(" Edit Model #{} ", model_idx + 1)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim));

    // 获取当前 model 数据
    let model_data = app.form.as_ref().and_then(|f| match f {
        FormState::ProviderAdd(form) => form.opencode_models.get(model_idx).cloned(),
        _ => None,
    });

    let inner = block.inner(content_area);
    frame.render_widget(block, content_area);

    if let Some(draft) = model_data {
        // 渲染 4 个字段
        let fields = vec![
            (
                if i18n::is_chinese() {
                    "模型名称"
                } else {
                    "Model Name"
                },
                if draft.model_name.is_empty() {
                    "-".to_string()
                } else {
                    draft.model_name.clone()
                },
            ),
            (
                "Model ID",
                if draft.model_id.is_empty() {
                    "-".to_string()
                } else {
                    draft.model_id.clone()
                },
            ),
            (
                if i18n::is_chinese() {
                    "输入限制"
                } else {
                    "Input Limit"
                },
                draft
                    .input_limit
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                if i18n::is_chinese() {
                    "输出限制"
                } else {
                    "Output Limit"
                },
                draft
                    .output_limit
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ];

        let focused_idx = app.provider_idx.min(3); // 4 个字段

        let lines: Vec<Line> = fields
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let is_focused = i == focused_idx;
                let cursor = if is_focused { "> " } else { "  " };
                let value_style = if is_focused {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(vec![
                    Span::raw(cursor),
                    Span::styled(format!("{:>14}: ", label), Style::default().fg(theme.dim)),
                    Span::styled(value.as_str(), value_style),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    } else if is_new {
        let hint = if i18n::is_chinese() {
            "新模型将在保存时创建。\n按 [Esc] 返回列表。"
        } else {
            "New model will be created on save.\nPress [Esc] to go back."
        };
        let paragraph = Paragraph::new(hint).style(Style::default().fg(theme.dim));
        frame.render_widget(paragraph, inner);
    } else {
        let error = if i18n::is_chinese() {
            "模型数据不存在。"
        } else {
            "Model data not found."
        };
        let paragraph = Paragraph::new(error).style(Style::default().fg(theme.err));
        frame.render_widget(paragraph, inner);
    }

    // 底部快捷键提示
    let footer_text = if i18n::is_chinese() {
        "[↑/↓] 切换字段  [Enter] 编辑  [Ctrl+S] 保存  [Esc] 返回"
    } else {
        "[↑/↓] switch field  [Enter] edit  [Ctrl+S] save  [Esc] back"
    };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.dim));
    frame.render_widget(footer, footer_area);
}
