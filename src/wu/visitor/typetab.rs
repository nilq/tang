use std::cell::RefCell;
use super::{ Type, TypeNode, };
use super::super::error::Response::Wrong;

use std::rc::Rc;
use std::collections::HashMap;



#[derive(Clone, Debug)]
pub struct TypeTab<'t> {
  pub parent: Option<Rc<TypeTab<'t>>>,
  pub types:  RefCell<Vec<Type<'t>>>, // type and offset
  pub covers: HashMap<String, Type<'t>>,  
}

impl<'t> TypeTab<'t> {
  pub fn new(parent: Rc<Self>, types: &[Type<'t>], covers: HashMap<String, Type<'t>>) -> Self {
    TypeTab {
      parent: Some(parent),
      types:  RefCell::new(types.to_owned()),
      covers,
    }
  }

  

  pub fn global() -> Self {
    TypeTab {
      parent: None,
      types:  RefCell::new(Vec::new()),
      covers: HashMap::new(),
    }
  }



  pub fn set_type(&self, index: usize, env_index: usize, t: Type<'t>) -> Result<(), ()> {
    if env_index == 0 {
      match self.types.borrow_mut().get_mut(index) {
        Some(v) => {
          *v = t;
          Ok(())
        },
        None => self.set_type(index, env_index + 1, t)
      }
    } else {
      match self.parent {
        Some(ref p) => p.set_type(index, env_index - 1, t),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }



  pub fn get_type(&self, index: usize, env_index: usize) -> Result<Type<'t>, ()> {
    if env_index == 0 {
      match self.types.borrow().get(index) {
        Some(v) => Ok(v.clone()),
        None    => self.get_type(index, env_index + 1)
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_type(index, env_index - 1),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }



  pub fn get_cover(&self, index: String, env_index: usize) -> Result<Type<'t>, ()> {
    if env_index == 0 {
      match self.covers.get(&index) {
        Some(v) => Ok(v.clone()),
        None    => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_cover(index, env_index - 1),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }



  pub fn visualize(&self, env_index: usize) {
    if env_index > 0 {
      if let Some(ref p) = self.parent {
        p.visualize(env_index - 1);
        println!("------------------------------");
      }
    }

    for (i, v) in self.types.borrow().iter().enumerate() {
      println!("({} : {}) = {:?}", i, env_index, v)
    }
  }



  pub fn size(&self) -> usize {
    self.types.borrow().len()
  }

  pub fn grow(&mut self) {
    RefCell::borrow_mut(&self.types).push(Type::from(TypeNode::Nil))
  }
}