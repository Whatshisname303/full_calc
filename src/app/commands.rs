use crate::parser::tokens::Token;
use super::state::App;

// returns is_handled, errors are handled without warning caller
fn handle_commands(app: &mut App, tokens: &Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

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
            _ => false,
        },
        _ => false,
    }
}

fn clear_history(app: &mut App) -> bool {
    app.history.clear();
    true
}

fn exit_app(app: &mut App) -> bool {
    app.exit = true;
    true
}

fn reload_app(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn use_scope(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn load_script(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn declare_function(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn update_config(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn show_page(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}

fn toggle_panel(app: &mut App, tokens: &Vec<Token>) -> bool {
    todo!();
}
