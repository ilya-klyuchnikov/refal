use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parsing,
    IllegalState,
}

#[derive(Debug, PartialEq)]
pub enum Object {
    Symbol(String),
    StrBracketL,
    StrBracketR,
    FunBracketL,
    FunBracketR,
    EVar(String),
    SVar(String),
    TVar(String),
}

#[derive(Debug, PartialEq)]
pub struct Sentence {
    pub pattern: Vec<Object>,
    pub rewrite: Vec<Object>,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub sentences: Vec<Sentence>,
}

#[derive(Debug, PartialEq)]
pub struct RefalModule {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    MatchStart,
    MatchEmpty,
    MatchSymbolL(String),
    MatchSymbolR(String),
    MatchStrBracketL,
    MatchStrBracketR,
    MatchSVarL,
    MatchSVarR,
    MatchSVarLProj(usize),
    MatchSVarRProj(usize),
    MatchTVarL,
    MatchTVarR,
    MatchEVar,
    MatchEVarPrepare,
    MatchEVarLengthen,
    MatchEVarLProj(usize),
    MatchEVarRProj(usize),
    MatchMoveBorderL(usize),
    MatchMoveBorderR(usize),
    SetupTransition(usize),
    ConstrainLengthen(usize),

    RewriteStart,
    InsertStrBracketL,
    InsertStrBracketR,
    InsertFunBracketL,
    InsertFunBracketR,
    InsertSymbol(String),
    CopySymbol(usize),
    CopyExpr(usize),
    RewriteFinalize,
}
