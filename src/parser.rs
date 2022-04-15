use crate::data::Object::*;
use crate::data::{Condition, Error, Function, Object, RefalModule, Result, Sentence};
use tree_sitter::{Node, TreeCursor};

pub fn parse_input(text: &str) -> Result<RefalModule> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_refal::language()).unwrap();
    let tree = parser.parse(text, None).unwrap();
    if tree.root_node().has_error() {
        Err(Error::Parsing)
    } else {
        Ok(translate_module(&mut tree.walk(), text))
    }
}

fn translate_module(cursor: &mut TreeCursor, text: &str) -> RefalModule {
    let root_node = cursor.node();
    let module_node = root_node.child_by_field_id(MODULE).unwrap();
    let name = get_name(&module_node, text);
    let function_nodes: Vec<_> = root_node.children_by_field_id(FUNCTION, cursor).collect();
    let functions: Vec<_> = function_nodes
        .iter()
        .map(|n| translate_function(n, cursor, text))
        .collect();
    RefalModule { name, functions }
}

fn translate_function<'a>(node: &Node<'a>, cursor: &mut TreeCursor<'a>, text: &str) -> Function {
    let name = get_name(node, text);
    let sentence_nodes: Vec<_> = node.children_by_field_id(SENTENCE, cursor).collect();
    let sentences: Vec<_> = sentence_nodes
        .iter()
        .map(|n| translate_sentence(n, cursor, text))
        .collect();
    Function { name, sentences }
}

fn translate_sentence<'a>(node: &Node<'a>, cursor: &mut TreeCursor<'a>, text: &str) -> Sentence {
    let pattern: Vec<_> = node
        .children_by_field_id(PATTERN, cursor)
        .map(|n| translate_object(n, text))
        .collect();
    let condition_nodes: Vec<_> = node.children_by_field_id(CONDITIONS, cursor).collect();
    let conditions: Vec<_> = condition_nodes
        .iter()
        .map(|n| translate_condition(n, cursor, text))
        .collect();
    let rewrite: Vec<_> = node
        .children_by_field_id(REWRITE, cursor)
        .map(|n| translate_object(n, text))
        .collect();
    Sentence {
        pattern,
        conditions,
        rewrite,
    }
}

fn translate_condition<'a>(node: &Node<'a>, cursor: &mut TreeCursor<'a>, text: &str) -> Condition {
    let test: Vec<_> = node
        .children_by_field_id(TEST, cursor)
        .map(|n| translate_object(n, text))
        .collect();
    let pattern: Vec<_> = node
        .children_by_field_id(PATTERN, cursor)
        .map(|n| translate_object(n, text))
        .collect();
    Condition { test, pattern }
}

fn translate_object(node: tree_sitter::Node, text: &str) -> Object {
    match node.kind_id() {
        E_VAR => EVar(get_string(&node, text)),
        S_VAR => SVar(get_string(&node, text)),
        T_VAR => TVar(get_string(&node, text)),
        ID => Symbol(get_string(&node, text)),
        Q_SYMBOL => Symbol(get_string_stripped(&node, text)),
        STR_BR_L => StrBracketL,
        STR_BR_R => StrBracketR,
        FUN_BR_L => FunBracketL,
        FUN_BR_R => FunBracketR,
        _ => panic!(),
    }
}

fn get_name(node: &tree_sitter::Node, text: &str) -> String {
    let name_node = node.child_by_field_id(NAME).unwrap();
    text[name_node.byte_range()].to_string()
}

fn get_string(node: &tree_sitter::Node, text: &str) -> String {
    text[node.byte_range()].to_string()
}

fn get_string_stripped(node: &tree_sitter::Node, text: &str) -> String {
    let mut range = node.byte_range();
    range.start += 1;
    range.end -= 1;
    text[range].to_string()
}

const MODULE: u16 = 3;
const FUNCTION: u16 = 2;
const SENTENCE: u16 = 7;
const PATTERN: u16 = 5;
const REWRITE: u16 = 6;
const E_VAR: u16 = 13;
const S_VAR: u16 = 14;
const T_VAR: u16 = 15;
const ID: u16 = 16;
const Q_SYMBOL: u16 = 12;
const STR_BR_L: u16 = 8;
const STR_BR_R: u16 = 9;
const FUN_BR_L: u16 = 10;
const FUN_BR_R: u16 = 11;
const NAME: u16 = 4;
const CONDITIONS: u16 = 1;
const TEST: u16 = 8;

#[test]
fn test_mapping() {
    let language = tree_sitter_refal::language();
    assert_eq!(E_VAR, language.id_for_node_kind("e_var", true));
    assert_eq!(S_VAR, language.id_for_node_kind("s_var", true));
    assert_eq!(T_VAR, language.id_for_node_kind("t_var", true));
    assert_eq!(ID, language.id_for_node_kind("id", true));
    assert_eq!(Q_SYMBOL, language.id_for_node_kind("q_symbol", true));
    assert_eq!(STR_BR_L, language.id_for_node_kind("str_br_l", true));
    assert_eq!(STR_BR_R, language.id_for_node_kind("str_br_r", true));
    assert_eq!(FUN_BR_L, language.id_for_node_kind("fun_br_l", true));
    assert_eq!(FUN_BR_R, language.id_for_node_kind("fun_br_r", true));

    assert_eq!(MODULE, language.field_id_for_name("module").unwrap());
    assert_eq!(FUNCTION, language.field_id_for_name("function").unwrap());
    assert_eq!(SENTENCE, language.field_id_for_name("sentence").unwrap());
    assert_eq!(PATTERN, language.field_id_for_name("pattern").unwrap());
    assert_eq!(REWRITE, language.field_id_for_name("rewrite").unwrap());
    assert_eq!(NAME, language.field_id_for_name("name").unwrap());

    assert_eq!(
        CONDITIONS,
        language.field_id_for_name("conditions").unwrap()
    );
    assert_eq!(TEST, language.field_id_for_name("test").unwrap());
}
