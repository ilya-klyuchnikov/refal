use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Lexing,
    Parsing,
    IllegalState,
}

#[derive(Debug)]
pub enum RefalObject {
    Symbol(String),
    StrBracketL,
    StrBracketR,
    FunBracketL,
    FunBracketR,
    EVar(String),
    SVar(String),
    TVar(String),
}

pub struct Sentence {
    pub pattern: Vec<RefalObject>,
    pub expression: Vec<RefalObject>,
}

pub struct Function {
    pub name: String,
    pub sentences: Vec<Sentence>,
}

pub struct RefalModule {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
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
    MatchMoveBorders(usize, usize),
    SetupTransition(usize),
    ConstrainLengthen(usize),

    MoveBorder,
    InsertStrBracketL,
    InsertStrBracketR,
    InsertFunBracketL,
    InsertFunBracketR,
    InsertSymbol(String),
    CopySymbol(usize),
    CopyExpr(usize),
    TransplantObject(usize),
    TransplantExpr(usize),
    CompleteStep,
    NextStep,
}
