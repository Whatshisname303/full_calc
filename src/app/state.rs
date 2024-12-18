use std::io;
use std::collections::HashMap;

use crate::parser::{self, syntax_tree, tokens::Token};
use super::{config::{self, Config}, executor::Value};

use ratatui::{
    prelude::*,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    DefaultTerminal,
};


#[derive(Debug)]
pub struct App {
    pub counter: u8,
    pub history: Vec<String>,
    pub current_line: String,
    pub vars: HashMap<String, Value>,
    // pub functions: HashMap<String, idk>,
    pub config: Config,
    pub exit: bool,
}


impl App {
    pub fn new() -> App {
        let mut app = App::new_blank();
        app.vars.insert("ans".to_string(), Value::Number(0.0));
        app.run_script(config::load_index());
        app
    }

    pub fn new_blank() -> App {
        App {
            counter: 0,
            history: Vec::new(),
            current_line: String::new(),
            vars: HashMap::new(),
            config: Config::default(),
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_down(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_down(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => self.execute_line(),
            KeyCode::Backspace => {self.current_line.pop();},
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Char(char) => {
                self.current_line.push(char);
            },
            _ => {},
        }
    }

    fn run_script(&mut self, script: String) {
        for line in script.lines() {
            line.clone_into(&mut self.current_line);
            self.execute_line();
        }
    }

    fn execute_line(&mut self) {
        let mut tokens = parser::tokens::tokenize(&self.current_line).unwrap();
        // don't unwrap forever pls

        self.history.push(self.current_line.clone());
        self.current_line.clear();

        let processed = execute_commands(self, &tokens);
        if processed {
            return;
        }

        if tokens.get(0).is_some_and(|token| token.is_binary_op()) {
            tokens.insert(0, Token::Identifier("ans".to_string()));
        }

        let execution_response = match syntax_tree::generate_syntax_tree(tokens) {
            Ok(tree) => match self.execute(tree) {
                Ok(value) => {
                    let screen_output = value.to_display_string();
                    self.vars.insert("ans".to_string(), value);
                    screen_output
                },
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        };

        self.history.push(execution_response);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
        self.current_line.push('a');
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

// need to figure out what to do with processed commands
// should remaining tokens be executed normally?
// should there be a response for whether a command was processed?
fn execute_commands(app: &mut App, tokens: &Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

    let mut is_processed = true;

    match tokens[0] {
        Token::Identifier(ref word) => match word.as_str() {
            "clear" => app.history.clear(),
            "quit" | "exit" => app.exit(),
            "def" => todo!(),
            "set" => todo!(), // probably a good keyword for config
            _ => is_processed = false,
        },
        _ => is_processed = false,
    }

    is_processed
}
