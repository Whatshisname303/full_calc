use crate::parser::tokens::Token;
use super::state::{App, Context};

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
            "view" => toggle_panel(app, tokens),
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
    todo!();
}
