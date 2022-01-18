use crate::data::{Error, Result};
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq)]
enum InnerToken {
    #[token(";")]
    Semi,

    #[token("=")]
    Eq,

    #[token("{")]
    CurlyL,

    #[token("}")]
    CurlyR,

    #[token("(")]
    ParenL,

    #[token(")")]
    ParenR,

    #[token("<")]
    AngleL,

    #[token(">")]
    AngleR,

    #[regex(r"[a-zA-Z][-_a-zA-Z0-9]*", str)]
    Id(String),

    #[regex("'([^'\\t\\n\\f])*'", strip_quotes)]
    QSymbol(String),

    #[regex(r"\$e\.([a-zA-Z][-_a-zA-Z0-9]*|[0-9]+)", str)]
    EVar(String),

    #[regex(r"\$s\.([a-zA-Z][-_a-zA-Z0-9]*|[0-9]+)", str)]
    SVar(String),

    #[regex(r"\$t\.([a-zA-Z][-_a-zA-Z0-9]*|[0-9]+)", str)]
    TVar(String),

    #[token("$MODULE")]
    Module,

    #[regex(r"[ \r\t\n\f]+", logos::skip)]
    Ws,

    #[regex(r"/\*[^\*]*\*/", logos::skip)]
    Comment,

    #[error]
    Error,
}

pub enum Token {
    Semi,
    Eq,
    CurlyL,
    CurlyR,
    ParenL,
    ParenR,
    AngleL,
    AngleR,
    Symbol(String),
    EVar(String),
    SVar(String),
    TVar(String),
    Module,
}

pub fn tokens(input: &str) -> Result<Vec<Token>> {
    InnerToken::lexer(input).map(convert).collect()
}

fn convert(in_token: InnerToken) -> Result<Token> {
    match in_token {
        InnerToken::Semi => Ok(Token::Semi),
        InnerToken::Eq => Ok(Token::Eq),
        InnerToken::CurlyL => Ok(Token::CurlyL),
        InnerToken::CurlyR => Ok(Token::CurlyR),
        InnerToken::ParenL => Ok(Token::ParenL),
        InnerToken::ParenR => Ok(Token::ParenR),
        InnerToken::AngleL => Ok(Token::AngleL),
        InnerToken::AngleR => Ok(Token::AngleR),
        InnerToken::Id(s) => Ok(Token::Symbol(s)),
        InnerToken::QSymbol(s) => Ok(Token::Symbol(s)),
        InnerToken::EVar(s) => Ok(Token::EVar(s)),
        InnerToken::SVar(s) => Ok(Token::SVar(s)),
        InnerToken::TVar(s) => Ok(Token::TVar(s)),
        InnerToken::Module => Ok(Token::Module),
        InnerToken::Ws => Err(Error::IllegalState),
        InnerToken::Comment => Err(Error::IllegalState),
        InnerToken::Error => Err(Error::Lexing),
    }
}

fn str(lex: &Lexer<InnerToken>) -> Option<String> {
    Some(lex.slice().to_string())
}

fn strip_quotes(lex: &Lexer<InnerToken>) -> Option<String> {
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_string())
}

#[test]
fn test_curly_l() {
    let mut lex = InnerToken::lexer(" { ");
    assert_eq!(lex.next(), Some(InnerToken::CurlyL));
    assert_eq!(lex.slice(), "{");
    assert_eq!(lex.next(), None);
}

#[test]
fn test_id() {
    let mut lex = InnerToken::lexer(" my_id ");
    assert_eq!(lex.next(), Some(InnerToken::Id("my_id".to_string())));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_ws() {
    let input = "
    /* Contains basic simple tests form turchin book */
    $MODULE Test;
    ";
    let mut lex = InnerToken::lexer(input);
    assert_eq!(lex.next(), Some(InnerToken::Module));
    assert_eq!(lex.next(), Some(InnerToken::Id("Test".to_string())));
    assert_eq!(lex.next(), Some(InnerToken::Semi));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_q_symbols() {
    let mut lex = InnerToken::lexer(" 'a' '////^^^$$$'");
    assert_eq!(lex.next(), Some(InnerToken::QSymbol("a".to_string())));
    assert_eq!(
        lex.next(),
        Some(InnerToken::QSymbol("////^^^$$$".to_string()))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_vars() {
    let mut lex = InnerToken::lexer(" $e.1 $t.T_VAR $s.s");
    assert_eq!(lex.next(), Some(InnerToken::EVar("$e.1".to_string())));
    assert_eq!(lex.next(), Some(InnerToken::TVar("$t.T_VAR".to_string())));
    assert_eq!(lex.next(), Some(InnerToken::SVar("$s.s".to_string())));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_comments() {
    let mut lex = InnerToken::lexer(" /* 123 */ Symbol");
    assert_eq!(lex.next(), Some(InnerToken::Id("Symbol".to_string())));
    assert_eq!(lex.slice(), "Symbol");
}

#[test]
fn test_error_1() {
    let mut lex = InnerToken::lexer(" ! ");
    assert_eq!(lex.next(), Some(InnerToken::Error));
    assert_eq!(lex.slice(), "!");
    assert_eq!(lex.span(), 1..2);
    assert_eq!(lex.next(), None);
}

#[test]
fn test_error_2() {
    let mut lex = InnerToken::lexer(" !! ");
    assert_eq!(lex.next(), Some(InnerToken::Error));
    assert_eq!(lex.slice(), "!");
    assert_eq!(lex.span(), 1..2);
    assert_eq!(lex.next(), Some(InnerToken::Error));
    assert_eq!(lex.span(), 2..3);
    assert_eq!(lex.slice(), "!");
    assert_eq!(lex.next(), None);
}
