use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border::Set, text::{Span, Text}, widgets::{Block, BorderType, Borders, List, ListDirection, Paragraph, Widget}, DefaultTerminal, Frame
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
    ] = Layout::vertical([Constraint::Percentage(70), Constraint::Fill(1)]).areas(area_ctr);


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

    let mut text: Vec<Text> = vec![];
    for message in app.message_history.clone().into_iter().rev() {
        let color = if message.msg_type == MessageType::Game {
            Color::LightCyan
        } else {
            Color::White
        };
        text.push(Text::from(format!("{}\n\n", message.text.clone())).style(color));
    }

    let messages = List::new(text).direction(ListDirection::BottomToTop);

    frame.render_widget(text_area_block, text_area);
    frame.render_stateful_widget(messages, inner_text_area, &mut app.message_scroll_state);

    // =========================================================================================
    // INPUT AREA

    let input_area_block = Block::bordered().border_style(Style::new().bg(Color::Black).fg(Color::LightBlue));

    

    frame.render_widget(input_area_block, input_area);
}