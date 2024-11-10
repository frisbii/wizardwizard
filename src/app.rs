use pyo3::{prelude::*, types::IntoPyDict};

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use ratatui::widgets::ListState;

// stores the state of the program
pub struct App {
    pub exit: bool,

    pub message_history: Vec<Message>,
    pub message_scroll_state: ListState,

    pub input_str: String,
    pub input_char_index: usize,
    pub input_width: usize
}

impl App {

    // constructor
    pub fn new() -> App {
        App {
            exit: false,
            message_history: vec![],
            message_scroll_state: ListState::default(),
            input_str: String::default(),
            input_char_index: usize::default(),
            input_width: usize::default()
        }
    }

    // ############################################################################################################
    // TODO: OUTSIDE INTERFACES SHOULD POST NEW RESPONSES USING THIS
    pub fn post_message(&mut self, message: Message) {
        self.message_history.push(message);
    }
    
    // when input box is submitted, clear input and do something with the input_str
    fn submit_input(&mut self, t: MessageType) {

        // ############################################################################################################
        // TODO: CHANGE THIS TO INTERFACE WITH OUR AI STUFF; MAYBE DEDICATED RUST MODUlE FOR AI INTERFACING?
        //       WHATEVER WE DO, THE RESPONSE MESSAGE SHOULD BE PASSED TO post_message();
        let msg = Message {
            text: self.input_str.clone(),
            msg_type: t,
            
        };
        self.post_message(msg);
        // ############################################################################################################
        
        // reset input box for next input
        self.input_str.clear();
        self.reset_cursor();

        let to_post = App::call_ai(msg.clone(), t);

        self.post_message(to_post);

    }

    fn call_ai(msg: Message, t: MessageType) -> Message{

        // call AI stuff here
        let gil = Python::acquire_gil();
        let py = gil.python();
        
        Python::with_gil(|py| {
            let module = PyModule::new(py, "ai");

            let question = msg.clone();
            let actions = vec!["read letter", "open door", "break window"];

            let result = module.call1("generateAction", (actions, question))?.extract(py)?;

            let to_return = Message{
                text: result,
                msg_type: t.clone(),
            };
    
            return to_return;
        });
    }

    // (called by main)
    // update state based on terminal events
    pub fn handle_terminal_events(&mut self) -> io::Result<()> {
        match event::read()? {

            // on key press
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.exit(),

                    KeyCode::Up    => self.messages_scroll_up(),
                    KeyCode::Down  => self.messages_scroll_down(),

                    KeyCode::Char(c)        => {
                        if self.input_str.len() > self.input_width {
                            // TODO: we should really permit this. shouldn't be that hard to scroll horizontally with cursor
                            ()
                        } else {
                            self.enter_char(c)
                        }
                    },
                    KeyCode::Left               => self.move_cursor_left(), 
                    KeyCode::Right              => self.move_cursor_right(), 
                    KeyCode::Backspace          => self.delete_char(),
                    KeyCode::Enter              => {
                        if self.input_str.is_empty() {
                            ()
                        } else {
                            self.submit_input(MessageType::User);
                        }
                    },

                    // TO DELETE: TEST CODE ONLY
                    KeyCode::Delete => self.submit_input(MessageType::Game),
                    
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

    // ===================================================================================
    // ===================================================================================
    // INPUT BOX LOGIC: TAKEN FROM RATATUI EXAMPLES REPOSITORY
    fn messages_scroll_down(&mut self) {
        *self.message_scroll_state.offset_mut() = self.message_scroll_state.offset().saturating_sub(1);
    }

    fn messages_scroll_up(&mut self) {
        *self.message_scroll_state.offset_mut() = self.message_scroll_state.offset().saturating_add(1);
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.input_char_index.saturating_sub(1);
        self.input_char_index = self.clamp_cursor(cursor_moved_left);
    }
    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.input_char_index.saturating_add(1);
        self.input_char_index = self.clamp_cursor(cursor_moved_right);
    }
    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input_str.insert(index, new_char);
        self.move_cursor_right();
    }
    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input_str
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.input_char_index)
            .unwrap_or(self.input_str.len())
    }
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.input_char_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.input_char_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input_str.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input_str.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input_str = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input_str.chars().count())
    }
    fn reset_cursor(&mut self) {
        self.input_char_index = 0;
    }
    // ===================================================================================
    // ===================================================================================

}

#[derive(PartialEq, Clone)]
// determines if a message was sent by the user or the game (theming)
pub enum MessageType {
    User,
    Game
}

// stores a message that is to be displayed in the text_area of the ui
#[derive(Clone)]
pub struct Message {
    pub text: String,
    pub msg_type: MessageType
}