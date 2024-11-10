use ratatui::{
    layout::{Constraint, Layout, Position, Rect}, style::{Color, Style}, symbols::border::Set, text::Text, widgets::{Block, Borders, List, ListDirection, Paragraph}, Frame
};

use crate::app::{App, MessageType};

// redraw the frame 
pub fn ui(frame: &mut Frame, app: &mut App) {

    let area = frame.area();

    // =========================================================================================
    // SETUP CENTER AREA

    let h_pad = area.width / 20;
    let v_pad = area.height / 15;

    let area_ctr = Rect::new(
        h_pad,
        v_pad,
        area.width - 2 * h_pad,
        area.height - 2 * v_pad
    );

    let [
        text_area, 
        input_area
    ] = Layout::vertical([Constraint::Min(0), Constraint::Max(3)]).areas(area_ctr);


    // =========================================================================================
    // TEXT AREA

    let text_area_block = Block::bordered()
        .borders(Borders::TOP)
        .border_set(Set {
            top_left: "#",
            top_right: "",
            bottom_left: "",
            bottom_right: "",
            vertical_left: "",
            vertical_right: "",
            horizontal_top: "#",
            horizontal_bottom: "",
        })
        .border_style(Style::new().fg(Color::LightBlue));

    let inner_text_area = text_area_block.inner(text_area);

    let mut messages: Vec<Text> = vec![];
    for message in app.message_history.clone().into_iter().rev() {
        let color = match message.msg_type {
            MessageType::User => Color::White,
            MessageType::Game => Color::LightCyan,
            MessageType::Jesse => Color::Red
        };
       let text = wrap_line(message.text, inner_text_area.width.into());

        messages.push(Text::from(format!("{}\n\n", text)).style(color));
    }

    let messages = List::new(messages).direction(ListDirection::BottomToTop);

    frame.render_widget(text_area_block, text_area);
    frame.render_stateful_widget(messages, inner_text_area, &mut app.message_scroll_state);

    // =========================================================================================
    // INPUT AREA

    let iw = Paragraph::new(app.input_str.as_str())
        .block(Block::bordered().border_style(Style::new().fg(Color::LightBlue))
    );
    frame.set_cursor_position(Position::new(
        input_area.x + app.input_char_index as u16 + 1,
        input_area.y + 1
    ));

    // pass this to app so that it can cut off responses that are too long
    // TODO: figure out this 3 thing
    app.input_width = (input_area.width - 3).into();

    frame.render_widget(iw, input_area);
}

// Given a string and a max width, return a string with newlines inserted
// to fit within the width. Finds the closest whitespace character to the
// width threshold and newlines there. If none, newline at the width.
fn wrap_line(text: String, width: usize) -> String {
    if text.len() <= width {
        return text;
    } else {
        let mut s = "".to_string();
        for (i, c) in text[..width].chars().rev().enumerate() {
            if c == ' ' {
                s.push_str(&text[..(width - i)]);
                s.push('\n');
                s.push_str(&wrap_line(text[(width - i)..].to_string(), width));
                return s;
            }
        }
        s.push_str(&text[..(width)]);
        s.push('\n');
        s.push_str(&wrap_line(text[(width)..].to_string(), width));
        return s;
    }
}