use crate::compiler::compile_function;
use crate::data::Command;
use crate::parser;

#[cfg(test)]
fn check_compile(fun_input: &str, commands: Vec<Command>) {
    let mut input = String::from("$MODULE T;");
    input.push_str(fun_input);
    let mut module = parser::parse_input(&input).unwrap();
    let in_fun = module.functions.pop().unwrap();
    let compiled = compile_function(&String::from("T"), &in_fun);
    assert_eq!(compiled, commands)
}

#[test]
fn test01() {
    check_compile(
        "Empty { = ; }",
        vec![
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::RewriteFinalize,
            Command::MatchStart,
        ],
    )
}

#[test]
fn test02() {
    check_compile(
        "BracketsLeft { () = A ; }",
        vec![
            Command::MatchStrBracketL,
            Command::MatchEmpty,
            Command::MatchMoveBorderL(4),
            Command::MatchMoveBorderR(2),
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("A")),
            Command::RewriteFinalize,
            Command::MatchStart,
        ],
    )
}

#[test]
fn test60() {
    check_compile(
        "Clauses { A = B; B = C; C = A; }",
        vec![
            Command::SetupTransition(7),
            Command::MatchSymbolL(String::from("A")),
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("B")),
            Command::RewriteFinalize,
            Command::MatchStart,
            Command::SetupTransition(14),
            Command::MatchSymbolL(String::from("B")),
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("C")),
            Command::RewriteFinalize,
            Command::MatchStart,
            Command::MatchSymbolL(String::from("C")),
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("A")),
            Command::RewriteFinalize,
            Command::MatchStart,
        ],
    )
}

#[test]
fn test_palindrome() {
    check_compile(
        "P { = T; $s.1 = T; $s.1 $e.1 $s.1 = <P $e.1>; $e.1 = F; }",
        vec![
            Command::SetupTransition(6),
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("T")),
            Command::RewriteFinalize,
            Command::MatchStart,
            Command::SetupTransition(13),
            Command::MatchSVarL,
            Command::MatchEmpty,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("T")),
            Command::RewriteFinalize,
            Command::MatchStart,
            Command::SetupTransition(24),
            Command::MatchSVarL,
            Command::MatchSVarRProj(3),
            Command::MatchEVar,
            Command::RewriteStart,
            Command::InsertFunBracketL,
            Command::InsertSymbol(String::from("T.P")),
            Command::TransplantExpr(6),
            Command::InsertFunBracketR,
            Command::RewriteFinalize,
            Command::MatchStart,
            Command::MatchEVar,
            Command::RewriteStart,
            Command::InsertSymbol(String::from("F")),
            Command::RewriteFinalize,
            Command::MatchStart,
        ],
    )
}
