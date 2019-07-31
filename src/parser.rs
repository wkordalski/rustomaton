use crate::parser::Token::*;
use crate::regex::Operations;
use logos::Logos;
use std::collections::VecDeque;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[end]
    End,

    #[error]
    Error,

    #[token = "|"]
    Union,

    #[token = "("]
    Lpar,

    #[token = ")"]
    Rpar,

    #[token = "."]
    Dot,

    #[token = "*"]
    Kleene,

    #[token = "?"]
    Question,

    #[token = "+"]
    Plus,

    #[token = "𝜀"]
    Epsilon,

    #[regex = "[^|+().*?𝜀]"]
    Letter,
}

/*
    (REG) > REG* = REG+ = REG? > REGREG > REG|REG

    REG ::= .
            𝜀
            CHAR
            (REG)
            REG*
            REG+
            REG?
            REGREG
            REG|REG
*/

pub fn tokens(s: &str) -> VecDeque<(Token, &str)> {
    let mut lexer = Token::lexer(s);
    let mut tokens = VecDeque::new();

    while lexer.token != Token::End {
        tokens.push_back((lexer.token.clone(), lexer.slice()));
        lexer.advance();
    }

    tokens
}

pub fn peak(tokens: &mut VecDeque<(Token, &str)>) -> Option<Token> {
    tokens.get(0).map(|x| x.0.clone())
}

pub fn read_union(tokens: &mut VecDeque<(Token, &str)>) -> Result<Operations<char>, String> {
    let mut u = Vec::new();

    loop {
        u.push(read_concat(tokens)?);
        if peak(tokens) == Some(Union) {
            tokens.pop_front();
        } else {
            break;
        }
    }

    if u.len() == 1 {
        Ok(u.pop().unwrap())
    } else {
        Ok(Operations::Union(u))
    }
}

pub fn read_paren(tokens: &mut VecDeque<(Token, &str)>) -> Result<Operations<char>, String> {
    if peak(tokens) != Some(Lpar) {
        return Err("Expected left parenthesis.".to_string());
    }
    tokens.pop_front();

    let o = read_union(tokens)?;

    if peak(tokens) != Some(Rpar) {
        return Err("Expected right parenthesis.".to_string());
    }
    tokens.pop_front();
    Ok(read_quantif(tokens, o))
}

pub fn read_quantif(
    tokens: &mut VecDeque<(Token, &str)>,
    mut o: Operations<char>,
) -> Operations<char> {
    while let Some(x) = peak(tokens) {
        if x == Plus {
            o = Operations::Repeat(Box::new(o), 1, None);
        } else if x == Kleene {
            o = Operations::Repeat(Box::new(o), 0, None);
        } else if x == Question {
            o = Operations::Repeat(Box::new(o), 0, Some(1));
        } else {
            break;
        }
        tokens.pop_front();
    }

    return o;
}

pub fn read_letter(tokens: &mut VecDeque<(Token, &str)>) -> Result<Operations<char>, String> {
    if let Some(x) = peak(tokens) {
        let o = if x == Dot {
            Operations::Dot
        } else if x == Epsilon {
            Operations::Epsilon
        } else if x == Letter {
            Operations::Letter(tokens[0].1.chars().next().unwrap())
        } else {
            return Err("Expected letter".to_string());
        };
        tokens.pop_front();
        Ok(read_quantif(tokens, o))
    } else {
        Err("Expected letter".to_string())
    }
}

pub fn read_concat(tokens: &mut VecDeque<(Token, &str)>) -> Result<Operations<char>, String> {
    let mut c = Vec::new();
    while let Some(x) = peak(tokens) {
        if x == Dot || x == Epsilon || x == Letter {
            c.push(read_letter(tokens)?);
        } else if x == Lpar {
            c.push(read_paren(tokens)?);
        } else if x == Kleene || x == Plus || x == Question {
            return Err(format!(
                "Unexpected {}",
                tokens[0].1.chars().next().unwrap()
            ));
        } else if x == Rpar || x == Union || x == End {
            break;
        } else {
            unreachable!()
        }
    }

    if c.len() == 1 {
        Ok(c.pop().unwrap())
    } else {
        Ok(Operations::Concat(c))
    }
}
