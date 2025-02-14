use std::{collections::HashMap, rc::Rc, cell::RefCell};

#[derive(Debug, PartialEq, Clone)]
pub enum Module {
    Template(Template),
    Extension(Extension)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Template {
    pub name: String,
    pub content: Contents
}

#[derive(Debug, PartialEq, Clone)]
pub struct Extension {
    pub name: String,
    pub parent: String,
    pub blocks: HashMap<String, Box<Block>>
}

pub type Contents = Vec<Content>;

#[derive(Debug, PartialEq, Clone)]
pub enum Content {
    Text(String),
    Print(Expression),
    Block(Box<Block>),
    Statement(Stmt),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Atom(ExpressionAtom),
    Term(Term)
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionAtom {
    Str(String),
    Var(String),
    Number(i64),
    Float(f32),
    Array(Vec<ExpressionToken>),
    HashMap(Vec<KeyValuePair>),
    Parens(Vec<ExpressionToken>),
    Parent()
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionToken {
    Atom(ExpressionAtom),
    Op(OperatorToken)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Term {
    pub op: OperatorToken,
    pub params: Vec<Expression>
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperatorToken {
    Ternary,
    BAnd, BOr, BXor,
    Or, And,
    Eq, Neq, Starship,
    Lt, Gt, Gte, Lte,
    In, Matches, StartsWith,
    EndWith, Range, Add, Sub,
    StrConcat, Mul, Div, Divi,
    Modulo, Is, Exp, NullCoal,
    Filter, ArrayIndex, Get

}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyValuePair {
    pub key: ExpressionAtom,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub typ: BlockType,
    pub contents: Contents
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockType {
    BlockName(String),
    Loop(Loop),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub typ: IterationType,
    pub iterator: String
}

#[derive(Debug, PartialEq, Clone)]
pub enum IterationType {
    SingleVal(String),
    KeyVal((String, String))
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Set(Setter),
    Include(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Setter {
    pub target: String,
    pub value: Expression 
}

impl Template {
    pub fn replace_includes(mut self, replace: &mut dyn FnMut(Content)-> Content) -> Template {
        self.content = self.content.into_iter().map(|c| replace_includes(c, replace)).collect();
        self
    }
    
    pub fn into_block(self) -> Content {
        let Self { name, content } = self;
        Content::Block(Box::new(Block { typ: BlockType::BlockName(name), contents: content}))
    }

    pub fn apply_extensions(&mut self, mut extensions: HashMap<String, Box<Block>>) {
        extend_blocks(&mut self.content, &mut extensions);
    }
}

fn replace_includes(content: Content, replace: &mut dyn FnMut(Content) -> Content) -> Content {
    match content {
        Content::Statement(Stmt::Include(_)) => replace(content),
        Content::Block(mut block) => {
            block.contents = block.contents.into_iter().map(|c| replace_includes(c, replace)).collect();
            Content::Block(block)
        }
        _ => content
    }
}

fn extend_blocks(content: &mut Contents, extensions: &mut HashMap<String, Box<Block>>) {
    for elem in content.iter_mut() {
        if let Content::Block(base) = elem  {
            if let Some(child) = base.get_name().and_then(|name| extensions.remove(name)) {
                let parent = std::mem::replace(base, child);
                base.set_parents(parent)
            }
            extend_blocks(&mut base.contents, extensions);
        }
    }
}

impl Block {
    pub fn get_name(&self) -> Option<&str> {
        match &self.typ {
            BlockType::BlockName(name) => Some(name),
            _ => None,
        }
    } 

    pub fn set_parents(&mut self, parent: Box<Block>) {
        for elem in self.contents.iter_mut() {
            match elem {
                Content::Print(Expression::Atom(ExpressionAtom::Parent())) => *elem = Content::Block(parent.clone()),
                Content::Block(block) => block.set_parents(parent.clone()),
                _ => ()
            }
        }
    }
}

pub fn get_blocks(content: Contents, mut blocks: HashMap<String, Box<Block>>) -> HashMap<String, Box<Block>> {
    for elem in content.into_iter() {
        match elem {
            Content::Block(block) if block.get_name().is_some() => {blocks.insert(block.get_name().unwrap().to_string(), block);},
            _ => ()
        };
    }
    blocks
}
