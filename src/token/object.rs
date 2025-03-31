use std::{fmt::Display, i64, collections::HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use crate::token::evaluator::EvalError;

use super::ast::{Parameters, BlockStatement};

pub trait ObjectType {
    fn inspect(&self) -> String;
}
#[derive(PartialEq, Clone, Debug)]
pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Return(Return),
    String(String),
    Error(String),
    Function(Function),
    Null,
}
impl Object {
    pub fn obj_type(&self) -> String {
        match self {
            Object::Integer(_) => return "Integer".to_string(),
            Object::Boolean(_) => return "Bool".to_string(),
            Object::Return(_) => return "Return".to_string(),
            Object::Error(_) => return "Error".to_string(),
            Object::Function(_) => return "Function".to_string(),
            Object::String(_) => return "String".to_string(),
            Object::Null => return "Null".to_string(),
        }
    }

}
impl ObjectType for Object {
        fn inspect(&self) -> String {
        match self {
            Object::Integer(int) => return int.inspect(),
            Object::Boolean(bool) => return bool.inspect(),
            Object::Return(ret) => return ret.inspect(),
            Object::String(string) => return string.clone(),
            Object::Error(str) => return str.to_string(),
            Object::Function(funct) => return funct.inspect(),
            Object::Null => return "Null".to_string(),
        }
    }
}
impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(int) => int.fmt(f),
            Object::Boolean(bool) => bool.fmt(f),
            Object::Return(ret) => ret.fmt(f),
            Object::String(string) => write!(f, "String: {}", string),
            Object::Error(str) => write!(f, "{}", str),
            Object::Function(funct) => funct.fmt(f),
            Object::Null => write!(f, "Null value"),
        }
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct Integer {
    pub value: i64,
}
impl Default for Integer {
    fn default() -> Self {
        Integer { 
            value: -1,
        }
    }
}
impl Integer {
    pub fn new(num: String) -> Integer {
        let mut int = Integer::default();
        int.value = num.parse::<i64>().unwrap();
        int
    }
    pub fn get_value(&self) -> i64 {
        return self.value;
    }
    pub fn from_num(num: i64) -> Integer {
        let mut int = Integer::default();
        int.value = num;
        int
    }
}
impl ObjectType for Integer {
    fn inspect(&self) -> String {
        return self.value.to_string();
    }
}
impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value in INTEGER: {}", &self.value)
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct Boolean {
    pub value: bool,
}
impl Default for Boolean {
    fn default() -> Self {
         Boolean {
            value: false,
         }
    }
}
impl Boolean {
    pub fn new(bool: bool) -> Boolean {
        let mut boo = Boolean::default();
        boo.value = bool;
        boo
    }
}
impl ObjectType for Boolean {
    fn inspect(&self) -> String {
        return self.value.to_string();
    }
}
impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value: {}", &self.value)
    }
}
pub struct Null {}
impl Default for Null {
    fn default() -> Self {
        Null {}
    }
}
impl ObjectType for Null {
    fn inspect(&self) -> String {
        return "Null".to_string();
    }
}
impl Display for Null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Object Type: {}", "Null".to_string())
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct Return {
    pub value: Box<Object>,
}
impl Return {
    pub fn new(obj: Object) -> Self {
        Return {
            value: Box::new(obj),
        }
    }
}
impl ObjectType for Return {
    fn inspect(&self) -> String {
        return self.value.inspect().to_string();
    }
}
impl Display for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub parameters: Parameters,
    pub body: BlockStatement,
    pub environment: RefCell<Environment>,
}
impl ObjectType for Function {
    fn inspect(&self) -> String {
        let mut params = "fn (".to_string();
        for entry in &self.parameters.variables {
            params.push_str(&entry.to_string());
            params.push_str(", ");
        }
            params.pop();
            params.pop();
            params.push_str(") {\n");
            params.push_str(&self.body.to_string());
            params.push_str("\n}");
        return params;
    }
}
impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut params = "fn (".to_string();
        for entry in &self.parameters.variables {
            params.push_str(&entry.to_string());
            params.push_str(", ");
        }
            params.pop();
            params.pop();
            params.push_str(") {\n");
            params.push_str(&self.body.to_string());
            params.push_str("\n}");
        write!(f, "{}", params)
    }   
}
#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    // outer used to keep track of FN variable bindings
    //uses RC to prevent infinite recursion
    //uses RefCell to allow interior mutability
    outer: Option<Rc<RefCell<Environment>>>,
}
impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }
    pub fn is_empty(&self) -> bool {
        if self.store.is_empty() {
            return true;
        } else {
            return false;
        }
    }
    pub fn new_enclosed_environment(outer: Rc<RefCell<Environment>>) -> Self {
        let mut environment = Environment::new();
        environment.outer = Some(outer);
        return environment;
    }
    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(object) => {
                return Some(object.clone())
            }
            None => match self.outer {
                Some(ref outer) => outer.borrow_mut().get(name),
                None => None
            },
        }
    }
    pub fn set(&mut self, name: String, obj: &Object) -> Result<Option<Object>, EvalError> {
        Ok(self.store.insert(name, obj.clone()))
    }

    pub fn print(&self) -> () {
        if self.store.is_empty() { 
           //println!("Empty store");
        }
        for (name, value) in &self.store {
            println!("name: {}, value: {}", name, value);
        }
    }
}
