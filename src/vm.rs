use crate::data::Command;
use crate::runtime::*;
use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

struct Jump {
    border1: Rc<Node>,
    border2: Rc<Node>,
    projection_index: usize,
    command_index: usize,
}

struct VM<'a> {
    defs: &'a HashMap<String, Vec<Command>>,
    commands: &'a Vec<Command>,
    command_index: usize,
    projections: Vec<Rc<Node>>,
    jumps: Vec<Jump>,
    border1: Rc<Node>,
    border2: Rc<Node>,
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
        let command = vm.commands[vm.command_index].clone();
        vm.command_index += 1;
        execute_cmd(&mut vm, &command);
    }
}

fn init_vm(defs: &HashMap<String, Vec<Command>>, mut dots: Vec<Rc<Node>>) -> VM {
    let fun_br_r = dots.pop().unwrap();
    let fun_br_l = fun_br_r.twin();
    let fun = fun_br_l.next();
    let fun_br_l_prev = fun_br_l.prev();

    let fun_sym = fun.object.symbol().unwrap();
    let commands = defs.get(fun_sym).unwrap();

    VM {
        command_index: 0,
        projections: vec![fun_br_l_prev, fun.clone(), fun_br_r.clone()],
        jumps: Vec::new(),
        border1: fun,
        border2: fun_br_r,
        dots,
        commands,
        end: false,
        defs,
    }
}

fn execute_cmd(vm: &mut VM, cmd: &Command) {
    match cmd {
        Command::MatchEmpty => vm.match_empty(),
        Command::MatchStrBracketL => vm.match_str_bracket_l(),
        Command::MatchStrBracketR => vm.match_str_bracket_r(),
        Command::MatchSymbolL(symbol) => vm.symbol_l(symbol),
        Command::MatchSymbolR(symbol) => vm.symbol_r(symbol),
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
        Command::MatchMoveBorders(l, r) => vm.match_move_borders(*l, *r),
        Command::MoveBorder => vm.move_border(),
        Command::NextStep => vm.next_step(),
        Command::SetupTransition(n) => vm.setup_transition(*n),
        Command::ConstrainLengthen(n) => vm.constrain_lengthen(*n),
        _ => panic!("illegal cmd: {:?}", cmd),
    }
}

// commands
impl VM<'_> {
    fn next_step(&mut self) {
        self.projections = vec![];
        self.jumps = vec![];

        if self.dots.is_empty() {
            self.end = true;
            return;
        }

        self.border2 = self.dots.pop().unwrap();
        self.border1 = self.border2.twin();

        let fun = self.border1.next();
        let fun_name = fun.object.symbol().unwrap();
        self.commands = self.defs.get(fun_name).unwrap();

        self.projections.push(self.border1.prev());
        self.projections.push(fun.clone());
        self.projections.push(self.border2.clone());

        self.border1 = fun;
        self.command_index = 0;
    }

    fn match_empty(&mut self) {
        let next = self.border1.next();
        if !ptr::eq(next.as_ref(), self.border2.as_ref()) {
            self.fail();
        }
    }

    fn symbol_l(&mut self, symbol: &str) {
        if self.shift_border1() {
            match &self.border1.object {
                Object::Symbol(s) if s == symbol => self.projections.push(self.border1.clone()),
                _ => self.fail(),
            }
        }
    }

    fn symbol_r(&mut self, symbol: &str) {
        if self.shift_border2() {
            match &self.border2.object {
                Object::Symbol(s) if s == symbol => self.projections.push(self.border2.clone()),
                _ => self.fail(),
            }
        }
    }

    fn match_str_bracket_l(&mut self) {
        if self.shift_border1() {
            match self.border1.object {
                Object::StrBracketL => {
                    self.border2 = self.border1.twin();
                    self.projections.push(self.border1.clone());
                    self.projections.push(self.border1.twin());
                }
                _ => self.fail(),
            }
        }
    }

    fn match_str_bracket_r(&mut self) {
        if self.shift_border2() {
            match self.border2.object {
                Object::StrBracketR => {
                    self.projections.push(self.border2.twin());
                    self.projections.push(self.border2.clone());
                    self.border2 = self.border2.twin();
                }
                _ => self.fail(),
            }
        }
    }

    fn match_s_var_l(&mut self) {
        if self.shift_border1() {
            if self.border1.object.symbol().is_none() {
                self.fail()
            } else {
                self.projections.push(self.border1.clone())
            }
        }
    }

    fn match_s_var_r(&mut self) {
        if self.shift_border2() {
            if self.border2.object.symbol().is_none() {
                self.fail()
            } else {
                self.projections.push(self.border2.clone())
            }
        }
    }

    fn match_s_var_l_proj(&mut self, n: usize) {
        if self.shift_border1() {
            let object = &self.projections.get(n).unwrap().object;
            if &self.border1.object != object {
                self.fail()
            } else {
                self.projections.push(self.border1.clone())
            }
        }
    }

    fn match_s_var_r_proj(&mut self, n: usize) {
        if self.shift_border2() {
            let object = &self.projections.get(n).unwrap().object;
            if &self.border2.object != object {
                self.fail()
            } else {
                self.projections.push(self.border2.clone())
            }
        }
    }

    fn match_t_var_l(&mut self) {
        if self.shift_border1() {
            self.projections.push(self.border1.clone());
            if self.border1.object == Object::StrBracketL {
                self.border1 = self.border1.twin();
            }
            self.projections.push(self.border1.clone());
        }
    }

    fn match_t_var_r(&mut self) {
        if self.shift_border2() {
            let to_insert = self.border2.clone();
            if self.border2.object == Object::StrBracketR {
                self.border2 = self.border2.twin()
            }
            self.projections.push(self.border2.clone());
            self.projections.push(to_insert);
        }
    }

    fn match_e_var(&mut self) {
        let start = &self.border1.next();
        let end = &self.border2.prev();
        self.projections.push(start.clone());
        self.projections.push(end.clone());
    }

    fn match_e_var_l_proj(&mut self, n: usize) {
        let node1 = self.projections.get(n - 1).unwrap().clone();
        let node2 = self.projections.get(n).unwrap().clone();
        let start = self.border1.next();
        let mut border0 = node1.prev();
        while !ptr::eq(border0.as_ref(), node2.as_ref()) {
            border0 = border0.next();
            if !self.shift_border1() {
                return;
            }
            if border0.object == self.border1.object {
                continue;
            }
            self.fail();
            return;
        }
        self.projections.push(start);
        self.projections.push(self.border1.clone());
    }

    fn match_e_var_r_proj(&mut self, n: usize) {
        let end = self.border2.prev();
        let node1 = self.projections.get(n - 1).unwrap().clone();
        let node2 = self.projections.get(n).unwrap().clone();
        let mut border0 = node2.next();
        while !ptr::eq(border0.as_ref(), node1.as_ref()) {
            border0 = border0.prev();
            if !self.shift_border2() {
                return;
            }
            if border0.object == self.border2.object {
                continue;
            }
            self.fail();
            return;
        }
        self.projections.push(self.border2.clone());
        self.projections.push(end);
    }

    fn match_move_borders(&mut self, l: usize, r: usize) {
        self.border1 = self.projections.get(l).unwrap().clone();
        self.border2 = self.projections.get(r).unwrap().clone();
    }

    fn prepare_lengthen(&mut self) {
        self.projections.push(self.border1.next());
        self.projections.push(self.border1.clone());
        self.jumps.push(Jump {
            border1: self.border1.clone(),
            border2: self.border2.clone(),
            projection_index: self.projections.len(),
            command_index: self.command_index,
        });
        self.command_index += 1;
    }

    fn lengthen(&mut self) {
        self.border1 = self.projections.pop().unwrap();
        if self.shift_border1() {
            if self.border1.object == Object::StrBracketL {
                self.border1 = self.border1.twin();
            }
            self.projections.push(self.border1.clone());
            self.jumps.push(Jump {
                border1: self.border1.clone(),
                border2: self.border2.clone(),
                projection_index: self.projections.len(),
                command_index: self.command_index - 1,
            });
        }
    }

    fn setup_transition(&mut self, command_index: usize) {
        self.jumps.push(Jump {
            border1: self.border1.clone(),
            border2: self.border2.clone(),
            projection_index: self.projections.len(),
            command_index,
        });
    }

    fn constrain_lengthen(&mut self, n: usize) {
        for _ in 0..n {
            self.jumps.pop();
        }
    }
}

impl VM<'_> {
    fn move_border(&mut self) {
        let mut border = self.projections.get(0).unwrap().clone();
        let mut local_dots: Vec<Rc<Node>> = vec![];
        let mut l_brackets: Vec<Rc<Node>> = vec![];
        let mut l_fun_brackets: Vec<Rc<Node>> = vec![];
        let mut transplants: Vec<(Rc<Node>, Rc<Node>, Rc<Node>)> = vec![];
        loop {
            let cmd = self.commands.get(self.command_index).unwrap();
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
                        self.projections.get(*n).unwrap().clone(),
                        self.projections.get(*n).unwrap().clone(),
                    ));
                }
                Command::TransplantExpr(n) => {
                    let start = self.projections.get(*n - 1).unwrap().clone();
                    let end = self.projections.get(*n).unwrap().clone();
                    if !ptr::eq(end.next().as_ref(), start.as_ref()) {
                        transplants.push((border.clone(), start, end));
                    }
                }
                Command::CopySymbol(n) => {
                    let node = self.projections.get(*n).unwrap();
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
                Command::CompleteStep => {
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
                }
                Command::NextStep => {
                    while let Some(dot) = local_dots.pop() {
                        self.dots.push(dot)
                    }
                    return;
                }
                _ => panic!("internal error"),
            }
            self.command_index += 1;
        }
    }
}

// utilities
impl VM<'_> {
    fn fail(&mut self) {
        match self.jumps.pop() {
            None => panic!("Recognition impossible"),
            Some(jump) => {
                self.border1 = jump.border1;
                self.border2 = jump.border2;
                self.projections.truncate(jump.projection_index);
                self.command_index = jump.command_index;
            }
        }
    }

    fn shift_border1(&mut self) -> bool {
        self.border1 = self.border1.next();
        if ptr::eq(self.border1.as_ref(), self.border2.as_ref()) {
            self.fail();
            false
        } else {
            true
        }
    }

    fn shift_border2(&mut self) -> bool {
        self.border2 = self.border2.prev();
        if ptr::eq(self.border1.as_ref(), self.border2.as_ref()) {
            self.fail();
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
static TEST_PROGRAM: &str = r#"
$MODULE Test;

Palindrome {
    = True;
    $s.1 = True;
    $s.1 $e.1 $s.1 = <Palindrome $e.1>;
    $e.1 = False;
}

ChangePlusToMinus {
    '+' $e.1 = '-' <ChangePlusToMinus $e.1>;
    $s.1 $e.1 = $s.1 <ChangePlusToMinus $e.1>;
    = ;
}

Translate {
    ($e.Word) $e.1 (($e.Word) $e.Trans) $e.2 = $e.Trans;
    ($e.Word) $e.1  =  '*' '*' '*';
}

Table {
 = 	(('c' 'a' 'n' 'e') 'd' 'o' 'g')
    (('g' 'a' 't' 't' 'o') 'c' 'a' 't')
    (('c' 'a' 'v' 'a' 'l' 'l' 'o') 'h' 'o' 'r' 's' 'e')
    (('r' 'a' 'n' 'a') 'f' 'r' 'o' 'g')
    (('p' 'o' 'r' 'c' 'o') 'p' 'i' 'g');
}

D1 {
 = <'Test.Translate' ('c' 'a' 'n' 'e') <'Test.Table'>>;
}

D2 {
 = <'Test.Translate' ('g' 'a' 't' 't' 'o') <'Test.Table'>>;
}

D3 {
 = <'Test.Translate' ('i' 'l' 'y' 'u' 's' 'h' 'k' 'i' 'n') <'Test.Table'>>;
}

BinaryAdd {
    ($e.X '0')($e.Y $s.1) = <BinaryAdd ($e.X)($e.Y)> $s.1;
    ($e.X '1')($e.Y '0') = <BinaryAdd ($e.X)($e.Y)> '1';
    ($e.X '1')($e.Y '1') = <BinaryAdd (<BinaryAdd ($e.X)('1')>)($e.Y)> '0';
    ($e.X)($e.Y) = $e.X $e.Y;
}

Blanks {
    $e.1 ' ' ' ' $e.2 = $e.1 <Blanks ' ' $e.2>;
    $e.1 = $e.1;
}

PreAlph {
    $s.1 $s.1 = True;
    $s.1 $s.2 = <Before $s.1 $s.2 In <Alphabet>>;
}

Before {
    $s.1 $s.2 In $e.A $s.1 $e.B $s.2 $e.C = True;
    $e.Z = False;
}

Alphabet {
     = 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z';
}

RecursiveAdd  {
    ($e.X) '0' = $e.X;
    ($e.X) $e.Y 's' = <RecursiveAdd ($e.X) $e.Y> 's';
}

EqualSymbols {
    $s.1 $s.1 = True;
    $s.1 $s.2 = False;
}

SortInsert {
    $e.1 = <Sort1 () $e.1>;
}

Sort1 {
    ($e.1) $t.1 $e.2 = <Sort1 (<Insert $e.1 $t.1>) $e.2>;
    ($e.1) = $e.1;
}

Insert {
    $e.1 $t.1 $t.2 = <Insert1 (<PreAlph $t.1 $t.2>) $e.1 $t.1 $t.2>;
    $e.1 = $e.1;
}

Insert1 {
    (True) $e.1 = $e.1;
    (False) $e.1 $t.1 $t.2 = <Insert $e.1 $t.2> $t.1;
}

SortMerge {
    $e.1 = <Check <Merge <Pairs $e.1>>>;
}

Pairs {
    $t.1 $t.2 $e.3 =<Pairs1 <PreAlph $t.1 $t.2> $t.1 $t.2 $e.3>;
    $t.1 = ($t.1);
     = ;
}

Pairs1 {
    True $t.1 $t.2 $e.3  = ($t.1 $t.2) <Pairs $e.3>;
    False $t.1 $t.2 $e.3 = ($t.2 $t.1) <Pairs $e.3>;
}

Merge {
     ($e.1) ($e.2) $e.Rest = (<Merge2 ($e.1)$e.2>) <Merge $e.Rest>;
     ($e.1) = ($e.1);
       = ;
}

/* merge two lists */
Merge2 {
    ($t.1 $e.X) $t.2 $e.Y = <Merge3 <PreAlph $t.1 $t.2> ($t.1 $e.X) $t.2 $e.Y>;
    ($e.1) $e.2 = $e.1 $e.2;  /* One of e1,e2 is empty */
}

Merge3 {
    True ($t.1 $e.X) $t.2 $e.Y = $t.1 <Merge2 ($e.X) $t.2 $e.Y>;
    False ($t.1 $e.X) $t.2 $e.Y = $t.2 <Merge2 ($t.1 $e.X) $e.Y>;
}

/* Check whether there is one list or more */
Check {
     = ;
    ($e.1) = $e.1;
    $e.1 = <Check <Merge $e.1>>;
}

MyCheck {
    $e.1 's' $e.1 's' $e.1= True;
    $e.1 = False;
}

BracketsRight{
    $e.1 ($e.2) = True;
    $e.1 = False;
}

IsA {
    'A' = T;
    $e.1 = F;
}

CountTerms {
     = '0';
    $t.1 $e.1 = <'Arithmetic.Add' '1' <CountTerms $e.1>>;
}

Permutations {
     = ();
    $s.1 $e.1 = <InsertAll $s.1 <Permutations $e.1>>;
}

InsertAll {
    $s.1 ($e.1) = <InsertPer () $s.1 ($e.1)>;
    $s.1 ($e.1) $e.2 = <InsertPer () $s.1 ($e.1)> <InsertAll $s.1 $e.2>;
}

InsertPer {
    ($e.Before) $s.1 () = ($e.Before $s.1);
    ($e.Before) $s.1 ($s.2 $e.After) = ($e.Before $s.1 $s.2 $e.After) <InsertPer ($e.Before $s.2) $s.1 ($e.After)>;
}

DeleteDuplicates {
    $e.1 ($e.A) $e.2 ($e.A) $e.3 = <DeleteDuplicates $e.1 ($e.A) $e.2 $e.3>;
    $e.1 = $e.1;
}

RepeatedSL {
     = True;
   $s.1 $s.1 $e.1 = <RepeatedSL $e.1>;
   $e.1 = False;
}

RepeatedSR {
     = True;
   $e.1 $s.1 $s.1 = <RepeatedSR $e.1>;
   $e.1 = False;
}

RepeatedTL {
     = True;
   $t.1 $t.1 $e.1 = <RepeatedTL $e.1>;
   $e.1 = False;
}

RepeatedTR {
     = True;
   $e.1 $t.1 $t.1 = <RepeatedTR $e.1>;
   $e.1 = False;
}

RepeatedEL {
     = True;
   $t.1 $e.1  $t.1 $e.1 $e.2 = <RepeatedEL $e.2>;
   $e.2 = False;
}

RepeatedER {
     = True;
   $e.1 $e.2 $e.2 = <RepeatedER $e.1>;
   $e.1 = False;
}

RepeatedER1 {
     = True;
   $e.1 $e.1 $e.2 $e.2 = True;
   $e.1 = False;
}

RepeatedInBrackets {
     = True;
   ($e.1 $e.1) ($e.2 $e.2) = True;
   $e.1 = False;
}

RepeatedER4 {
     = True;
   ($s.1 $e.1 $e.1) ($s.2 $e.2 $e.2) = True;
   $e.1 = False;
}

DoubleInBrackets {
   ($e.1) $e.2 = ($e.1 $e.1) <DoubleInBrackets $e.2>;
   $t.1 $e.2 = $t.1 <DoubleInBrackets $e.2>;
   =;
}

SymmetryE {
    = True;
   $t.1 $e.1 $e.2 $t.1 $e.1 = True;
   $e.2 = False;
}

SymbolR {
    $e.1 $s.1 = True;
    $e.1 = False;
}

TermR {
    $e.1 $t.1 = True;
    $e.1 = False;
}

Repeated {
    ($e.1 $s.1 $e.2)
    ($e.3 $s.1 $e.4)
    ($e.5 $s.2 $e.6)
    ($e.7 $s.2 $e.8)
    ($e.9 $s.3 $e.10)
    ($e.11 $s.3 $e.12)
        = $s.1 $s.2 $s.3;
    $e.1
        = N;
}

RemoveRepeated2 {
    $e.1 '|' $e.1 '|' $e.2 = $e.1 $e.1 $e.2;
    $e.1 = 'no_match';
}

RemoveRepeated3 {
    $e.1 '|' $e.1 '|' $e.1 '|' $e.2 = $e.1 $e.1 $e.1 <RemoveRepeated3 $e.2>;
    $e.1 = $e.1;
}

/* ----- integration tests ----- */

TestPalindrome1
{ = <'Test.Palindrome'>; }
TestPalindrome1Expected
{ = 'True'; }

TestPalindrome2
{ = <'Test.Palindrome' 'a'>; }
TestPalindrome2Expected
{ = 'True'; }

TestPalindrome3
{ = <'Test.Palindrome' 'a' 'a'>; }
TestPalindrome3Expected
{ = 'True'; }

TestPalindrome4
{ = <'Test.Palindrome' 'a' 'b'>; }
TestPalindrome4Expected
{ = 'False'; }

TestPalindrome5
{ = <'Test.Palindrome' ()>; }
TestPalindrome5Expected
{ = 'False'; }

TestChangePlusToMinus1
{ = <'Test.ChangePlusToMinus'>; }
TestChangePlusToMinus1Expected
{ = ; }

TestChangePlusToMinus2
{ = <'Test.ChangePlusToMinus' '-'>; }
TestChangePlusToMinus2Expected
{ = '-'; }

TestChangePlusToMinus3
{ = <'Test.ChangePlusToMinus' '+'>; }
TestChangePlusToMinus3Expected
{ = '-'; }

TestChangePlusToMinus4
{ = <'Test.ChangePlusToMinus' '+' '12' '-' '123'>; }
TestChangePlusToMinus4Expected
{ = '-' '12' '-' '123'; }

TestDictionary1
{ = <'Test.Translate' ('c' 'a' 'n' 'e')  <'Test.Table'>>; }
TestDictionary1Expected
{ = 'd' 'o' 'g'; }

TestDictionary2
{ = <'Test.Translate' ('g' 'a' 't' 't' 'o')  <'Test.Table'>>; }
TestDictionary2Expected
{ = 'c' 'a' 't'; }

TestD1
{ = <'Test.D1'>; }
TestD1Expected
{ = 'd' 'o' 'g'; }

TestD2
{ = <'Test.D2'>; }
TestD2Expected
{ = 'c' 'a' 't'; }

TestD3
{ = <'Test.D3'>; }
TestD3Expected
{ = '*' '*' '*'; }

TestBinaryAdd1
{ = <'Test.BinaryAdd' ('0') ('0')>; }
TestBinaryAdd1Expected
{ = '0'; }

TestBinaryAdd2
{ = <'Test.BinaryAdd' ('1') ('0')>; }
TestBinaryAdd2Expected
{ = '1'; }

TestBinaryAdd3
{ = <'Test.BinaryAdd' ('0') ('1')>; }
TestBinaryAdd3Expected
{ = '1'; }

TestBinaryAdd4
{ = <'Test.BinaryAdd' ('1') ('1')>; }
TestBinaryAdd4Expected
{ = '1' '0'; }

TestBinaryAdd5
{ = <'Test.BinaryAdd' ('1' '0') ('1')>; }
TestBinaryAdd5Expected
{ = '1' '1'; }

TestBinaryAdd6
{ = <'Test.BinaryAdd' ('1' '0') ('1' '0')>; }
TestBinaryAdd6Expected
{ = '1' '0' '0'; }

TestBlanks1
{ = <'Test.Blanks' ' '>; }
TestBlanks1Expected
{ = ' '; }

TestBlanks2
{ = <'Test.Blanks' ' ' ' '>; }
TestBlanks2Expected
{ = ' '; }

TestBlanks3
{ = <'Test.Blanks' ' ' ' ' '1' ' ' ' ' '2'>; }
TestBlanks3Expected
{ = ' ' '1' ' ' '2'; }

TestPreAlph1
{ = <'Test.PreAlph' 'a' 'a'>; }
TestPreAlph1Expected
{ = 'True'; }

TestPreAlph2
{ = <'Test.PreAlph' 'a' 'b'>; }
TestPreAlph2Expected
{ = 'True'; }

TestPreAlph3
{ = <'Test.PreAlph' 'a' 'z'>; }
TestPreAlph3Expected
{ = 'True'; }

TestPreAlph4
{ = <'Test.PreAlph' 'y' 'z'>; }
TestPreAlph4Expected
{ = 'True'; }

TestPreAlph5
{ = <'Test.PreAlph' 'z' 'a'>; }
TestPreAlph5Expected
{ = 'False'; }

TestRecursiveAdd1
{ = <'Test.RecursiveAdd' ('0') '0'>; }
TestRecursiveAdd1Expected
{ = '0'; }

TestRecursiveAdd2
{ = <'Test.RecursiveAdd' ('0' 's') '0'>; }
TestRecursiveAdd2Expected
{ = '0' 's'; }

TestRecursiveAdd3
{ = <'Test.RecursiveAdd' ('0') '0' 's'>; }
TestRecursiveAdd3Expected
{ = '0' 's'; }

TestRecursiveAdd4
{ = <'Test.RecursiveAdd' ('0' 's' 's' 's') '0' 's' 's' 's'>; }
TestRecursiveAdd4Expected
{ = '0' 's' 's' 's' 's' 's' 's'; }

TestRecursiveAdd5
{ = <'Test.RecursiveAdd' ('0') '0' 's' 's' 's' 's' 's' 's'>; }
TestRecursiveAdd5Expected
{ = '0' 's' 's' 's' 's' 's' 's'; }

TestRecursiveAdd6
{ = <'Test.RecursiveAdd' ('0' 's') '0' 's' 's' 's' 's' 's' 's'>; }
TestRecursiveAdd6Expected
{ = '0' 's' 's' 's' 's' 's' 's' 's'; }

TestSortInsert1
{ = <'Test.SortInsert' 'z' 'c'>; }
TestSortInsert1Expected
{ = 'c' 'z'; }

TestSortInsert2
{ = <'Test.SortInsert' 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'>; }
TestSortInsert2Expected
{ = 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'; }

TestSortInsert3
{ = <'Test.SortInsert' 'z' 'y' 'x' 'w' 'v' 'u' 't' 's' 'r' 'q' 'p' 'o' 'n' 'm' 'l' 'k' 'j' 'i' 'h' 'g' 'f' 'e' 'd' 'c' 'b' 'a'>; }
TestSortInsert3Expected
{ = 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'; }

TestSortMerge1
{ = <'Test.SortMerge' 'z' 'c'>; }
TestSortMerge1Expected
{ = 'c' 'z'; }

TestSortMerge2
{ = <'Test.SortMerge' 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'>; }
TestSortMerge2Expected
{ = 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'; }

TestSortMerge3
{ = <'Test.SortMerge' 'z' 'y' 'x' 'w' 'v' 'u' 't' 's' 'r' 'q' 'p' 'o' 'n' 'm' 'l' 'k' 'j' 'i' 'h' 'g' 'f' 'e' 'd' 'c' 'b' 'a'>; }
TestSortMerge3Expected
{ = 'a' 'b' 'c' 'd' 'e' 'f' 'g' 'h' 'i' 'j' 'k' 'l' 'm' 'n' 'o' 'p' 'q' 'r' 's' 't' 'u' 'v' 'w' 'x' 'y' 'z'; }

TestBracketsRight1
{ = <'Test.BracketsRight' >; }
TestBracketsRight1Expected
{ = 'False' ;}

TestBracketsRight2
{ = <'Test.BracketsRight' ()>; }
TestBracketsRight2Expected
{ = 'True' ;}

TestBracketsRight3
{ = <'Test.BracketsRight' ()() 'a'>; }
TestBracketsRight3Expected
{ = 'False' ;}

TestPermutations1
{ = <'Test.Permutations' >; }
TestPermutations1Expected
{ = ( ) ;}

TestPermutations2
{ = <'Test.Permutations' '1'>; }
TestPermutations2Expected
{ = ( '1' ) ;}

TestPermutations3
{ = <'Test.Permutations' '1' '2' '3'>; }
TestPermutations3Expected
{ = ( '1' '2' '3' ) ( '2' '1' '3' ) ( '2' '3' '1' ) ( '1' '3' '2' ) ( '3' '1' '2' ) ( '3' '2' '1' ) ;}

TestPermutations4
{ = <'Test.Permutations' '1' '2' '2'>; }
TestPermutations4Expected
{ = ( '1' '2' '2' ) ( '2' '1' '2' ) ( '2' '2' '1' ) ( '1' '2' '2' ) ( '2' '1' '2' ) ( '2' '2' '1' ) ;}

RepeatedSL1
{ = <'Test.RepeatedSL'>; }
RepeatedSL1Expected
{ = 'True' ;}

RepeatedSL2
{ = <'Test.RepeatedSL' 'a' 'a'>; }
RepeatedSL2Expected
{ = 'True' ;}

RepeatedSL3
{ = <'Test.RepeatedSL' 'a'>; }
RepeatedSL3Expected
{ = 'False' ;}

RepeatedSR1
{ = <'Test.RepeatedSR'>; }
RepeatedSR1Expected
{ = 'True' ;}

RepeatedSR2
{ = <'Test.RepeatedSR' 'a' 'a'>; }
RepeatedSR2Expected
{ = 'True' ;}

RepeatedSR3
{ = <'Test.RepeatedSR' 'a'>; }
RepeatedSR3Expected
{ = 'False' ;}

TestRepeatedTL1
{ = <'Test.RepeatedTL'>; }
TestRepeatedTL1Expected
{ = 'True' ;}

TestRepeatedTL2
{ = <'Test.RepeatedTL' 'a' 'a'>; }
TestRepeatedTL2Expected
{ = 'True' ;}

TestRepeatedTL3
{ = <'Test.RepeatedTL' 'a'>; }
TestRepeatedTL3Expected
{ = 'False' ;}

TestRepeatedTL4
{ = <'Test.RepeatedTL' 'a' 'b'>; }
TestRepeatedTL4Expected
{ = 'False' ;}

TestRepeatedTR1
{ = <'Test.RepeatedTR'>; }
TestRepeatedTR1Expected
{ = 'True' ;}

TestRepeatedTR2
{ = <'Test.RepeatedTR' 'a' 'a'>; }
TestRepeatedTR2Expected
{ = 'True' ;}

TestRepeatedTR3
{ = <'Test.RepeatedTR' 'a'>; }
TestRepeatedTR3Expected
{ = 'False' ;}

TestRepeatedTR4
{ = <'Test.RepeatedTR' 'a' 'b'>; }
TestRepeatedTR4Expected
{ = 'False' ;}

TestRepeatedEL1
{ = <'Test.RepeatedEL'>; }
TestRepeatedEL1Expected
{ = 'True' ;}

TestRepeatedEL2
{ = <'Test.RepeatedEL' 'a' 'a'>; }
TestRepeatedEL2Expected
{ = 'True' ;}

TestRepeatedEL3
{ = <'Test.RepeatedEL' 'a' 'b' 'a' 'b'>; }
TestRepeatedEL3Expected
{ = 'True' ;}

TestRepeatedEL4
{ = <'Test.RepeatedEL' 'a' 'b' 'c' 'a' 'b'>; }
TestRepeatedEL4Expected
{ = 'False' ;}

TestRepeatedEL5
{ = <'Test.RepeatedEL' () ()>; }
TestRepeatedEL5Expected
{ = 'True' ;}

TestRepeatedEL6
{ = <'Test.RepeatedEL' ('a') ('a')>; }
TestRepeatedEL6Expected
{ = 'True' ;}

TestRepeatedEL7
{ = <'Test.RepeatedEL' ('a') ('b')>; }
TestRepeatedEL7Expected
{ = 'False' ;}

TestRepeatedInBrackets1
{ = <'Test.RepeatedInBrackets'>; }
TestRepeatedInBrackets1Expected
{ = 'True' ;}

TestRepeatedInBrackets2
{ = <'Test.RepeatedInBrackets' () ()>; }
TestRepeatedInBrackets2Expected
{ = 'True' ;}

TestRepeatedInBrackets3
{ = <'Test.RepeatedInBrackets' ('a' 'a') ('b' 'b')>; }
TestRepeatedInBrackets3Expected
{ = 'True' ;}

TestRepeatedInBrackets4
{ = <'Test.RepeatedInBrackets' ('a' 'a') ('b' 'c')>; }
TestRepeatedInBrackets4Expected
{ = 'False' ;}

TestDoubleInBrackets1
{ = <'Test.DoubleInBrackets'>; }
TestDoubleInBrackets1Expected
{ = ;}

TestDoubleInBrackets2
{ = <'Test.DoubleInBrackets' (()) 'a'>; }
TestDoubleInBrackets2Expected
{ = ( ( ) ( ) ) 'a' ;}

TestSymmetryE1
{ = <'Test.SymmetryE'>; }
TestSymmetryE1Expected
{ = 'True' ;}

TestSymmetryE2
{ = <'Test.SymmetryE' 'a' 'a'>; }
TestSymmetryE2Expected
{ = 'True' ;}

TestSymmetryE3
{ = <'Test.SymmetryE' 'a' 'b' 'a'>; }
TestSymmetryE3Expected
{ = 'True' ;}

TestSymmetryE4
{ = <'Test.SymmetryE' 'a' () 'b' 'a' ()>; }
TestSymmetryE4Expected
{ = 'True' ;}

TestSymmetryE5
{ = <'Test.SymmetryE' 'a' () 'b' 'z'>; }
TestSymmetryE5Expected
{ = 'False' ;}

TestSymbolR1
{ = <'Test.SymbolR'>; }
TestSymbolR1Expected
{ = 'False' ;}

TestSymbolR2
{ = <'Test.SymbolR' ()>; }
TestSymbolR2Expected
{ = 'False' ;}

TestSymbolR3
{ = <'Test.SymbolR' () () 'a'>; }
TestSymbolR3Expected
{ = 'True' ;}

TestTermR1
{ = <'Test.TermR'>; }
TestTermR1Expected
{ = 'False' ;}

TestTermR2
{ = <'Test.TermR' ()>; }
TestTermR2Expected
{ = 'True' ;}

TestTermR3
{ = <'Test.TermR' () () 'a'>; }
TestTermR3Expected
{ = 'True' ;}

TestMu1
{ = <'Mu.Mu' 'Test.Palindrome' 'a'>; }
TestMu1Expected
{ = 'True' ;}

TestMu2
{ = <'Mu.Mu' 'Test.Palindrome' 'a' 'b'>; }
TestMu2Expected
{ = 'False' ;}

TestBuiltinPlus
{ = <'Builtin.+' '1' '2'>; }
TestBuiltinPlusExpected
{ = '3' ;}

TestBuiltinMinus
{ = <'Builtin.-' '10' '2'>; }
TestBuiltinMinusExpected
{ = '8' ;}

TestRepeated1
{ = <'Test.Repeated' () () () () () ()>; }
TestRepeated1Expected
{ = 'N' ;}

TestRepeated2
{ = <'Test.Repeated' ('a') ('a') ('b') ('b') ('c') ('c')>; }
TestRepeated2Expected
{ = 'a' 'b' 'c' ;}

TestRepeated3
{ = <'Test.Repeated' ('1' 'a') ('2' 'a') ('3' 'b') ('4' 'b') ('c' 'd') ('c' 'd')>; }
TestRepeated3Expected
{ = 'a' 'b' 'c' ;}

TestRemoveRepeated21
{ = <RemoveRepeated2 '|' '|' >;}
TestRemoveRepeated21Expected
{ = ;}

TestRemoveRepeated31
{ = <RemoveRepeated3 '|' '|' '|' >;}
TestRemoveRepeated31Expected
{ = ;}

TestRemoveRepeated32
{ = <RemoveRepeated3 'a' '|' 'a' '|' 'a' '|' >;}
TestRemoveRepeated32Expected
{ = 'a' 'a' 'a';}

TestRemoveRepeated33
{ = <RemoveRepeated3 'a' 'a' '|' 'a' 'a' '|' 'a' 'a' '|' 'a' 'a' '|' 'a' 'a' '|' 'a' 'a' >;}
TestRemoveRepeated33Expected
{ = 'a' 'a' 'a' 'a' 'a' 'a' 'a' 'a' '|' 'a' 'a' '|' 'a' 'a';}
"#;

#[cfg(test)]
fn test_example(goal1: &str, goal2: &str) {
    use crate::compiler::compile;
    let defs = compile(TEST_PROGRAM).unwrap();
    let out1 = eval_main(&defs, &String::from(goal1));
    let out2 = eval_main(&defs, &String::from(goal2));
    assert_eq!(out1, out2);
}

#[test]
fn test_palindrome1() {
    test_example("Test.TestPalindrome1", "Test.TestPalindrome1Expected")
}

#[test]
fn test_palindrome2() {
    test_example("Test.TestPalindrome2", "Test.TestPalindrome2Expected")
}

#[test]
fn test_palindrome3() {
    test_example("Test.TestPalindrome3", "Test.TestPalindrome3Expected")
}

#[test]
fn test_palindrome4() {
    test_example("Test.TestPalindrome4", "Test.TestPalindrome4Expected")
}

#[test]
fn test_palindrome5() {
    test_example("Test.TestPalindrome5", "Test.TestPalindrome5Expected")
}

#[test]
fn test_change_plus_to_minus1() {
    test_example(
        "Test.TestChangePlusToMinus1",
        "Test.TestChangePlusToMinus1Expected",
    )
}

#[test]
fn test_change_plus_to_minus2() {
    test_example(
        "Test.TestChangePlusToMinus2",
        "Test.TestChangePlusToMinus2Expected",
    )
}

#[test]
fn test_change_plus_to_minus3() {
    test_example(
        "Test.TestChangePlusToMinus3",
        "Test.TestChangePlusToMinus3Expected",
    )
}

#[test]
fn test_change_plus_to_minus4() {
    test_example(
        "Test.TestChangePlusToMinus4",
        "Test.TestChangePlusToMinus4Expected",
    )
}

#[test]
fn test_dictionary1() {
    test_example("Test.TestDictionary1", "Test.TestDictionary1Expected")
}

#[test]
fn test_dictionary2() {
    test_example("Test.TestDictionary2", "Test.TestDictionary2Expected")
}

#[test]
fn test_d1() {
    test_example("Test.TestD1", "Test.TestD1Expected")
}

#[test]
fn test_d2() {
    test_example("Test.TestD2", "Test.TestD2Expected")
}

#[test]
fn test_d3() {
    test_example("Test.TestD3", "Test.TestD3Expected")
}

#[test]
fn test_binary_add1() {
    test_example("Test.TestBinaryAdd1", "Test.TestBinaryAdd1Expected")
}

#[test]
fn test_binary_add2() {
    test_example("Test.TestBinaryAdd2", "Test.TestBinaryAdd2Expected")
}

#[test]
fn test_binary_add3() {
    test_example("Test.TestBinaryAdd3", "Test.TestBinaryAdd3Expected")
}

#[test]
fn test_binary_add4() {
    test_example("Test.TestBinaryAdd4", "Test.TestBinaryAdd4Expected")
}

#[test]
fn test_binary_add5() {
    test_example("Test.TestBinaryAdd5", "Test.TestBinaryAdd5Expected")
}

#[test]
fn test_binary_add6() {
    test_example("Test.TestBinaryAdd6", "Test.TestBinaryAdd6Expected")
}

#[test]
fn test_blanks1() {
    test_example("Test.TestBlanks1", "Test.TestBlanks1Expected")
}

#[test]
fn test_blanks2() {
    test_example("Test.TestBlanks2", "Test.TestBlanks2Expected")
}

#[test]
fn test_blanks3() {
    test_example("Test.TestBlanks3", "Test.TestBlanks3Expected")
}

#[test]
fn test_pre_alph1() {
    test_example("Test.TestPreAlph1", "Test.TestPreAlph1Expected")
}

#[test]
fn test_pre_alph2() {
    test_example("Test.TestPreAlph2", "Test.TestPreAlph2Expected")
}

#[test]
fn test_pre_alph3() {
    test_example("Test.TestPreAlph3", "Test.TestPreAlph3Expected")
}

#[test]
fn test_pre_alph4() {
    test_example("Test.TestPreAlph4", "Test.TestPreAlph4Expected")
}

#[test]
fn test_pre_alph5() {
    test_example("Test.TestPreAlph5", "Test.TestPreAlph5Expected")
}

#[test]
fn test_recursive_add1() {
    test_example("Test.TestRecursiveAdd1", "Test.TestRecursiveAdd1Expected")
}

#[test]
fn test_recursive_add2() {
    test_example("Test.TestRecursiveAdd2", "Test.TestRecursiveAdd2Expected")
}

#[test]
fn test_recursive_add3() {
    test_example("Test.TestRecursiveAdd3", "Test.TestRecursiveAdd3Expected")
}

#[test]
fn test_recursive_add4() {
    test_example("Test.TestRecursiveAdd4", "Test.TestRecursiveAdd4Expected")
}

#[test]
fn test_recursive_add5() {
    test_example("Test.TestRecursiveAdd5", "Test.TestRecursiveAdd5Expected")
}

#[test]
fn test_recursive_add6() {
    test_example("Test.TestRecursiveAdd6", "Test.TestRecursiveAdd6Expected")
}

#[test]
fn test_sort_insert1() {
    test_example("Test.TestSortInsert1", "Test.TestSortInsert1Expected")
}

#[test]
fn test_sort_insert2() {
    test_example("Test.TestSortInsert2", "Test.TestSortInsert2Expected")
}

#[test]
fn test_sort_insert3() {
    test_example("Test.TestSortInsert3", "Test.TestSortInsert3Expected")
}

#[test]
fn test_sort_merge1() {
    test_example("Test.TestSortMerge1", "Test.TestSortMerge1Expected")
}

#[test]
fn test_sort_merge2() {
    test_example("Test.TestSortMerge2", "Test.TestSortMerge2Expected")
}

#[test]
fn test_sort_merge3() {
    test_example("Test.TestSortMerge3", "Test.TestSortMerge3Expected")
}

#[test]
fn test_brackets_right1() {
    test_example("Test.TestBracketsRight1", "Test.TestBracketsRight1Expected")
}

#[test]
fn test_brackets_right2() {
    test_example("Test.TestBracketsRight2", "Test.TestBracketsRight2Expected")
}

#[test]
fn test_brackets_right3() {
    test_example("Test.TestBracketsRight3", "Test.TestBracketsRight3Expected")
}

#[test]
fn test_permutations1() {
    test_example("Test.TestPermutations1", "Test.TestPermutations1Expected")
}

#[test]
fn test_permutations2() {
    test_example("Test.TestPermutations2", "Test.TestPermutations2Expected")
}

#[test]
fn test_permutations3() {
    test_example("Test.TestPermutations3", "Test.TestPermutations3Expected")
}

#[test]
fn test_permutations4() {
    test_example("Test.TestPermutations4", "Test.TestPermutations4Expected")
}

#[test]
fn test_repeated_sl1() {
    test_example("Test.RepeatedSL1", "Test.RepeatedSL1Expected")
}

#[test]
fn test_repeated_sl2() {
    test_example("Test.RepeatedSL2", "Test.RepeatedSL2Expected")
}

#[test]
fn test_repeated_sl3() {
    test_example("Test.RepeatedSL3", "Test.RepeatedSL3Expected")
}

#[test]
fn test_repeated_sr1() {
    test_example("Test.RepeatedSR1", "Test.RepeatedSR1Expected")
}

#[test]
fn test_repeated_sr2() {
    test_example("Test.RepeatedSR2", "Test.RepeatedSR2Expected")
}

#[test]
fn test_repeated_sr3() {
    test_example("Test.RepeatedSR3", "Test.RepeatedSR3Expected")
}

#[test]
fn test_repeated_tl1() {
    test_example("Test.TestRepeatedTL1", "Test.TestRepeatedTL1Expected")
}

#[test]
fn test_repeated_tl2() {
    test_example("Test.TestRepeatedTL2", "Test.TestRepeatedTL2Expected")
}

#[test]
fn test_repeated_tl3() {
    test_example("Test.TestRepeatedTL3", "Test.TestRepeatedTL3Expected")
}

#[test]
fn test_repeated_tl4() {
    test_example("Test.TestRepeatedTL4", "Test.TestRepeatedTL4Expected")
}

#[test]
fn test_repeated_tr1() {
    test_example("Test.TestRepeatedTR1", "Test.TestRepeatedTR1Expected")
}

#[test]
fn test_repeated_tr2() {
    test_example("Test.TestRepeatedTR2", "Test.TestRepeatedTR2Expected")
}

#[test]
fn test_repeated_tr3() {
    test_example("Test.TestRepeatedTR3", "Test.TestRepeatedTR3Expected")
}

#[test]
fn test_repeated_tr4() {
    test_example("Test.TestRepeatedTR4", "Test.TestRepeatedTR4Expected")
}

#[test]
fn test_repeated_el1() {
    test_example("Test.TestRepeatedEL1", "Test.TestRepeatedEL1Expected")
}

#[test]
fn test_repeated_el2() {
    test_example("Test.TestRepeatedEL2", "Test.TestRepeatedEL2Expected")
}

#[test]
fn test_repeated_el3() {
    test_example("Test.TestRepeatedEL3", "Test.TestRepeatedEL3Expected")
}

#[test]
fn test_repeated_el4() {
    test_example("Test.TestRepeatedEL4", "Test.TestRepeatedEL4Expected")
}

#[test]
fn test_repeated_el5() {
    test_example("Test.TestRepeatedEL5", "Test.TestRepeatedEL5Expected")
}

#[test]
fn test_repeated_el6() {
    test_example("Test.TestRepeatedEL6", "Test.TestRepeatedEL6Expected")
}

#[test]
fn test_repeated_el7() {
    test_example("Test.TestRepeatedEL7", "Test.TestRepeatedEL7Expected")
}

#[test]
fn test_repeated_in_brackets1() {
    test_example(
        "Test.TestRepeatedInBrackets1",
        "Test.TestRepeatedInBrackets1Expected",
    )
}

#[test]
fn test_repeated_in_brackets2() {
    test_example(
        "Test.TestRepeatedInBrackets2",
        "Test.TestRepeatedInBrackets2Expected",
    )
}

#[test]
fn test_repeated_in_brackets3() {
    test_example(
        "Test.TestRepeatedInBrackets3",
        "Test.TestRepeatedInBrackets3Expected",
    )
}

#[test]
fn test_repeated_in_brackets4() {
    test_example(
        "Test.TestRepeatedInBrackets4",
        "Test.TestRepeatedInBrackets4Expected",
    )
}

#[test]
fn test_double_in_brackets1() {
    test_example(
        "Test.TestDoubleInBrackets1",
        "Test.TestDoubleInBrackets1Expected",
    )
}

#[test]
fn test_double_in_brackets2() {
    test_example(
        "Test.TestDoubleInBrackets2",
        "Test.TestDoubleInBrackets2Expected",
    )
}

#[test]
fn test_symmetry_e1() {
    test_example("Test.TestSymmetryE1", "Test.TestSymmetryE1Expected")
}

#[test]
fn test_symmetry_e2() {
    test_example("Test.TestSymmetryE2", "Test.TestSymmetryE2Expected")
}

#[test]
fn test_symmetry_e3() {
    test_example("Test.TestSymmetryE3", "Test.TestSymmetryE3Expected")
}

#[test]
fn test_symmetry_e4() {
    test_example("Test.TestSymmetryE4", "Test.TestSymmetryE4Expected")
}

#[test]
fn test_symmetry_e5() {
    test_example("Test.TestSymmetryE5", "Test.TestSymmetryE5Expected")
}

#[test]
fn test_symbol_r1() {
    test_example("Test.TestSymbolR1", "Test.TestSymbolR1Expected")
}

#[test]
fn test_symbol_r2() {
    test_example("Test.TestSymbolR2", "Test.TestSymbolR2Expected")
}

#[test]
fn test_symbol_r3() {
    test_example("Test.TestSymbolR3", "Test.TestSymbolR3Expected")
}

#[test]
fn test_term_r1() {
    test_example("Test.TestTermR1", "Test.TestTermR1Expected")
}

#[test]
fn test_term_r2() {
    test_example("Test.TestTermR2", "Test.TestTermR2Expected")
}

#[test]
fn test_term_r3() {
    test_example("Test.TestTermR3", "Test.TestTermR3Expected")
}

#[test]
fn test_remove_repeated_21() {
    test_example(
        "Test.TestRemoveRepeated21",
        "Test.TestRemoveRepeated21Expected",
    )
}

#[test]
fn test_remove_repeated_31() {
    test_example(
        "Test.TestRemoveRepeated31",
        "Test.TestRemoveRepeated31Expected",
    )
}

#[test]
fn test_remove_repeated_32() {
    test_example(
        "Test.TestRemoveRepeated32",
        "Test.TestRemoveRepeated32Expected",
    )
}

#[test]
fn test_remove_repeated_33() {
    test_example(
        "Test.TestRemoveRepeated33",
        "Test.TestRemoveRepeated33Expected",
    )
}
