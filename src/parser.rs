use crate::data::*;
use crate::lexer;
use crate::lexer::Token;
use std::vec::IntoIter;

pub fn parse_input(input: &str) -> Result<RefalModule> {
    let tokens = lexer::tokens(input)?;
    parse_module(tokens)
}

pub fn parse_module(tokens: Vec<Token>) -> Result<RefalModule> {
    let PreModule(name, pre_funcs) = pre_parse_module(tokens)?;
    let functions: Result<_> = pre_funcs.into_iter().map(function).collect();
    Ok(RefalModule {
        name,
        functions: functions?,
    })
}

fn pre_parse_module(tokens: Vec<Token>) -> Result<PreModule> {
    let input: &mut IntoIter<Token> = &mut tokens.into_iter();

    module(input)?;
    let module_name = symbol(input)?;
    semi(input)?;
    let mut pre_funcs = Vec::<PreFunc>::new();

    loop {
        let pre_func_opt = pre_func(input)?;
        match pre_func_opt {
            None => break,
            Some(pre_func) => pre_funcs.push(pre_func),
        }
    }
    Ok(PreModule(module_name, pre_funcs))
}

struct PreModule(String, Vec<PreFunc>);
struct PreFunc(String, Vec<Token>);

fn function(p_func: PreFunc) -> Result<Function> {
    let PreFunc(name, toks) = p_func;
    let mut sentences = Vec::<Sentence>::new();
    let tokens: &mut IntoIter<Token> = &mut toks.into_iter();
    loop {
        let sentence_opt = sentence(tokens)?;
        match sentence_opt {
            None => break,
            Some(s) => {
                sentences.push(s);
            }
        }
    }
    Ok(Function { name, sentences })
}

fn sentence(input: &mut IntoIter<Token>) -> Result<Option<Sentence>> {
    let mut pattern = Vec::<RefalObject>::new();
    let mut read = false;
    loop {
        match input.next() {
            None if read => return Err(Error::Parsing),
            None => return Ok(None),
            Some(Token::Eq) => break,
            Some(t) => {
                read = true;
                pattern.push(refal_object(t)?)
            }
        }
    }

    let mut expression: Vec<RefalObject> = Vec::<RefalObject>::new();
    loop {
        match input.next() {
            None => return Err(Error::Parsing),
            Some(Token::Semi) => break,
            Some(t) => expression.push(refal_object(t)?),
        }
    }
    Ok(Some(Sentence {
        pattern,
        expression,
    }))
}

fn refal_object(token: Token) -> Result<RefalObject> {
    match token {
        Token::Semi | Token::Eq | Token::CurlyL | Token::CurlyR | Token::Module => {
            Err(Error::Parsing)
        }
        Token::ParenL => Ok(RefalObject::StrBracketL),
        Token::ParenR => Ok(RefalObject::StrBracketR),
        Token::AngleL => Ok(RefalObject::FunBracketL),
        Token::AngleR => Ok(RefalObject::FunBracketR),
        Token::Symbol(s) => Ok(RefalObject::Symbol(s)),
        Token::EVar(s) => Ok(RefalObject::EVar(s)),
        Token::SVar(s) => Ok(RefalObject::SVar(s)),
        Token::TVar(s) => Ok(RefalObject::TVar(s)),
    }
}

fn pre_func(input: &mut IntoIter<Token>) -> Result<Option<PreFunc>> {
    let token_opt = input.next();
    match token_opt {
        None => Ok(None),
        Some(Token::Symbol(name)) => {
            let body: Vec<Token> = pre_func_body(input)?;
            Ok(Some(PreFunc(name, body)))
        }
        _ => Err(Error::Parsing),
    }
}

fn pre_func_body(input: &mut IntoIter<Token>) -> Result<Vec<Token>> {
    curly_l(input)?;
    let mut body = Vec::<Token>::new();
    loop {
        match input.next() {
            None => return Err(Error::Parsing),
            Some(token) => match token {
                Token::CurlyR => return Ok(body),
                t => body.push(t),
            },
        }
    }
}

fn module(input: &mut IntoIter<Token>) -> Result<()> {
    let token = input.next().ok_or(Error::Parsing)?;
    match token {
        Token::Module => Ok(()),
        _ => Err(Error::Parsing),
    }
}

fn symbol(input: &mut IntoIter<Token>) -> Result<String> {
    let token = input.next().ok_or(Error::Parsing)?;
    match token {
        Token::Symbol(s) => Ok(s),
        _ => Err(Error::Parsing),
    }
}

fn semi(input: &mut IntoIter<Token>) -> Result<()> {
    let token = input.next().ok_or(Error::Parsing)?;
    match token {
        Token::Semi => Ok(()),
        _ => Err(Error::Parsing),
    }
}

fn curly_l(input: &mut IntoIter<Token>) -> Result<()> {
    let token = input.next().ok_or(Error::Parsing)?;
    match token {
        Token::CurlyL => Ok(()),
        _ => Err(Error::Parsing),
    }
}
