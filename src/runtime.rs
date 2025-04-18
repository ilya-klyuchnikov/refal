use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Symbol(String),
    StrBracketL,
    StrBracketR,
    FunBracketL,
    FunBracketR,
    First,
    Last,
}

impl Object {
    pub fn symbol(&self) -> Option<&String> {
        match self {
            Object::Symbol(s) => Some(s),
            _ => None,
        }
    }
}

// A node in a doubly linked list of Refal objects.
pub struct Node {
    // The Refal object contained in this node
    pub object: Object,
    // Previous node reference, None if this is the first node
    prev: RefCell<Option<Rc<Node>>>,
    // Next node reference, None if this is the last node
    next: RefCell<Option<Rc<Node>>>,
    // Reference to matching bracket node for StrBracketL/R and FunBracketL/R
    twin: RefCell<Option<Rc<Node>>>,
}

impl Node {
    pub fn new(object: Object) -> Self {
        Node {
            object,
            prev: RefCell::new(None),
            next: RefCell::new(None),
            twin: RefCell::new(None),
        }
    }
    #[inline(always)]
    pub fn next_opt(&self) -> Option<Rc<Node>> {
        self.next.borrow().as_ref().cloned()
    }
    #[inline(always)]
    pub fn next(&self) -> Rc<Node> {
        self.next.borrow().as_ref().unwrap().clone()
    }
    #[inline(always)]
    pub fn prev(&self) -> Rc<Node> {
        self.prev.borrow().as_ref().unwrap().clone()
    }
    #[inline(always)]
    pub fn twin(&self) -> Rc<Node> {
        self.twin.borrow().as_ref().unwrap().clone()
    }
}

pub fn flatten(first: Rc<Node>) -> Vec<Object> {
    let mut objects = Vec::<Object>::new();
    let mut cursor = first.clone();
    loop {
        match &cursor.object {
            Object::First | Object::Last => (),
            obj => objects.push(obj.clone()),
        }
        if let Some(next) = cursor.next_opt() {
            cursor = next;
        } else {
            break;
        }
    }
    objects
}

pub fn link_nodes(n1: &Rc<Node>, n2: &Rc<Node>) {
    *n1.next.borrow_mut() = Some(n2.clone());
    *n2.prev.borrow_mut() = Some(n1.clone());
}

pub fn unlink_prev(n: &Rc<Node>) {
    *n.prev.borrow_mut() = None;
}

pub fn unlink_next(n: &Rc<Node>) {
    *n.next.borrow_mut() = None;
}

fn unpair(n: &Rc<Node>) {
    *n.twin.borrow_mut() = None;
}

pub fn pair_nodes(n1: &Rc<Node>, n2: &Rc<Node>) {
    *n1.twin.borrow_mut() = Some(n2.clone());
    *n2.twin.borrow_mut() = Some(n1.clone());
}

pub fn free(start: Rc<Node>) {
    let mut cursor = start;
    loop {
        let next = cursor.next_opt();
        unlink_prev(&cursor);
        unlink_next(&cursor);
        unpair(&cursor);
        match next {
            None => break,
            Some(n) => cursor = n,
        }
    }
}
