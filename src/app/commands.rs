use crate::parser::tokens::Token;
use crate::parser::general_parsing;
use super::{config::Panel, state::{App, Context, FunctionDef, PopupName}, user_scripts::ScriptError};

// returns is_handled, errors are handled without warning caller
pub fn handle_commands(app: &mut App, tokens: &Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if let (Some(Token::Minus), Some(Token::Minus)) = (tokens.get(0), tokens.get(1)) {
        return true;
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
    app.context.history_scroll = 0;
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
    match tokens.get(1) {
        Some(Token::Identifier(scope_name)) => {
            let scope_name = scope_name.to_string() + ".";

            let new_vars: Vec<_> = app.context.vars.iter().filter_map(|(name, value)| {
                match name.starts_with(&scope_name) {
                    true => Some((name[scope_name.len()..].to_string(), value.clone())),
                    false => None,
                }
            }).collect();

            for (name, value) in new_vars {
                app.context.set_var(name, value);
            }

            let new_functions: Vec<_> = app.context.functions.iter().filter_map(|function_def| {
                match function_def.name.starts_with(&scope_name) {
                    true => Some(FunctionDef {
                        name: function_def.name[scope_name.len()..].to_string(),
                        params: function_def.params.clone(),
                        body: function_def.body.clone(),
                    }),
                    false => None,
                }
            }).collect();

            for function_def in new_functions {
                app.context.set_function(function_def);
            }
        },
        _ => app.context.push_history_text("usage: use <scope>"),
    }
}

fn load_script(app: &mut App, tokens: &Vec<Token>) {
    match tokens.get(1) {
        Some(Token::Identifier(script_name)) => {
            let file_path = script_name.replace(".", "/");
            match app.run_script(&file_path) {
                Ok(()) => (),
                Err(ScriptError::NoConfigPath) => {
                    app.context.push_history_text(
                        "create a 'config' folder at (will put path here later) or next to the executable to use scripts"
                    );
                },
                Err(ScriptError::ScriptNotFound(_)) => {
                    match (tokens.get(2), tokens.get(3)) {
                        (Some(Token::Div), Some(Token::Identifier(name2))) => {
                            app.context.push_history_text(&format!("script paths use '.' instead of '/' ex: {script_name}.{name2}"));
                        },
                        _ => app.context.push_history_text("script not found"),
                    }
                }
            }
        },
        _ => app.context.push_history_msg("usage: load <scriptname>"),
    }
}

fn declare_function(app: &mut App, tokens: &Vec<Token>) {
    match general_parsing::parse_function_definition(tokens) {
        Ok(function_definition) => app.context.set_function(function_definition),
        Err(e) => app.context.push_history_msg(&e.to_string()),
    }
}

fn update_config(app: &mut App, tokens: &Vec<Token>) {
    app.update_config(tokens);
}

fn show_page(app: &mut App, tokens: &Vec<Token>) {
    match tokens.get(1) {
        Some(Token::Identifier(ident)) => match ident.as_str() {
            "vars" => app.context.current_popup = Some(PopupName::Vars),
            "functions" => app.context.current_popup = Some(PopupName::Functions),
            "commands" => app.context.current_popup = Some(PopupName::Commands),
            _ => app.context.push_history_text("usage: show <vars/functions/commands>"),
        },
        _ => app.context.push_history_text("usage: show <vars/functions/commands>"),
    }
}

fn toggle_panel(app: &mut App, tokens: &Vec<Token>) {
    let err_msg = "usage: panel <vars/autocomplete/preview> <optional: on/off>";

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
                _ => app.context.push_history_msg(err_msg),
            },
            None => {
                let index = app.config.panels.iter().position(|p| p == &panel);
                match index {
                    Some(index) => {app.config.panels.remove(index);},
                    None => {app.config.panels.push(panel);},
                };
            },
            _ => app.context.push_history_msg(err_msg),
        };
    };

    match tokens.get(1) {
        Some(Token::Identifier(ident)) => match ident.as_str() {
            "vars" => toggle_panel(Panel::Variables),
            "autocomplete" => toggle_panel(Panel::Autocomplete),
            "preview" => toggle_panel(Panel::ExpPreview),
            _ => app.context.push_history_msg(err_msg),
        },
        _ => app.context.push_history_msg(err_msg),
    };
}
