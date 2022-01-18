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

pub struct Node {
    pub object: Object,
    prev: RefCell<Option<Rc<Node>>>,
    next: RefCell<Option<Rc<Node>>>,
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

    pub fn next(&self) -> Option<Rc<Node>> {
        self.next.borrow().as_ref().cloned()
    }

    pub fn prev(&self) -> Option<Rc<Node>> {
        self.prev.borrow().as_ref().cloned()
    }

    pub fn twin(&self) -> Option<Rc<Node>> {
        self.twin.borrow().as_ref().cloned()
    }
}

pub struct Chain {
    pub first: Rc<Node>,
    pub last: Rc<Node>,
}

pub fn flatten(chain: &Chain) -> Vec<Object> {
    let mut objects = Vec::<Object>::new();
    let mut cursor = chain.first.clone();
    loop {
        match &cursor.object {
            Object::First | Object::Last => (),
            obj => objects.push(obj.clone()),
        }
        if let Some(next) = cursor.next() {
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

pub fn pair_nodes(n1: &Rc<Node>, n2: &Rc<Node>) {
    *n1.twin.borrow_mut() = Some(n2.clone());
    *n2.twin.borrow_mut() = Some(n1.clone());
}

pub fn init_view(main: &str) -> (Vec<Rc<Node>>, Chain) {
    let first = Rc::new(Node::new(Object::First));
    let fun_br_l = Rc::new(Node::new(Object::FunBracketL));
    let fun = Rc::new(Node::new(Object::Symbol(String::from(main))));
    let fun_br_r = Rc::new(Node::new(Object::FunBracketR));
    let last = Rc::new(Node::new(Object::Last));

    link_nodes(&first, &fun_br_l);
    link_nodes(&fun_br_l, &fun);
    link_nodes(&fun, &fun_br_r);
    link_nodes(&fun_br_r, &last);
    pair_nodes(&fun_br_l, &fun_br_r);

    (vec![fun_br_r], Chain { first, last })
}
