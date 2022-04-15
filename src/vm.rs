#[cfg(test)]
mod tests;

use crate::data::Command;
use crate::runtime::*;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

struct Jump {
    border_l: Rc<Node>,
    border_r: Rc<Node>,
    projection_index: usize,
    command_index: usize,
}

struct VM<'a> {
    defs: &'a HashMap<String, Vec<Command>>,
    commands: &'a Vec<Command>,
    command_index: usize,
    projections: Vec<Rc<Node>>,
    jumps: Vec<Jump>,
    border_l: Rc<Node>,
    border_r: Rc<Node>,
    dots: Vec<Rc<Node>>,
    end: bool,
}

pub fn eval_main(defs: &HashMap<String, Vec<Command>>, main: &str) -> Vec<Object> {
    let (dots, chain) = init_view(main);
    eval(defs, dots);
    flatten(&chain)
}

fn eval(defs: &HashMap<String, Vec<Command>>, dots: Vec<Rc<Node>>) {
    let mut vm = init_vm(defs, dots);
    while !vm.end {
        let cmd = &vm.commands[vm.command_index];
        vm.command_index += 1;
        execute_cmd(&mut vm, cmd);
    }
}

fn init_vm(defs: &HashMap<String, Vec<Command>>, mut dots: Vec<Rc<Node>>) -> VM {
    let fun_br_r = dots.pop().unwrap();
    let fun_br_l = fun_br_r.twin();
    let fun = fun_br_l.next();
    let fun_br_l_prev = fun_br_l.prev();

    let fun_sym = fun.object.symbol().unwrap();
    let commands = defs.get(fun_sym).expect(fun_sym);

    VM {
        command_index: 0,
        projections: vec![fun_br_l_prev, fun.clone(), fun_br_r.clone()],
        jumps: Vec::new(),
        border_l: fun,
        border_r: fun_br_r,
        dots,
        commands,
        end: false,
        defs,
    }
}

fn execute_cmd(vm: &mut VM, cmd: &Command) {
    match cmd {
        Command::MatchStart => vm.match_start(),
        Command::MatchEmpty => vm.match_empty(),
        Command::MatchStrBracketL => vm.match_str_bracket_l(),
        Command::MatchStrBracketR => vm.match_str_bracket_r(),
        Command::MatchSymbolL(symbol) => vm.match_symbol_l(symbol),
        Command::MatchSymbolR(symbol) => vm.match_symbol_r(symbol),
        Command::MatchSVarL => vm.match_s_var_l(),
        Command::MatchSVarR => vm.match_s_var_r(),
        Command::MatchSVarLProj(n) => vm.match_s_var_l_proj(*n),
        Command::MatchSVarRProj(n) => vm.match_s_var_r_proj(*n),
        Command::MatchTVarL => vm.match_t_var_l(),
        Command::MatchTVarR => vm.match_t_var_r(),
        Command::MatchEVarPrepare => vm.prepare_lengthen(),
        Command::MatchEVar => vm.match_e_var(),
        Command::MatchEVarLengthen => vm.lengthen(),
        Command::MatchEVarLProj(n) => vm.match_e_var_l_proj(*n),
        Command::MatchEVarRProj(n) => vm.match_e_var_r_proj(*n),
        Command::MatchMoveBorderL(n) => vm.match_move_border_l(*n),
        Command::MatchMoveBorderR(n) => vm.match_move_border_r(*n),
        Command::SetupTransition(n) => vm.setup_transition(*n),
        Command::ConstrainLengthen(n) => vm.constrain_lengthen(*n),
        Command::RewriteStart => vm.rewrite_start(),
        _ => panic!("illegal cmd: {:?}", cmd),
    }
}

impl VM<'_> {
    fn match_start(&mut self) {
        if self.dots.is_empty() {
            self.end = true;
            return;
        }

        self.border_r = self.dots.pop().unwrap();
        self.border_l = self.border_r.twin();

        let fun = self.border_l.next();
        let fun_name = fun.object.symbol().unwrap();
        self.commands = self.defs.get(fun_name).unwrap();

        self.projections.push(self.border_l.prev());
        self.projections.push(fun.clone());
        self.projections.push(self.border_r.clone());

        self.border_l = fun;
        self.command_index = 0;
    }

    fn match_empty(&mut self) {
        let next = self.border_l.next();
        if !ptr::eq(next.as_ref(), self.border_r.as_ref()) {
            self.fail();
        }
    }

    fn match_symbol_l(&mut self, symbol: &str) {
        if self.shift_border1() {
            match &self.border_l.object {
                Object::Symbol(s) if s == symbol => self.projections.push(self.border_l.clone()),
                _ => self.fail(),
            }
        }
    }

    fn match_symbol_r(&mut self, symbol: &str) {
        if self.shift_border2() {
            match &self.border_r.object {
                Object::Symbol(s) if s == symbol => self.projections.push(self.border_r.clone()),
                _ => self.fail(),
            }
        }
    }

    fn match_str_bracket_l(&mut self) {
        if self.shift_border1() {
            match self.border_l.object {
                Object::StrBracketL => {
                    self.border_r = self.border_l.twin();
                    self.projections.push(self.border_l.clone());
                    self.projections.push(self.border_l.twin());
                }
                _ => self.fail(),
            }
        }
    }

    fn match_str_bracket_r(&mut self) {
        if self.shift_border2() {
            match self.border_r.object {
                Object::StrBracketR => {
                    self.projections.push(self.border_r.twin());
                    self.projections.push(self.border_r.clone());
                    self.border_r = self.border_r.twin();
                }
                _ => self.fail(),
            }
        }
    }

    fn match_s_var_l(&mut self) {
        if self.shift_border1() {
            if self.border_l.object.symbol().is_none() {
                self.fail()
            } else {
                self.projections.push(self.border_l.clone())
            }
        }
    }

    fn match_s_var_r(&mut self) {
        if self.shift_border2() {
            if self.border_r.object.symbol().is_none() {
                self.fail()
            } else {
                self.projections.push(self.border_r.clone())
            }
        }
    }

    fn match_s_var_l_proj(&mut self, n: usize) {
        if self.shift_border1() {
            let object = &self.projections[n].object;
            if &self.border_l.object != object {
                self.fail()
            } else {
                self.projections.push(self.border_l.clone())
            }
        }
    }

    fn match_s_var_r_proj(&mut self, n: usize) {
        if self.shift_border2() {
            let object = &self.projections[n].object;
            if &self.border_r.object != object {
                self.fail()
            } else {
                self.projections.push(self.border_r.clone())
            }
        }
    }

    fn match_t_var_l(&mut self) {
        if self.shift_border1() {
            self.projections.push(self.border_l.clone());
            if self.border_l.object == Object::StrBracketL {
                self.border_l = self.border_l.twin();
            }
            self.projections.push(self.border_l.clone());
        }
    }

    fn match_t_var_r(&mut self) {
        if self.shift_border2() {
            let to_insert = self.border_r.clone();
            if self.border_r.object == Object::StrBracketR {
                self.border_r = self.border_r.twin()
            }
            self.projections.push(self.border_r.clone());
            self.projections.push(to_insert);
        }
    }

    fn match_e_var(&mut self) {
        let start = &self.border_l.next();
        let end = &self.border_r.prev();
        self.projections.push(start.clone());
        self.projections.push(end.clone());
    }

    fn match_e_var_l_proj(&mut self, n: usize) {
        let border1 = self.projections[n - 1].clone();
        let border2 = self.projections[n].clone();
        let start = self.border_l.next();
        let mut cursor = border1.prev();
        while !ptr::eq(cursor.as_ref(), border2.as_ref()) {
            cursor = cursor.next();
            if !self.shift_border1() {
                return;
            }
            if cursor.object == self.border_l.object {
                continue;
            }
            self.fail();
            return;
        }
        self.projections.push(start);
        self.projections.push(self.border_l.clone());
    }

    fn match_e_var_r_proj(&mut self, n: usize) {
        let border1 = self.projections[n - 1].clone();
        let border2 = self.projections[n].clone();
        let end = self.border_r.prev();
        let mut cursor = border2.next();
        while !ptr::eq(cursor.as_ref(), border1.as_ref()) {
            cursor = cursor.prev();
            if !self.shift_border2() {
                return;
            }
            if cursor.object == self.border_r.object {
                continue;
            }
            self.fail();
            return;
        }
        self.projections.push(self.border_r.clone());
        self.projections.push(end);
    }

    fn match_move_border_l(&mut self, n: usize) {
        self.border_l = self.projections[n].clone();
    }

    fn match_move_border_r(&mut self, n: usize) {
        self.border_r = self.projections[n].clone();
    }

    fn prepare_lengthen(&mut self) {
        self.projections.push(self.border_l.next());
        self.projections.push(self.border_l.clone());
        self.jumps.push(Jump {
            border_l: self.border_l.clone(),
            border_r: self.border_r.clone(),
            projection_index: self.projections.len(),
            command_index: self.command_index,
        });
        self.command_index += 1;
    }

    fn lengthen(&mut self) {
        self.border_l = self.projections.pop().unwrap();
        if self.shift_border1() {
            if self.border_l.object == Object::StrBracketL {
                self.border_l = self.border_l.twin();
            }
            self.projections.push(self.border_l.clone());
            self.jumps.push(Jump {
                border_l: self.border_l.clone(),
                border_r: self.border_r.clone(),
                projection_index: self.projections.len(),
                command_index: self.command_index - 1,
            });
        }
    }

    fn setup_transition(&mut self, command_index: usize) {
        self.jumps.push(Jump {
            border_l: self.border_l.clone(),
            border_r: self.border_r.clone(),
            projection_index: self.projections.len(),
            command_index,
        });
    }

    fn constrain_lengthen(&mut self, n: usize) {
        for _ in 0..n {
            self.jumps.pop();
        }
    }

    fn rewrite_start(&mut self) {
        self.jumps.clear();
        let mut border = self.projections[0].clone();
        let mut local_dots: Vec<Rc<Node>> = vec![];
        let mut l_brackets: Vec<Rc<Node>> = vec![];
        let mut l_fun_brackets: Vec<Rc<Node>> = vec![];
        let mut transplants: Vec<(Rc<Node>, Rc<Node>, Rc<Node>)> = vec![];
        loop {
            let cmd = &self.commands[self.command_index];
            self.command_index += 1;
            match cmd {
                Command::InsertSymbol(s) => {
                    let symbol = Rc::new(Node::new(Object::Symbol(s.clone())));
                    let next = border.next();
                    link_nodes(&border, &symbol);
                    border = symbol;
                    link_nodes(&border, &next);
                }
                Command::InsertStrBracketL => {
                    let bracket_l = Rc::new(Node::new(Object::StrBracketL));
                    l_brackets.push(bracket_l.clone());
                    let next = border.next();
                    link_nodes(&border, &bracket_l);
                    border = bracket_l;
                    link_nodes(&border, &next);
                }
                Command::InsertStrBracketR => {
                    let bracket_r = Rc::new(Node::new(Object::StrBracketR));
                    let bracket_l = l_brackets.pop().unwrap();
                    pair_nodes(&bracket_l, &bracket_r);
                    let next = border.next();
                    link_nodes(&border, &bracket_r);
                    border = bracket_r;
                    link_nodes(&border, &next);
                }
                Command::InsertFunBracketL => {
                    let bracket_l = Rc::new(Node::new(Object::FunBracketL));
                    l_fun_brackets.push(bracket_l.clone());
                    let next = border.next();
                    link_nodes(&border, &bracket_l);
                    border = bracket_l;
                    link_nodes(&border, &next);
                }
                Command::InsertFunBracketR => {
                    let bracket_r = Rc::new(Node::new(Object::FunBracketR));
                    let bracket_l = l_fun_brackets.pop().unwrap();
                    pair_nodes(&bracket_l, &bracket_r);
                    let next = border.next();
                    link_nodes(&border, &bracket_r);
                    border = bracket_r;
                    link_nodes(&border, &next);
                    local_dots.push(border.clone());
                }
                Command::TransplantObject(n) => {
                    transplants.push((
                        border.clone(),
                        self.projections[*n].clone(),
                        self.projections[*n].clone(),
                    ));
                }
                Command::TransplantExpr(n) => {
                    let start = self.projections[*n - 1].clone();
                    let end = self.projections[*n].clone();
                    if !ptr::eq(end.next().as_ref(), start.as_ref()) {
                        transplants.push((border.clone(), start, end));
                    }
                }
                Command::CopySymbol(n) => {
                    let node = &self.projections[*n];
                    let s = node.object.symbol().unwrap().to_string();
                    let symbol = Rc::new(Node::new(Object::Symbol(s)));
                    let current_next = border.next();
                    link_nodes(&border, &symbol);
                    border = symbol;
                    link_nodes(&border, &current_next);
                }
                Command::CopyExpr(n) => {
                    let node1 = &self.projections[*n - 1];
                    let node2 = &self.projections[*n];
                    let mut current_node: Rc<Node> = node1.prev();
                    let next = &border.next();
                    while !ptr::eq(current_node.as_ref(), node2.as_ref()) {
                        current_node = current_node.next();
                        match &current_node.object {
                            Object::StrBracketL => {
                                let current_to_insert = Rc::new(Node::new(Object::StrBracketL));
                                l_brackets.push(current_to_insert.clone());
                                link_nodes(&border, &current_to_insert);
                                border = current_to_insert;
                            }
                            Object::StrBracketR => {
                                let current_to_insert = Rc::new(Node::new(Object::StrBracketR));
                                let prev_l_bracket = l_brackets.pop().unwrap();
                                pair_nodes(&prev_l_bracket, &current_to_insert);
                                link_nodes(&border, &current_to_insert);
                                border = current_to_insert;
                            }
                            object => {
                                let current_to_insert = Rc::new(Node::new(object.clone()));
                                link_nodes(&border, &current_to_insert);
                                border = current_to_insert;
                            }
                        }
                    }
                    link_nodes(&border, next);
                }
                Command::RewriteFinalize => {
                    let node = &self.projections[2];
                    let garbage = if !ptr::eq(border.as_ref(), node.as_ref()) {
                        let next = &node.next();
                        let first_to_delete = border.next();
                        let last_to_delete = next.prev();
                        link_nodes(&border, next);
                        unlink_next(&last_to_delete);
                        unlink_prev(&first_to_delete);
                        Some(first_to_delete)
                    } else {
                        None
                    };
                    while let Some(transplant) = transplants.pop() {
                        let (border, start, end) = transplant;
                        link_nodes(&start.prev(), &end.next());
                        link_nodes(&end, &border.next());
                        link_nodes(&border, &start);
                    }
                    if let Some(start) = garbage {
                        free(start);
                    }
                    while let Some(dot) = local_dots.pop() {
                        self.dots.push(dot)
                    }
                    self.projections.clear();
                    return;
                }
                _ => panic!("internal error"),
            }
        }
    }

    fn fail(&mut self) {
        match self.jumps.pop() {
            None => panic!("Recognition impossible"),
            Some(jump) => {
                self.border_l = jump.border_l;
                self.border_r = jump.border_r;
                self.projections.truncate(jump.projection_index);
                self.command_index = jump.command_index;
            }
        }
    }

    fn shift_border1(&mut self) -> bool {
        self.border_l = self.border_l.next();
        if ptr::eq(self.border_l.as_ref(), self.border_r.as_ref()) {
            self.fail();
            false
        } else {
            true
        }
    }

    fn shift_border2(&mut self) -> bool {
        self.border_r = self.border_r.prev();
        if ptr::eq(self.border_l.as_ref(), self.border_r.as_ref()) {
            self.fail();
            false
        } else {
            true
        }
    }
}
