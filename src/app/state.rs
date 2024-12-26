use std::io;
use std::collections::HashMap;

use crate::parser::{self, syntax_tree::{self, Expression}, tokens::Token};
use super::{commands, config::Config, executor::Value, user_scripts::{self, ScriptError}};

use ratatui::{
    prelude::*,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    DefaultTerminal,
};

//todo: figure out where this struct should go
#[derive(Debug)]
struct UserFunction {
    name: String,
    params: Vec<String>,
    body: Expression,
}

#[derive(Debug)]
pub struct Context {
    pub history: Vec<String>,
    pub current_line: String,
    pub vars: HashMap<String, Value>,
    pub user_functions: HashMap<String, UserFunction>,
    pub config: Config,
}

impl Default for Context {
    fn default() -> Self {
        let mut ctx = Context {
            history: Vec::new(),
            current_line: String::new(),
            vars: HashMap::new(),
            user_functions: HashMap::new(),
            config: Config::default(),
        };
        ctx.vars.insert("ans".to_string(), Value::Number(0.0));
        ctx
    }
}

#[derive(Debug)]
pub struct App {
    pub context: Context,
    pub exit: bool,
}

impl App {
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
}

impl App {
    pub fn new() -> App {
        let mut app = App::new_raw();
        // probably change this in the future to print where config is loaded
        // also might add a tip for if no config dir exists
        if let Err(ScriptError::ScriptNotFound(_)) = app.run_script("init") {
            app.context.history.push("create init.txt inside your config dir to load a default script".to_string());
        }
        app
    }

    pub fn new_raw() -> App {
        App {
            context: Context::default(),
            exit: false,
        }
    }

    fn handle_key_down(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => self.execute_current_line(),
            KeyCode::Backspace => {self.context.current_line.pop();},
            KeyCode::Char(char) => {
                self.context.current_line.push(char);
            },
            _ => {},
        };
    }

    pub fn run_script(&mut self, script_name: &str) -> Result<(), ScriptError> {
        let script = user_scripts::read_script(script_name)?;
        for line in script.lines() {
            line.clone_into(&mut self.context.current_line);
            self.execute_current_line();
        }
        Ok(())
    }

    fn execute_current_line(&mut self) {
        let mut tokens = parser::tokens::tokenize(&self.context.current_line).unwrap();
        // don't unwrap forever pls

        self.context.history.push(self.context.current_line.clone());
        self.context.current_line.clear();

        let processed = commands::handle_commands(self, &tokens);
        if processed {
            return;
        }

        if tokens.get(0).is_some_and(|token| token.is_binary_op()) {
            tokens.insert(0, Token::Identifier("ans".to_string()));
        }

        let execution_response = match syntax_tree::generate_syntax_tree(tokens) {
            Ok(tree) => match self.execute(tree) {
                Ok(value) => {
                    let screen_output = value.to_string();
                    self.context.vars.insert("ans".to_string(), value);
                    screen_output
                },
                Err(e) => format!("{:?}", e),
            },
            Err(e) => e.to_string(),
        };

        self.context.history.push(execution_response);
    }
}
