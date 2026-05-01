use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::super::*;

pub(crate) fn render_opencode_model_list(
    frame: &mut Frame<'_>,
    app: &App,
    _data: &UiData,
    area: Rect,
    theme: &theme::Theme,
    _provider_id: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(pane_border_style(app, Focus::Content, theme))
        .title(texts::tui_opencode_model_list_title());
    frame.render_widget(outer.clone(), area);
    let inner = outer.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    render_key_bar(
        frame,
        chunks[0],
        theme,
        &[
            ("Ctrl+S", texts::tui_key_save()),
            ("Esc", texts::tui_key_close()),
            ("↑↓", texts::tui_key_select()),
            ("Enter", texts::tui_key_open()),
            ("n", texts::tui_key_add()),
            ("Del", texts::tui_key_delete()),
        ],
    );

    let Some(FormState::ProviderAdd(form)) = &app.form else {
        frame.render_widget(
            Paragraph::new(texts::tui_opencode_model_no_data())
                .style(Style::default().fg(theme.dim)),
            chunks[1],
        );
        return;
    };

    if form.opencode_models.is_empty() {
        frame.render_widget(
            Paragraph::new(texts::tui_opencode_model_empty_list())
                .style(Style::default().fg(theme.dim)),
            chunks[1],
        );
        return;
    }

    let items = form
        .opencode_models
        .iter()
        .map(|draft| {
            let display_name = if draft.model_name.trim().is_empty() {
                draft.model_id.to_uppercase()
            } else {
                draft.model_name.clone()
            };
            let detail = if draft.model_name.trim().is_empty() {
                draft.model_id.clone()
            } else {
                format!("{}  ({})", display_name, draft.model_id)
            };
            ListItem::new(Line::raw(detail))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(
        app.provider_idx
            .min(form.opencode_models.len().saturating_sub(1)),
    ));

    let list = List::new(items)
        .highlight_style(selection_style(theme))
        .highlight_symbol(highlight_symbol(theme));
    frame.render_stateful_widget(list, chunks[1], &mut state);
}
