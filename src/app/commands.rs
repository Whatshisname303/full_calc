use crate::parser::tokens::Token;
use super::{config::Panel, state::{App, Context}};

// returns is_handled, errors are handled without warning caller
pub fn handle_commands(app: &mut App, tokens: &Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

    let mut is_handled = true;

    match tokens[0] {
        Token::Identifier(ref word) => match word.as_str() {
            "clear" => clear_history(app),
            "quit" | "exit" => exit_app(app),
            "reload" => reload_app(app, tokens),
            "use" => use_scope(app, tokens),
            "load" => load_script(app, tokens),
            "def" => declare_function(app, tokens),
            "config" => update_config(app, tokens),
            "show" => show_page(app, tokens),
            "panel" => toggle_panel(app, tokens),
            _ => is_handled = false,
        },
        _ => is_handled = false,
    };

    is_handled
}

fn clear_history(app: &mut App) {
    app.context.history.clear();
}

fn exit_app(app: &mut App) {
    app.exit = true;
}

fn reload_app(app: &mut App, tokens: &Vec<Token>) {
    let is_raw = tokens.get(1).is_some_and(|token| token.is_from_str("raw"));
    app.context = Context::default();

    if !is_raw {
        let _ = app.run_script("init");
    }
}

fn use_scope(app: &mut App, tokens: &Vec<Token>) {
    todo!();
}

fn load_script(app: &mut App, tokens: &Vec<Token>) {
    todo!();
}

fn declare_function(app: &mut App, tokens: &Vec<Token>) {
    todo!();
}

fn update_config(app: &mut App, tokens: &Vec<Token>) {
    todo!();
}

fn show_page(app: &mut App, tokens: &Vec<Token>) {
    todo!();
}

fn toggle_panel(app: &mut App, tokens: &Vec<Token>) {
    let mut print_err_msg = || {
        app.context.history.push("usage: panel <vars/autocomplete/preview> <optional: on/off>".to_string());
    };

    let mut toggle_panel = |panel: Panel| {
        match tokens.get(2) {
            Some(Token::Identifier(ident)) => match ident.as_str() {
                "on" => {
                    if !app.config.panels.contains(&panel) {
                        app.config.panels.push(panel);
                    }
                },
                "off" => {
                    if let Some(index) = app.config.panels.iter().position(|p| p == &panel) {
                        app.config.panels.remove(index);
                    }
                },
                _ => print_err_msg(),
            },
            None => {
                let index = app.config.panels.iter().position(|p| p == &panel);
                match index {
                    Some(index) => {app.config.panels.remove(index);},
                    None => {app.config.panels.push(panel);},
                };
            },
            _ => print_err_msg(),
        };
    };

    match tokens.get(1) {
        Some(Token::Identifier(ident)) => match ident.as_str() {
            "vars" => toggle_panel(Panel::Variables),
            "autocomplete" => toggle_panel(Panel::Autocomplete),
            "preview" => toggle_panel(Panel::ExpPreview),
            _ => print_err_msg(),
        },
        _ => print_err_msg(),
    };

    // let st: String = app.config.panels.iter().map(|p| format!(" {:?} ", p)).collect();
    // app.context.history.push(st);
}
