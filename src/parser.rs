use crate::data::RefalObject::*;
use crate::data::{Error, Function, RefalModule, RefalObject, Result, Sentence};
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
    let module_node = root_node.child_by_field_name("module").unwrap();
    let name = get_name(&module_node, text);
    let function_nodes: Vec<_> = root_node
        .children_by_field_name("function", cursor)
        .collect();
    let functions: Vec<_> = function_nodes
        .iter()
        .map(|n| translate_function(n, cursor, text))
        .collect();
    RefalModule { name, functions }
}

fn translate_function<'a>(node: &Node<'a>, cursor: &mut TreeCursor<'a>, text: &str) -> Function {
    let name = get_name(node, text);
    let sentence_nodes: Vec<_> = node.children_by_field_name("sentence", cursor).collect();
    let sentences: Vec<_> = sentence_nodes
        .iter()
        .map(|n| translate_sentence(n, cursor, text))
        .collect();
    Function { name, sentences }
}

fn translate_sentence<'a>(node: &Node<'a>, cursor: &mut TreeCursor<'a>, text: &str) -> Sentence {
    let pattern: Vec<_> = node
        .children_by_field_name("pattern", cursor)
        .map(|n| translate_object(n, text))
        .collect();
    let rewrite: Vec<_> = node
        .children_by_field_name("rewrite", cursor)
        .map(|n| translate_object(n, text))
        .collect();
    Sentence { pattern, rewrite }
}

fn translate_object(node: tree_sitter::Node, text: &str) -> RefalObject {
    match node.kind() {
        "e_var" => EVar(get_string(&node, text)),
        "s_var" => SVar(get_string(&node, text)),
        "t_var" => TVar(get_string(&node, text)),
        "id" => Symbol(get_string(&node, text)),
        "q_symbol" => Symbol(get_string_stripped(&node, text)),
        "str_br_l" => StrBracketL,
        "str_br_r" => StrBracketR,
        "fun_br_l" => FunBracketL,
        "fun_br_r" => FunBracketR,
        _ => panic!(),
    }
}

fn get_name(node: &tree_sitter::Node, text: &str) -> String {
    let name_node = node.child_by_field_name("name").unwrap();
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
