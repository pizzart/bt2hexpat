use std::collections::VecDeque;

use crate::ast_bt::token::TokenKind;

pub fn tokenize(content: String) -> VecDeque<(TokenKind, String)> {
    let mut tokens = VecDeque::new();
    let mut current = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek() == Some(&'/') => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), current));
                }
                current = ch.to_string();
                for c in chars.by_ref() {
                    current.push(c);
                    if c == '\n' {
                        break;
                    }
                }
                // tokens.push_back((
                //     TokenKind::Comment(current.clone()),
                //     current.drain(..).collect(),
                // ));
                current.clear();
            }
            '/' if chars.peek() == Some(&'*') => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), current));
                }
                current = ch.to_string();
                while let Some(c) = chars.next() {
                    current.push(c);
                    if c == '*' && chars.peek() == Some(&'/') {
                        current.push(chars.next().unwrap());
                        break;
                    }
                }
                // tokens.push_back((
                //     TokenKind::Comment(current.clone()),
                //     current.drain(..).collect(),
                // ));
                current.clear();
            }
            '#' => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), current));
                }
                current = ch.to_string();
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                    current.push(c);
                }
                tokens.push_back((
                    TokenKind::CPPDirective(current.clone()),
                    std::mem::take(&mut current),
                ));
            }
            '"' => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), current));
                }
                current = ch.to_string();
                while let Some(c) = chars.next() {
                    current.push(c);
                    if c == '"' && current.len() == 2 {
                        tokens.push_back((current.parse().unwrap(), std::mem::take(&mut current)));
                        break;
                    }
                    if c != '\\' && chars.peek() == Some(&'"') {
                        current.push(chars.next().unwrap());
                        tokens.push_back((current.parse().unwrap(), std::mem::take(&mut current)));
                        break;
                    }
                }
            }
            '\'' => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), current));
                }
                current = ch.to_string();
                while let Some(c) = chars.next() {
                    current.push(c);
                    if c != '\\' && chars.peek() == Some(&'\'') {
                        tokens.push_back((current.parse().unwrap(), std::mem::take(&mut current)));
                        current.push(chars.next().unwrap());
                        break;
                    }
                }
            }
            _ if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push_back((current.parse().unwrap(), std::mem::take(&mut current)));
                }
            }
            _ if !current.is_empty()
                && let TokenKind::Unknown(_) = format!("{}{}", current, ch).parse().unwrap() =>
            {
                tokens.push_back((current.parse().unwrap(), std::mem::take(&mut current)));
                current = ch.to_string();
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push_back((current.parse().unwrap(), current));
    }

    tokens
}
