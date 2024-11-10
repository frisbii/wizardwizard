use std::io;

mod game;
mod app;
mod ui;

use app::{Message, MessageType};
use ratatui::DefaultTerminal;

use crate::{
    app::App,
    ui::ui
};

fn main() -> io::Result<()> {

    // initialize new terminal in the alternate screen buffer
    let mut terminal = ratatui::init();     
    terminal.clear()?;

    // create the app state, start program loop
    let mut app = App::new();

    let m = Message {
        text: "adfa".to_string(),
        msg_type: MessageType::Game,
    };
    app.post_message(m);
    let m = Message {
        text: "asdfacvzcxv THIS SIF A F AEFLASDKF AV ERY LONG STRIN GYAYAFYASDFASYF".to_string(),
        msg_type: MessageType::User,
    };
    app.post_message(m);

    let res = run_app(&mut terminal, &mut app);

    // restore the terminal's original state
    ratatui::restore();
    
    res

}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    
    // run until quit
    // two steps to the application loop
    //      1)  redraw the frame
    //      2)  handle events, if any
    while app.exit == false {
        // ui handled in separate module
        terminal.draw(|frame| ui(frame, app))?;
        // state updates handled in app state
        app.handle_terminal_events()?;
    }

    Ok(())

}