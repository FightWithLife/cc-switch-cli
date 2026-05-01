use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::super::*;

const DETAIL_FIELDS: [ProviderAddField; 4] = [
    ProviderAddField::OpenCodeModelName,
    ProviderAddField::OpenCodeModelId,
    ProviderAddField::OpenCodeModelContextLimit,
    ProviderAddField::OpenCodeModelOutputLimit,
];

pub(crate) fn render_opencode_model_detail(
    frame: &mut Frame<'_>,
    app: &App,
    _data: &UiData,
    area: Rect,
    theme: &theme::Theme,
    _provider_id: &str,
    model_idx: usize,
) {
    let Some(FormState::ProviderAdd(form)) = &app.form else {
        frame.render_widget(
            Paragraph::new(texts::tui_opencode_model_no_data())
                .style(Style::default().fg(theme.dim)),
            area,
        );
        return;
    };

    let title = form
        .opencode_models
        .get(model_idx)
        .map(|draft| {
            if draft.model_id.trim().is_empty() {
                texts::tui_opencode_model_new_title().to_string()
            } else {
                texts::tui_opencode_model_edit_title(&draft.model_id)
            }
        })
        .unwrap_or_else(|| texts::tui_opencode_model_new_title().to_string());

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(pane_border_style(app, Focus::Content, theme))
        .title(title);
    frame.render_widget(outer.clone(), area);
    let inner = outer.inner(area);

    let selected = form.field_idx.min(DETAIL_FIELDS.len().saturating_sub(1));
    let editing = form.editing;

    let key_items = if editing {
        vec![
            ("Ctrl+S", texts::tui_key_save()),
            ("Esc/Enter", texts::tui_key_exit_edit()),
            ("←→", texts::tui_key_move()),
        ]
    } else {
        let enter_label = if DETAIL_FIELDS[selected] == ProviderAddField::OpenCodeModelId {
            texts::tui_key_fetch_model()
        } else {
            texts::tui_key_edit_mode()
        };
        let mut items = vec![
            ("Ctrl+S", texts::tui_key_save()),
            ("Esc", texts::tui_key_close()),
            ("↑↓", texts::tui_key_select()),
            ("Enter", enter_label),
        ];
        items.push(("n", texts::tui_key_add()));
        items.push(("Del", texts::tui_key_delete()));
        items
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    render_key_bar(frame, chunks[0], theme, &key_items);

    let label_col_width = field_label_column_width(
        [
            texts::tui_label_opencode_model_id(),
            texts::tui_label_opencode_model_name(),
            texts::tui_label_context_limit(),
            texts::tui_label_output_limit(),
            texts::tui_header_field(),
        ],
        1,
    );

    let header = Row::new(vec![
        Cell::from(cell_pad(texts::tui_header_field())),
        Cell::from(texts::tui_header_value()),
    ])
    .style(Style::default().fg(theme.dim).add_modifier(Modifier::BOLD));

    let rows = vec![
        (
            texts::tui_label_opencode_model_name().to_string(),
            form.opencode_model_display_name(),
        ),
        (
            texts::tui_label_opencode_model_id().to_string(),
            form.opencode_model_id.value.trim().to_string(),
        ),
        (
            texts::tui_label_context_limit().to_string(),
            form.opencode_model_context_limit.value.trim().to_string(),
        ),
        (
            texts::tui_label_output_limit().to_string(),
            form.opencode_model_output_limit.value.trim().to_string(),
        ),
    ]
    .into_iter()
    .map(|(label, value)| {
        Row::new(vec![
            Cell::from(cell_pad(&label)),
            Cell::from(if value.is_empty() {
                texts::tui_na().to_string()
            } else {
                value
            }),
        ])
    })
    .collect::<Vec<_>>();

    let mut state = TableState::default();
    state.select(Some(selected));
    let table = Table::new(
        rows,
        [Constraint::Length(label_col_width), Constraint::Min(10)],
    )
    .header(header)
    .row_highlight_style(selection_style(theme))
    .highlight_symbol(highlight_symbol(theme));
    frame.render_stateful_widget(table, chunks[1], &mut state);

    let editor_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(focus_block_style(editing, theme))
        .title(if editing {
            texts::tui_form_editing_title()
        } else {
            texts::tui_form_input_title()
        });
    frame.render_widget(editor_block.clone(), chunks[2]);
    let editor_inner = editor_block.inner(chunks[2]);

    let field = DETAIL_FIELDS[selected];
    if let Some(input) = form.input(field) {
        let (visible, cursor_x) =
            visible_text_window(&input.value, input.cursor, editor_inner.width as usize);
        frame.render_widget(
            Paragraph::new(Line::raw(visible)).wrap(Wrap { trim: false }),
            editor_inner,
        );

        if editing {
            let x = editor_inner.x + cursor_x.min(editor_inner.width.saturating_sub(1));
            frame.set_cursor_position((x, editor_inner.y));
        }
    } else {
        let hint = match field {
            ProviderAddField::OpenCodeModelId => texts::tui_opencode_model_edit_id_title(),
            ProviderAddField::OpenCodeModelName => texts::tui_opencode_model_edit_name_title(),
            ProviderAddField::OpenCodeModelContextLimit => {
                texts::tui_opencode_model_edit_input_limit_title()
            }
            ProviderAddField::OpenCodeModelOutputLimit => {
                texts::tui_opencode_model_edit_output_limit_title()
            }
            _ => "",
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                hint,
                Style::default().fg(theme.dim),
            )])),
            editor_inner,
        );
    }
}
