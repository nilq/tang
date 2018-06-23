use std::fmt::{ self, Display, Write, Formatter };
use std::rc::Rc;

use super::*;



#[derive(Debug, Clone)]
pub enum TypeNode {
  Int,
  Float,
  Bool,
  Str,
  Char,
  Nil,
  Id(String),
  Set(Vec<Type>),
  Array(Rc<Type>),
  Func(Vec<Type>, Rc<Type>),
}

impl TypeNode {
  pub fn check_expression(&self, other: &ExpressionNode) -> bool {
    use self::TypeNode::*;

    match *other {
      ExpressionNode::Int(_) => match *self {
        Int | Float => true,
        _           => false,
      },

      ExpressionNode::Array(ref content) => {
        for element in content {
          if let &Array(ref content) = self {
            if !content.node.check_expression(&element.node) {
              return false
            }
          }
        }

        true
      },

      _ => false
    }
  }
}

impl PartialEq for TypeNode {
  fn eq(&self, other: &TypeNode) -> bool {
    use self::TypeNode::*;

    match (self, other) {
      (&Int,   &Int)   => true,
      (&Float, &Float) => true,

      (&Bool, &Bool) => true,
      (&Str,  &Str)  => true,
      (&Char, &Char) => true,
      (&Nil,  &Nil)  => true,

      (&Array(ref a), &Array(ref b)) => a == b,
      (&Id(ref a), &Id(ref b))       => a == b,
      (&Set(ref a), &Set(ref b))     => a == b,

      _                              => false,
    }
  }
}



#[derive(Debug, Clone)]
pub enum TypeMode {
  Undeclared,
  Immutable,
  Optional,
  Regular,
}

impl Display for TypeNode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeNode::*;

    match *self {
      Int              => write!(f, "int"),
      Float           => write!(f, "double"),
      Bool             => write!(f, "bool"),
      Str              => write!(f, "string"),
      Char             => write!(f, "char"),
      Nil              => write!(f, "nil"),
      Array(ref n)     => write!(f, "[{}]", n),
      Id(ref n)        => write!(f, "{}", n),
      Set(ref content) => {
        write!(f, "(");

        for (index, element) in content.iter().enumerate() {
          if index < content.len() - 1 {
            write!(f, "{}, ", element)?
          } else {
            write!(f, "{}", element)?
          }
        }

        write!(f, ")")
      },
      Func(ref params, ref return_type) => {
        write!(f, "(");

        for (index, element) in params.iter().enumerate() {
          if index < params.len() - 1 {
            write!(f, "{}, ", element)?
          } else {
            write!(f, "{}", element)?
          }
        }

        write!(f, ") {}", return_type)
      },
    }
  }
}



impl TypeMode {
  pub fn check(&self, other: &TypeMode) -> bool {
    use self::TypeMode::{ Optional, Immutable, Regular, Undeclared, };

    match (self, other) {
      (&Regular,       &Regular)    => true,
      (&Immutable,     &Immutable)  => true,
      (&Undeclared,    &Undeclared) => true,
      (&Optional,      &Optional)   => true,
      _                             => false,
    }
  }
}

impl Display for TypeMode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::TypeMode::*;

    match *self {
      Regular    => Ok(()),
      Immutable  => write!(f, "constant "),
      Undeclared => write!(f, "undeclared "),
      Optional   => write!(f, "optional "),
    }
  }
}

impl PartialEq for TypeMode {
  fn eq(&self, other: &TypeMode) -> bool {
    use self::TypeMode::*;

    match (self, other) {
      (&Regular,    &Regular)    => true,
      (&Regular,    &Immutable)  => true,
      (&Immutable,  &Immutable)  => true,
      (&Immutable,  &Regular)    => true,
      (_,           &Optional)   => true,
      (&Optional,   _)           => true,
      (&Undeclared, _)           => false,
      (_,           &Undeclared) => false,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Type {
  pub node: TypeNode,
  pub mode: TypeMode,
}

impl Type {
  pub fn new(node: TypeNode, mode: TypeMode) -> Self {
    Type {
      node, mode,
    }
  }

  pub fn id(id: &str) -> Type {
    Type::new(TypeNode::Id(id.to_owned()), TypeMode::Regular)
  }

  pub fn from(node: TypeNode) -> Type {
    Type::new(node, TypeMode::Regular)
  }

  pub fn set(content: Vec<Type>) -> Type {
    Type::new(TypeNode::Set(content), TypeMode::Regular)
  }

  pub fn array(t: Type) -> Type {
    Type::new(TypeNode::Array(Rc::new(t)), TypeMode::Regular)
  }

  pub fn function(params: Vec<Type>, return_type: Type) -> Type {
    Type::new(TypeNode::Func(params, Rc::new(return_type)), TypeMode::Regular)
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}{}", self.mode, self.node)
  }
}