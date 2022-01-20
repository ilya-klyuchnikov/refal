use crate::data::*;
use crate::{globalize, parser};
use std::collections::{HashMap, HashSet};

pub fn compile(input: &str) -> Result<RaslModule> {
    let module = parser::parse_input(input)?;
    let global_module = globalize::globalize_module(module);
    Ok(compile_module(&global_module))
}

pub fn compile_module(m: &RefalModule) -> RaslModule {
    let mut functions = Vec::<RaslFunction>::new();
    for f in &m.functions {
        functions.push(compile_function(f));
    }
    RaslModule {
        name: m.name.clone(),
        functions,
    }
}

fn compile_function(f: &Function) -> RaslFunction {
    let mut sentence_commands = Vec::<Vec<Command>>::new();
    for sentence in &f.sentences {
        sentence_commands.push(compile_sentence(sentence));
    }
    let commands = flatten(sentence_commands);
    RaslFunction {
        name: f.name.clone(),
        commands,
    }
}

fn compile_sentence(sentence: &Sentence) -> Vec<Command> {
    let mut commands = Vec::<Command>::new();
    let pattern: Vec<&RefalObject> = sentence.pattern.iter().collect();
    let expression: Vec<&RefalObject> = sentence.expression.iter().collect();
    let mut vars = vars(&pattern);
    let mut result = compile_pattern(&pattern);
    commands.append(&mut result.commands);
    commands.append(&mut compile_expression(
        &expression,
        &result.projected_vars,
        &mut vars,
    ));
    commands
}

fn flatten(mut sentence_commands: Vec<Vec<Command>>) -> Vec<Command> {
    let mut result = Vec::<Command>::new();
    let mut iter = sentence_commands.iter_mut().peekable();
    while let Some(cur_commands) = iter.next() {
        if iter.peek().is_some() {
            let jump_to = result.len() + cur_commands.len() + 1;
            result.push(Command::SetupTransition(jump_to));
        };
        result.append(cur_commands);
    }
    result
}

fn compile_pattern(pattern: &[&RefalObject]) -> PatternCompile {
    let mut state = State {
        border1: 1,
        border2: 2,
        next_element: 3,
        transition_depth: 0,
        projected_vars: HashMap::new(),
        holes_stack: Vec::new(),
        transition_depth_stack: Vec::new(),
        holes: vec![Hole {
            left_border: 1,
            right_border: 2,
            objects: pattern.to_vec(),
        }],
        commands: Vec::new(),
    };

    loop {
        let s = &mut state;
        match find_hole(s) {
            Some(index) => {
                move_borders(s, index);
                let _ = empty(s, index)
                    || closed_expression(s, index)
                    || brackets_left(s, index)
                    || symbol_left(s, index)
                    || term_left(s, index)
                    || brackets_right(s, index)
                    || symbol_right(s, index)
                    || term_right(s, index);
            }
            None => {
                if !s.holes.is_empty() {
                    handle_holes(s)
                } else if !s.holes_stack.is_empty() {
                    constrain_lengthen(s)
                } else {
                    break;
                }
            }
        }
    }

    PatternCompile {
        commands: state.commands,
        projected_vars: state.projected_vars,
    }
}

fn move_borders(state: &mut State, index: usize) {
    if let Some(hole) = state.holes.get(index) {
        if hole.left_border != state.border1 || hole.right_border != state.border2 {
            let cmd = Command::MatchMoveBorders(hole.left_border, hole.right_border);
            state.commands.push(cmd);
            state.border1 = hole.left_border;
            state.border2 = hole.right_border;
        }
    }
}

fn empty(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) if hole.objects.is_empty() => {
            state.commands.push(Command::MatchEmpty);
            state.holes.remove(index);
            true
        }
        _ => false,
    }
}

fn closed_expression(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) if hole.objects.len() == 1 => {
            let first_in_hole = hole.objects[0];
            if let RefalObject::EVar(v) = first_in_hole {
                if !state.projected_vars.contains_key(v) {
                    state.commands.push(Command::MatchEVar);
                    state
                        .projected_vars
                        .insert(v.clone(), state.next_element + 1);
                    state.holes.remove(index);
                    state.next_element += 2;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn brackets_left(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let first_in_hole = hole.objects[0];
            if let RefalObject::StrBracketL = first_in_hole {
                state.commands.push(Command::MatchStrBracketL);
                let mut l_br_amount: usize = 1;
                let mut right_br_index: usize = 0;

                for (i, obj) in hole.objects.iter().enumerate() {
                    if i == 0 {
                        continue;
                    } else if let RefalObject::StrBracketR = obj {
                        l_br_amount -= 1;
                        if l_br_amount == 0 {
                            right_br_index = i;
                            break;
                        }
                    } else if let RefalObject::StrBracketL = obj {
                        l_br_amount += 1;
                    }
                }
                let hole1 = Hole {
                    left_border: state.next_element,
                    right_border: state.next_element + 1,
                    objects: hole.objects[1..right_br_index].to_vec(),
                };
                let hole2 = Hole {
                    left_border: state.next_element + 1,
                    right_border: hole.right_border,
                    objects: hole.objects[right_br_index + 1..].to_vec(),
                };
                state.border1 = state.next_element;
                state.border2 = state.next_element + 1;
                state.holes.remove(index);
                state.holes.insert(index, hole2);
                state.holes.insert(index, hole1);
                state.next_element += 2;
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn symbol_left(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let first_in_hole = hole.objects[0];
            if let RefalObject::Symbol(s) = first_in_hole {
                state.commands.push(Command::MatchSymbolL(s.clone()));
                state.border1 = state.next_element;
                state.border2 = hole.right_border;
                // replace current hole
                state.holes[index] = Hole {
                    left_border: state.next_element,
                    right_border: hole.right_border,
                    objects: hole.objects[1..].to_vec(),
                };
                state.next_element += 1;
                true
            } else if let RefalObject::SVar(v) = first_in_hole {
                match state.projected_vars.get(v) {
                    None => {
                        state.commands.push(Command::MatchSVarL);
                        state.projected_vars.insert(v.clone(), state.next_element);
                    }
                    Some(i) => state.commands.push(Command::MatchSVarLProj(*i)),
                }
                state.border1 = state.next_element;
                state.border2 = hole.right_border;
                // replace current hole
                state.holes[index] = Hole {
                    left_border: state.next_element,
                    right_border: hole.right_border,
                    objects: hole.objects[1..].to_vec(),
                };
                state.next_element += 1;
                true
            } else {
                false
            }
        }
        None => false,
    }
}

fn term_left(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let first_in_hole = hole.objects[0];
            let this_match = if let RefalObject::TVar(v) = first_in_hole {
                match state.projected_vars.get(v) {
                    None => {
                        state.commands.push(Command::MatchTVarL);
                        state
                            .projected_vars
                            .insert(v.clone(), state.next_element + 1);
                    }
                    Some(i) => state.commands.push(Command::MatchEVarLProj(*i)),
                };
                true
            } else if let RefalObject::EVar(v) = first_in_hole {
                match state.projected_vars.get(v) {
                    Some(i) => {
                        state.commands.push(Command::MatchEVarLProj(*i));
                        true
                    }
                    None => false,
                }
            } else {
                false
            };
            if this_match {
                state.border1 = state.next_element + 1;
                state.border2 = hole.right_border;
                // replace current hole
                state.holes[index] = Hole {
                    left_border: state.next_element + 1,
                    right_border: hole.right_border,
                    objects: hole.objects[1..].to_vec(),
                };
                state.next_element += 2;
            }
            this_match
        }
        None => false,
    }
}

fn brackets_right(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let last_obj_index = hole.objects.len() - 1;
            let last = hole.objects[last_obj_index];
            if let RefalObject::StrBracketR = last {
                state.commands.push(Command::MatchStrBracketR);
                let mut l_br_index: usize = 0;
                let mut r_br_amount: usize = 1;
                for (i, obj) in hole.objects.iter().enumerate().rev() {
                    if i == last_obj_index {
                        continue;
                    } else if let RefalObject::StrBracketL = obj {
                        r_br_amount -= 1;
                        if r_br_amount == 0 {
                            l_br_index = i;
                            break;
                        }
                    } else if let RefalObject::StrBracketR = obj {
                        r_br_amount += 1;
                    }
                }
                let hole1 = Hole {
                    left_border: hole.left_border,
                    right_border: state.next_element,
                    objects: hole.objects[..l_br_index].to_vec(),
                };
                let hole2 = Hole {
                    left_border: state.next_element,
                    right_border: state.next_element + 1,
                    objects: hole.objects[l_br_index + 1..hole.objects.len() - 1].to_vec(),
                };
                state.border1 = hole.left_border;
                state.border2 = state.next_element;
                state.holes.remove(index);
                state.holes.insert(index, hole2);
                state.holes.insert(index, hole1);
                state.next_element += 2;
                true
            } else {
                false
            }
        }
        None => false,
    }
}

fn symbol_right(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let last_obj_index = hole.objects.len() - 1;
            let last = hole.objects[last_obj_index];
            let found = if let RefalObject::Symbol(s) = last {
                state.commands.push(Command::MatchSymbolR(s.clone()));
                true
            } else if let RefalObject::SVar(v) = last {
                match state.projected_vars.get(v) {
                    None => {
                        state.commands.push(Command::MatchSVarR);
                        state.projected_vars.insert(v.clone(), state.next_element);
                    }
                    Some(i) => state.commands.push(Command::MatchSVarRProj(*i)),
                }
                true
            } else {
                false
            };
            if found {
                state.border1 = hole.left_border;
                state.border2 = state.next_element;
                // replace current hole
                state.holes[index] = Hole {
                    left_border: hole.left_border,
                    right_border: state.next_element,
                    objects: hole.objects[0..hole.objects.len() - 1].to_vec(),
                };
                state.next_element += 1;
            }
            found
        }
        None => false,
    }
}

fn term_right(state: &mut State, index: usize) -> bool {
    match state.holes.get(index) {
        Some(hole) => {
            let last_obj_index = hole.objects.len() - 1;
            let last = hole.objects[last_obj_index];
            let this_match = if let RefalObject::TVar(v) = last {
                match state.projected_vars.get(v) {
                    None => {
                        state.commands.push(Command::MatchTVarR);
                        state
                            .projected_vars
                            .insert(v.clone(), state.next_element + 1);
                    }
                    Some(i) => {
                        state.commands.push(Command::MatchEVarRProj(*i));
                    }
                };
                true
            } else if let RefalObject::EVar(v) = last {
                match state.projected_vars.get(v) {
                    Some(i) => {
                        state.commands.push(Command::MatchEVarRProj(*i));
                        true
                    }
                    None => false,
                }
            } else {
                false
            };
            if this_match {
                state.border1 = hole.left_border;
                state.border2 = state.next_element;
                // replace current hole
                state.holes[index] = Hole {
                    left_border: hole.left_border,
                    right_border: state.next_element,
                    objects: hole.objects[0..hole.objects.len() - 1].to_vec(),
                };
                state.next_element += 2;
            }
            this_match
        }
        None => false,
    }
}

fn handle_holes(state: &mut State) {
    let projected_vars = state.projected_vars.keys().cloned().collect();
    let decomposition = decompose_holes(&state.holes, &projected_vars);
    // tricky manipulations with holes...
    if decomposition.n > 1 {
        // moving holes out into old_holes
        let mut old_holes = Vec::<Hole>::new();
        for i in (0..state.holes.len()).rev() {
            old_holes.push(state.holes.remove(i));
        }
        old_holes.reverse();
        // state.holes are empty now

        let mut holes_per_class = HashMap::<usize, Vec<Hole>>::new();
        for i in 1..decomposition.n + 1 {
            holes_per_class.insert(i, vec![]);
        }
        for (i, hole) in old_holes.into_iter().enumerate() {
            let class = decomposition.classes[i];
            if let Some(c_holes) = holes_per_class.get_mut(&class) {
                c_holes.push(hole);
            }
        }
        for i in 2..decomposition.n + 1 {
            if let Some(holes) = holes_per_class.remove(&i) {
                state.holes_stack.push(holes);
                state.transition_depth_stack.push(state.transition_depth);
            }
        }
        if let Some(holes0) = holes_per_class.remove(&1) {
            state.holes = holes0;
        }
    }
    if let Some(hole) = state.holes.first() {
        if let Some(RefalObject::EVar(v)) = hole.objects.first() {
            if state.border1 != hole.left_border || state.border2 != hole.right_border {
                state.commands.push(Command::MatchMoveBorders(
                    hole.left_border,
                    hole.right_border,
                ));
            };
            state.commands.push(Command::MatchEVarPrepare);
            state.commands.push(Command::MatchEVarLengthen);
            state.transition_depth += 1;
            state
                .projected_vars
                .insert(v.clone(), state.next_element + 1);
            state.border1 = state.next_element + 1;
            state.border2 = hole.right_border;
            state.holes[0] = Hole {
                left_border: state.next_element + 1,
                right_border: hole.right_border,
                objects: hole.objects[1..].to_vec(),
            };
            state.next_element += 2;
        }
    }
}

fn constrain_lengthen(state: &mut State) {
    if let (Some(td0), Some(holes)) = (state.transition_depth_stack.pop(), state.holes_stack.pop())
    {
        let cmd = Command::ConstrainLengthen(state.transition_depth - td0);
        state.commands.push(cmd);
        state.transition_depth = td0;
        state.holes = holes;
    }
}

fn compile_expression(
    expression: &[&RefalObject],
    projected_vars: &HashMap<String, usize>,
    vars: &mut HashSet<String>,
) -> Vec<Command> {
    let mut commands = vec![Command::MoveBorder];
    for x in expression {
        match x {
            RefalObject::Symbol(s) => commands.push(Command::InsertSymbol(s.clone())),
            RefalObject::StrBracketL => commands.push(Command::InsertStrBracketL),
            RefalObject::StrBracketR => commands.push(Command::InsertStrBracketR),
            RefalObject::FunBracketL => commands.push(Command::InsertFunBracketL),
            RefalObject::FunBracketR => commands.push(Command::InsertFunBracketR),
            RefalObject::SVar(v) => {
                if vars.remove(v) {
                    commands.push(Command::TransplantObject(*projected_vars.get(v).unwrap()))
                } else {
                    commands.push(Command::CopySymbol(*projected_vars.get(v).unwrap()))
                }
            }
            RefalObject::EVar(v) | RefalObject::TVar(v) => {
                if vars.remove(v) {
                    commands.push(Command::TransplantExpr(*projected_vars.get(v).unwrap()))
                } else {
                    commands.push(Command::CopyExpr(*projected_vars.get(v).unwrap()))
                }
            }
        }
    }
    commands.push(Command::CompleteStep);
    commands.push(Command::NextStep);
    commands
}

fn find_hole(state: &State) -> Option<usize> {
    for (i, hole) in state.holes.iter().enumerate() {
        if !non_trivial_hole(&hole.objects, &state.projected_vars) {
            return Some(i);
        }
    }
    None
}

fn decompose_holes(holes: &[Hole], projected_vars: &HashSet<String>) -> Decomposition {
    let mut n: usize = 0;
    let mut classes = vec![0; holes.len()];
    let mut x = HashSet::<String>::new();
    loop {
        let mut j: Option<usize> = None;
        let mut end = true;
        for i in 0..classes.len() {
            if classes[i] == 0 {
                end = false;
                x = vars(&holes[i].objects);
                n += 1;
                classes[i] = n;
                j = Some(i);
                break;
            }
        }
        if end {
            return Decomposition { n, classes };
        }
        loop {
            let mut to_k2 = true;
            for i in 0..holes.len() {
                if classes[i] == 0 {
                    let mut hole_vars = vars(&holes[i].objects);
                    hole_vars.retain(|e| x.contains(e));
                    if Some(i) != j && !projected_vars.is_superset(&hole_vars) {
                        to_k2 = false;
                        break;
                    }
                }
            }
            if to_k2 {
                break;
            }
            for i in 0..holes.len() {
                let hole_vars = vars(&holes[i].objects);
                if classes[i] == 0
                    && !projected_vars.is_superset(&x)
                    && !projected_vars.is_superset(&hole_vars)
                {
                    classes[i] = n;
                    for v in hole_vars {
                        x.insert(v);
                    }
                    break;
                }
            }
        }
    }
}

fn non_trivial_hole(objects: &[&RefalObject], projected_vars: &HashMap<String, usize>) -> bool {
    (objects.len() > 1)
        && (match objects.first() {
            Some(RefalObject::EVar(v)) => !projected_vars.contains_key(v),
            _ => false,
        })
        && (match objects.last() {
            Some(RefalObject::EVar(v)) => !projected_vars.contains_key(v),
            _ => false,
        })
}

fn vars(objects: &[&RefalObject]) -> HashSet<String> {
    let mut result = HashSet::<String>::new();
    for object in objects {
        match object {
            RefalObject::EVar(s) | RefalObject::SVar(s) | RefalObject::TVar(s) => {
                result.insert(s.clone());
            }
            _ => continue,
        }
    }
    result
}

struct State<'a> {
    border1: usize,
    border2: usize,
    next_element: usize,
    transition_depth: usize,
    projected_vars: HashMap<String, usize>,
    holes_stack: Vec<Vec<Hole<'a>>>,
    holes: Vec<Hole<'a>>,
    transition_depth_stack: Vec<usize>,
    commands: Vec<Command>,
}

struct PatternCompile {
    commands: Vec<Command>,
    projected_vars: HashMap<String, usize>,
}

struct Decomposition {
    n: usize,
    classes: Vec<usize>,
}

struct Hole<'a> {
    left_border: usize,
    right_border: usize,
    objects: Vec<&'a RefalObject>,
}

#[cfg(test)]
fn check_compile(fun_input: &str, commands: Vec<Command>) {
    let mut input = String::from("$MODULE T;");
    input.push_str(fun_input);
    let fun = compile(&input).unwrap().functions.pop();
    assert!(fun.is_some());
    assert_eq!(fun.unwrap().commands, commands)
}

#[test]
fn test01() {
    check_compile(
        "Empty { = ; }",
        vec![
            Command::MatchEmpty,
            Command::MoveBorder,
            Command::CompleteStep,
            Command::NextStep,
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
            Command::MatchMoveBorders(4, 2),
            Command::MatchEmpty,
            Command::MoveBorder,
            Command::InsertSymbol(String::from("A")),
            Command::CompleteStep,
            Command::NextStep,
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
            Command::MoveBorder,
            Command::InsertSymbol(String::from("B")),
            Command::CompleteStep,
            Command::NextStep,
            Command::SetupTransition(14),
            Command::MatchSymbolL(String::from("B")),
            Command::MatchEmpty,
            Command::MoveBorder,
            Command::InsertSymbol(String::from("C")),
            Command::CompleteStep,
            Command::NextStep,
            Command::MatchSymbolL(String::from("C")),
            Command::MatchEmpty,
            Command::MoveBorder,
            Command::InsertSymbol(String::from("A")),
            Command::CompleteStep,
            Command::NextStep,
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
            Command::MoveBorder,
            Command::InsertSymbol(String::from("T")),
            Command::CompleteStep,
            Command::NextStep,
            Command::SetupTransition(13),
            Command::MatchSVarL,
            Command::MatchEmpty,
            Command::MoveBorder,
            Command::InsertSymbol(String::from("T")),
            Command::CompleteStep,
            Command::NextStep,
            Command::SetupTransition(24),
            Command::MatchSVarL,
            Command::MatchSVarRProj(3),
            Command::MatchEVar,
            Command::MoveBorder,
            Command::InsertFunBracketL,
            Command::InsertSymbol(String::from("T.P")),
            Command::TransplantExpr(6),
            Command::InsertFunBracketR,
            Command::CompleteStep,
            Command::NextStep,
            Command::MatchEVar,
            Command::MoveBorder,
            Command::InsertSymbol(String::from("F")),
            Command::CompleteStep,
            Command::NextStep,
        ],
    )
}
