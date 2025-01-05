use std::io;

use crate::parser::{self, highlighting::{get_highlight_tokens, HighlightToken, HighlightTokenType}, syntax_tree::{self, Expression}, tokens::Token};
use super::{builtin_functions, commands, config::Config, executor::{RuntimeError, Value}, user_scripts::{self, ScriptError}};

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

pub enum FunctionBody {
    User(Expression),
    Builtin(Box<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError>>),
}

pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: FunctionBody,
}

pub struct Context<'a> {
    pub history: Vec<HistoryEntry>,
    pub history_scroll: u16,
    pub current_line: String,
    pub vars: Vec<(String, Value)>,
    pub functions: Vec<FunctionDef>,
    pub parent_context: Option<&'a Context<'a>>, // for temporary function contexts
}

impl Context<'_> {
    pub fn from_context<'a>(context: &'a Context) -> Context<'a> {
        let mut new_context = Context::default();
        new_context.parent_context = Some(context);
        new_context
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.vars.iter()
            .find(|var| var.0 == name)
            .map(|(_, value)| value)
            .or_else(|| self.parent_context.and_then(|ctx| ctx.get_var(name)))
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.iter()
            .find(|func| &func.name == name)
            .or_else(|| self.parent_context.and_then(|ctx| ctx.get_function(name)))
    }

    pub fn set_var(&mut self, identifier: String, value: Value) {
        let existing_index = self.vars.iter().position(|(name, _)| name == &identifier);
        match existing_index {
            Some(index) => self.vars[index] = (identifier, value),
            None => self.vars.push((identifier, value)),
        };
    }

    pub fn set_function(&mut self, definition: FunctionDef) {
        let existing_index = self.functions.iter().position(|f| f.name == definition.name);
        match existing_index {
            Some(index) => self.functions[index] = definition,
            None => self.functions.push(definition),
        }
    }

    // history_msg will highlight input, history_text will output as pure text
    pub fn push_history_msg(&mut self, msg: &str) {
        let tokens = get_highlight_tokens(msg);
        self.history.push(HistoryEntry {tokens, is_output: true});
    }

    pub fn push_history_text(&mut self, msg: &str) {
        self.history.push(HistoryEntry {
            tokens: vec![HighlightToken {text: msg.to_string(), kind: HighlightTokenType::Identifier}],
            is_output: true,
        });
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        let mut ctx = Context {
            history: Vec::new(),
            history_scroll: 0,
            current_line: String::new(),
            vars: Vec::new(),
            functions: Vec::new(),
            parent_context: None,
        };
        ctx.vars.push(("ans".to_string(), Value::Number(0.0)));
        ctx
    }
}

pub struct App<'a> {
    pub context: Context<'a>,
    pub config: Config,
    pub exit: bool,
}

impl App<'_> {
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

impl App<'_> {
    pub fn new() -> App<'static> {
        let mut app = App::new_raw();
        // probably change this in the future to print where config is loaded
        // also might add a tip for if no config dir exists
        if let Err(ScriptError::ScriptNotFound(_)) = app.run_script("init") {
            app.context.push_history_msg("create init.txt inside your config dir to load a default script");
        }

        for (name, params, func) in builtin_functions::FUNCTIONS {
            app.context.set_function(FunctionDef {
                name: name.to_string(),
                params: params.iter().map(|s| s.to_string()).collect(),
                body: FunctionBody::Builtin(Box::new(func)),
            });
        }

        app
    }

    pub fn new_raw() -> App<'static> {
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
                self.context.push_history_msg(&e.to_string());
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
            Ok(tree) => match self.context.execute(tree) {
                Ok(value) => {
                    let output = value.output_tokens();
                    self.context.set_var("ans".to_string(), value);
                    output
                },
                Err(e) => get_highlight_tokens(&e.to_string()),
            },
            Err(e) => get_highlight_tokens(&e.to_string()),
        };

        self.context.history.push(HistoryEntry {tokens: execution_response, is_output: true});
    }
}
