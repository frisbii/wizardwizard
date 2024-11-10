use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEvent, MouseEventKind};
use ratatui::widgets::{List, ListState, Scrollbar, ScrollbarState};

// stores the state of the program
pub struct App {
    pub exit: bool,
    pub message_history: Vec<Message>,
    pub message_scroll_state: ListState
}

impl App {

    // constructor
    pub fn new() -> App {
        App {
            exit: false,
            message_history: vec![],
            message_scroll_state: ListState::default()
        }
    }

    // update state based on terminal events
    pub fn handle_terminal_events(&mut self) -> io::Result<()> {
        match event::read()? {

            // on key press
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.exit(),

                    KeyCode::Up    => self.messages_scroll_up(),
                    KeyCode::Down  => self.messages_scroll_down(),

                    
                    _ => ()
                }
            }

            // on mouse capture
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::ScrollUp    => self.messages_scroll_up(),
                    MouseEventKind::ScrollDown  => self.messages_scroll_down(),
                    _ => ()
                }
            }

            // do nothing on any other event
            _ => {}

        }
        Ok(())
    }

    // exit at the end of this loop iteration
    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn post_message(&mut self, message: Message) {
        self.message_history.push(message);
    }

    pub fn messages_scroll_down(&mut self) {
        *self.message_scroll_state.offset_mut() = self.message_scroll_state.offset().saturating_sub(1);
    }
    
    pub fn messages_scroll_up(&mut self) {
        *self.message_scroll_state.offset_mut() = self.message_scroll_state.offset().saturating_add(1);
    }

}

#[derive(PartialEq, Clone)]
pub enum MessageType {
    User,
    Game
}

#[derive(Clone)]
pub struct Message {
    pub text: String,
    pub msg_type: MessageType
}