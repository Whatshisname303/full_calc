use std::io;
use std::collections::HashMap;

use crate::parser::{self, highlighting::{get_highlight_tokens, HighlightToken}, syntax_tree::{self, Expression}, tokens::Token};
use super::{commands, config::Config, executor::Value, user_scripts::{self, ScriptError}};

use ratatui::{
    prelude::*,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    DefaultTerminal,
};

#[derive(Debug)]
pub struct HistoryEntry {
    pub tokens: Vec<HighlightToken>,
    pub is_output: bool,
}

//todo: figure out where this struct should go
#[derive(Debug)]
pub struct UserFunction {
    name: String,
    params: Vec<String>,
    body: Expression,
}

#[derive(Debug)]
pub struct Context {
    pub history: Vec<HistoryEntry>,
    pub history_scroll: u16,
    pub current_line: String,
    pub vars: Vec<(String, Value)>,
    pub user_functions: HashMap<String, UserFunction>,
}

impl Default for Context {
    fn default() -> Self {
        let mut ctx = Context {
            history: Vec::new(),
            history_scroll: 0,
            current_line: String::new(),
            vars: Vec::new(),
            user_functions: HashMap::new(),
        };
        ctx.vars.push(("ans".to_string(), Value::Number(0.0)));
        ctx
    }
}

#[derive(Debug)]
pub struct App {
    pub context: Context,
    pub config: Config,
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
            app.push_history_msg("create init.txt inside your config dir to load a default script");
        }
        app
    }

    pub fn new_raw() -> App {
        App {
            context: Context::default(),
            config: Config::default(),
            exit: false,
        }
    }

    fn handle_key_down(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => self.execute_current_line(),
            KeyCode::Backspace => {self.context.current_line.pop();},
            KeyCode::Down => {self.context.history_scroll += 1;},
            KeyCode::Up => {
                if self.context.history_scroll > 0 {
                    self.context.history_scroll -= 1;
                }
            },
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
        let tokens = parser::tokens::tokenize(&self.context.current_line);

        let highlight_tokens = get_highlight_tokens(&self.context.current_line);
        self.context.history.push(HistoryEntry {tokens: highlight_tokens, is_output: false});

        self.context.current_line.clear();

        let mut tokens = match tokens {
            Ok(tokens) => tokens,
            Err(e) => {
                self.push_history_msg(&e.to_string());
                return;
            }
        };

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
                    let output = value.output_tokens();
                    self.set_var("ans".to_string(), value);
                    output
                },
                Err(e) => get_highlight_tokens(&e.to_string()),
            },
            Err(e) => get_highlight_tokens(&e.to_string()),
        };

        self.context.history.push(HistoryEntry {tokens: execution_response, is_output: true});
    }

    pub fn push_history_msg(&mut self, msg: &str) {
        let tokens = get_highlight_tokens(msg);
        self.context.history.push(HistoryEntry {tokens, is_output: true});
    }
}
