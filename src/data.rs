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
    // Checks that there is nothing between the border nodes.
    // Preprint - 3.8 (NIL)
    // PhD - 3.8 (ПРОВ)
    MatchEmpty,

    // Matches a symbol on the left.
    // PhD - 3.8 (3HAЧ,S)
    // Preprint - 3.8 LSC(S)
    MatchSymbolL(String),
    // Matches a symbol on the right
    // PhD - 3.8 (ЗНАЧЯ,S)
    // Preprint - 3.8 RSC(S)
    MatchSymbolR(String),

    // Matches a bracket on the left
    // PhD - 3.8 (СКОБ)
    // Preprint - 3.8 LB
    MatchStrBracketL,
    // Matches a bracket on the left
    // PhD - 3.8 (СКОБЯ)
    // Preprint - 3.8 RB
    MatchStrBracketR,

    // Matches a symbol on the left
    // PhD - 3.9 (СИМ)
    // Preprint - 3.9 LS
    MatchSVarL,
    // Matches a symbol variable on the left
    // PhD - 3.9 (СИМЯ)
    // Preprint - 3.9 RS
    MatchSVarR,

    // Matches a bound symbol variable on the left
    // PhD - 3.9 (СТС)
    // Preprint - 3.9 LSD
    MatchSVarLProj(usize),
    // Matches a bound symbol variable on the right
    // PhD - 3.9 (СТСЯ)
    // Preprint - 3.9 RSD
    MatchSVarRProj(usize),

    // Matches a term variable on the left.
    // Preprint - 3.9 LW
    // PhD - 3.9 (ТЕРМ,N)
    MatchTVarL,
    // Matches a term variable on the left.
    // Preprint - 3.9 RW
    // PhD - 3.9 (ТЕРМЯ,N)
    MatchTVarR,

    // Matches a closed expression variable.
    // Preprint - 3.9 CE
    // PhD - 3.9 (ЗАКР)
    MatchEVar,
    MatchEVarPrepare,
    MatchEVarLengthen,

    // Matches a bound expression variable on the left
    // Preprint - 3.9 LED(N)
    // PhD - 3.9 (CTB,N)
    MatchEVarLProj(usize),

    // Matches a bound expression variable on the right
    // Preprint - 3.9 RED(N)
    // PhD - 3.9 (СТВЯ,N)
    MatchEVarRProj(usize),

    // Moves the left border to a corresponding projection.
    // Preprint - 3.8 SB(N, M)
    // PhD - 3.8 (УГР,N,M)
    MatchMoveBorderL(usize),

    // Moves the right border to a corresponding projection.
    // Preprint - 3.8 SB(N, M)
    // PhD - 3.8 (УГР,N,M)
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
    TransplantObject(usize),
    TransplantExpr(usize),
    RewriteFinalize,

    // Completes execution of the current sentence.
    // PhD - 3.25. Preprint - 3.20.
    Return,
}
