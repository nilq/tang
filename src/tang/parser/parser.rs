use super::*;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<&'p Token<'p>>,
  source: &'p Source,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<&'p Token<'p>>, source: &'p Source) -> Self {
    Parser {
      tokens,
      source,
      index: 0,
    }
  }



  pub fn parse(&mut self) -> Result<Vec<Statement<'p>>, ()> {
    let mut ast = Vec::new();

    while self.remaining() > 0 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }



  fn parse_statement(&mut self) -> Result<Statement<'p>, ()> {
    use self::TokenType::*;

    while self.current_type() == &EOL && self.remaining() != 0 {
      self.next()?
    }

    let statement = match *self.current_type() {
      Keyword => match self.current_lexeme().as_str() {
        "return" => {
          let position = self.current_position();

          self.next()?;

          if ["}", "\n"].contains(&self.current_lexeme().as_str()) {
            Statement::new(
              StatementNode::Return(None),
              position
            )
          } else {
            Statement::new(
              StatementNode::Return(Some(Rc::new(self.parse_expression()?))),
              self.span_from(position)
            )
          }
        },

        _ => {
          let expression = self.parse_expression()?;

          let position = expression.pos.clone();

          Statement::new(
            StatementNode::Expression(expression),
            position,
          )
        },
      },

      Identifier => {
        let backup_index = self.index;
        let position     = self.current_position();
        let name         = self.eat_type(&Identifier)?;

        match self.current_lexeme().as_str() {
          ":" => {
            self.next()?;

            let position = self.current_position();
            let backup   = self.index;

            if let Some(right) = self.parse_right_hand()? {
              Statement::new(
                StatementNode::Variable(
                  Type::from(TypeNode::Nil),
                  name,
                  Some(right)
                ),
                self.span_from(position)
              )
            } else {
              self.index = backup;

              let kind = if self.current_lexeme() == "=" {
                Type::from(TypeNode::Nil)
              } else {
                self.parse_type()?
              };

              if self.current_lexeme() == "=" {
                self.next()?;

                Statement::new(
                  StatementNode::Variable(
                    kind,
                    name,
                    Some(self.parse_expression()?)
                  ),
                  self.span_from(position)
                )
              } else {
                Statement::new(
                  StatementNode::Variable(
                    kind,
                    name,
                    None
                  ),
                  self.span_from(position)
                )
              }
            }
          },

          "=" => {
            self.next()?;

            Statement::new(
              StatementNode::Assignment(
                Expression::new(
                  ExpressionNode::Identifier(name),
                  position.clone()
                ),

                self.parse_expression()?
              ),
              position,
            )
          },

          _ => {
            self.index = backup_index;

            let expression = self.parse_expression()?;

            let position = expression.pos.clone();

            Statement::new(
              StatementNode::Expression(expression),
              position,
            )
          },
        }
      },

      _ => {
        let expression = self.parse_expression()?;
        let position   = expression.pos.clone();

        if self.current_lexeme() == "=" {
          self.next()?;

          Statement::new(
            StatementNode::Assignment(expression, self.parse_expression()?),
            position
          )
        } else {
          Statement::new(
            StatementNode::Expression(expression),
            position,
          )
        }
      },
    };

    self.new_line()?;

    Ok(statement)
  }

  fn parse_right_hand(&mut self) -> Result<Option<Expression<'p>>, ()> {
    let declaration = match self.current_lexeme().as_str() {
      "def" => {
        let mut position = self.current_position();

        self.next()?;
        self.next_newline()?;

        let generics = if self.current_lexeme() == "<" {
          self.parse_block_of(("<", ">"), &Self::_parse_name)?
        } else {
          Vec::new()
        };

        self.next_newline()?;

        let params = if self.current_lexeme() == "(" {
          self.parse_block_of(("(", ")"), &Self::_parse_param_comma)?
        } else {
          Vec::new()
        };

        let retty = if self.current_lexeme() == "->" {
          self.next()?;

          self.parse_type()?
        } else {
          Type::from(TypeNode::Nil)
        };

        position = self.span_from(position);

        self.next_newline()?;

        self.expect_lexeme("{")?;

        Some(
          Expression::new(
            ExpressionNode::Function(
              params,
              retty,
              Rc::new(self.parse_expression()?),
              Some(generics)
            ),

            position
          )
        )
      }
      _ => None
    };

    Ok(declaration)
  }



  fn parse_expression(&mut self) -> Result<Expression<'p>, ()> {
    let atom = self.parse_atom()?;

    if self.current_type() == &TokenType::Operator {
      self.parse_binary(atom)
    } else {
      Ok(atom)
    }
  }



  fn parse_atom(&mut self) -> Result<Expression<'p>, ()> {
    use self::TokenType::*;

    if self.remaining() == 0 {
      Ok(
        Expression::new(
          ExpressionNode::EOF,
          self.current_position()
        )
      )
    } else {
      let token_type = self.current_type().clone();
      let position   = self.current_position();

      let expression = match token_type {
        Int => Expression::new(
          ExpressionNode::Int(self.eat()?.parse::<u64>().unwrap()),
          position
        ),

        Float => Expression::new(
          ExpressionNode::Float(self.eat()?.parse::<f64>().unwrap()),
          position
        ),

        Char => Expression::new(
          ExpressionNode::Char(self.eat()?.chars().last().unwrap()),
          position
        ),

        Str => Expression::new(
          ExpressionNode::Str(self.eat()?),
          position
        ),

        Identifier => Expression::new(
          ExpressionNode::Identifier(self.eat()?),
          position
        ),

        Bool => Expression::new(
          ExpressionNode::Bool(self.eat()? == "true"),
          position
        ),

        Operator => match self.current_lexeme().as_str() {
          "*" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Unwrap(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected symbol `{}`", symbol)),
              self.source.file,
              TokenElement::Ref(self.current())
            )
          )
        },

        Symbol => match self.current_lexeme().as_str() {
          "{" => Expression::new(
            ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
            position
          ),

          "[" => Expression::new(
            ExpressionNode::Array(self.parse_block_of(("[", "]"), &Self::_parse_expression_comma)?),
            self.span_from(position)
          ),

          "(" => {
            self.next()?;
            self.next_newline()?;

            if self.current_lexeme() == ")" {
              self.next()?;

              Expression::new(
                ExpressionNode::Empty,
                self.span_from(position)
              )
            } else {
              let expression = self.parse_expression()?;

              self.eat_lexeme(")")?;

              expression
            }
          },

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected symbol `{}`", symbol)),
              self.source.file,
              TokenElement::Ref(self.current())
            )
          )
        },

        Keyword => match self.current_lexeme().as_str() {
          "if" => {
            self.next()?;

            let condition   = Rc::new(self.parse_expression()?);
            let if_position = self.span_from(position.clone());

            let body        = Rc::new(
              Expression::new(
                ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                position
              )
            );

            let mut elses = Vec::new();

            loop {
              let branch_position = self.current_position();

              match self.current_lexeme().as_str() {
                "elif" => {
                  self.next()?;

                  let condition = self.parse_expression()?;
                  let position  = self.current_position();
                  let body      = Expression::new(
                    ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                    position
                  );

                  elses.push((Some(condition), body, branch_position))
                },

                "else" => {
                  self.next()?;

                  let position  = self.current_position();
                  let body      = Expression::new(
                    ExpressionNode::Block(self.parse_block_of(("{", "}"), &Self::_parse_statement)?),
                    position
                  );

                  elses.push((None, body, branch_position))
                },

                _ => break,
              }
            }

            Expression::new(
              ExpressionNode::If(condition, body, if elses.len() > 0 { Some(elses) } else { None }),
              if_position
            )
          },

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected keyword `{}`", symbol)),
              self.source.file,
              TokenElement::Ref(self.current())
            )
          )
        },

        ref token_type => return Err(
          response!(
            Wrong(format!("unexpected token `{}`", token_type)),
            self.source.file,
            TokenElement::Ref(self.current())
          )
        )
      };

      self.parse_postfix(expression)
    }
  }



  fn parse_postfix(&mut self, expression: Expression<'p>) -> Result<Expression<'p>, ()> {
    match *self.current_type() {
      TokenType::Symbol => match self.current_lexeme().as_str() {
        "(" => {
          let args = self.parse_block_of(("(", ")"), &Self::_parse_expression_comma)?;

          let position = expression.pos.clone();

          let call = Expression::new(
            ExpressionNode::Call(Rc::new(expression), args),
            self.span_from(position)
          );

          self.parse_postfix(call)
        },

        "[" => {
          self.next()?;

          let expr = self.parse_expression()?;

          self.eat_lexeme("]")?;

          let position = expression.pos.clone();

          let index = Expression::new(
            ExpressionNode::Index(Rc::new(expression), Rc::new(expr)),
            self.span_from(position)
          );

          self.parse_postfix(index)
        },

        _ => Ok(expression)
      },

      TokenType::Keyword => match self.current_lexeme().as_str() {
        "as" => {
          self.next()?;

          let t        = self.parse_type()?;
          let position = expression.pos.clone();

          self.parse_postfix(
            Expression::new(
              ExpressionNode::Cast(Rc::new(expression), t),
              position
            )
          )
        },

        _ => Ok(expression)
      },

      _ => Ok(expression)
    }
  }



  // A simple shunting-yard implementation
  // ... naively hoping for unary operations to just play along
  fn parse_binary(&mut self, left: Expression<'p>) -> Result<Expression<'p>, ()> {
    let left_position = left.pos.clone();

    let mut expression_stack = vec!(left);
    let mut operator_stack   = vec!(Operator::from_str(&self.eat()?).unwrap());

    expression_stack.push(self.parse_atom()?);

    while operator_stack.len() > 0 {
      while self.current_type() == &TokenType::Operator {
        let position               = self.current_position();
        let (operator, precedence) = Operator::from_str(&self.eat()?).unwrap();

        if precedence < operator_stack.last().unwrap().1 {
          let right = expression_stack.pop().unwrap();
          let left  = expression_stack.pop().unwrap();

          expression_stack.push(
            Expression::new(
              ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
              self.current_position(),
            )
          );

          if self.remaining() > 0 {
            expression_stack.push(self.parse_atom()?);
            operator_stack.push((operator, precedence))
          } else {
            return Err(
              response!(
                Wrong("reached EOF in operation"),
                self.source.file,
                position
              )
            )
          }
        } else {
          expression_stack.push(self.parse_atom()?);
          operator_stack.push((operator, precedence))
        }
      }

      let right = expression_stack.pop().unwrap();
      let left  = expression_stack.pop().unwrap();

      expression_stack.push(
        Expression::new(
          ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
          self.current_position(),
        )
      );
    }

    let expression = expression_stack.pop().unwrap();

    Ok(
      Expression::new(
        expression.node,
        self.span_from(left_position)
      )
    )
  }



  fn parse_type(&mut self) -> Result<Type<'p>, ()> {
    use self::TokenType::*;

    let t = match *self.current_type() {
      Identifier => match self.eat()?.as_str() {
        "str"   => Type::from(TypeNode::Str),
        "char"  => Type::from(TypeNode::Char),

        "int"   => Type::from(TypeNode::Int),
        "float" => Type::from(TypeNode::Float),

        "bool"  => Type::from(TypeNode::Bool),
        id      => Type::id(id),
      },

      Symbol => match self.current_lexeme().as_str() {
        "[" => {
          self.next()?;
          self.next_newline()?;

          let t = self.parse_type()?;

          self.next_newline()?;

          self.eat_lexeme(";")?;

          self.next_newline()?;

          let expression = self.parse_expression()?;

          let len = if let ExpressionNode::Int(ref len) = Self::fold_expression(&expression)?.node {
            *len as usize
          } else {
            return Err(
              response!(
                Wrong(format!("length of array can be nothing but int")),
                self.source.file,
                expression.pos
              )
            )
          };

          self.eat_lexeme("]")?;

          Type::array(t, len)
        },

        "(" => {
          let params = self.parse_block_of(("(", ")"), &Self::_parse_type_comma)?;

          self.eat_lexeme("->")?;

          let return_type = self.parse_type()?;

          Type::function(params, return_type)
        },

        _ => return Err(
          response!(
            Wrong(format!("unexpected symbol `{}` in type", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      }

      _ => return Err(
        response!(
          Wrong(format!("expected type found `{}`", self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    };

    Ok(t)
  }



  fn new_line(&mut self) -> Result<(), ()> {
    if self.remaining() > 0 {
      match self.current_lexeme().as_str() {
        "\n" => self.next(),
        _    => Err(
          response!(
            Wrong(format!("expected new line found: `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      }
    } else {
      Ok(())
    }
  }



  fn next_newline(&mut self) -> Result<(), ()> {
    while self.current_lexeme() == "\n" && self.remaining() > 0 {
      self.next()?
    }

    Ok(())
  }



  fn next(&mut self) -> Result<(), ()> {
    if self.index <= self.tokens.len() {
      self.index += 1;
      Ok(())
    } else {
      Err(
        response!(
          Wrong("moving outside token stack"),
          self.source.file
        )
      )
    }
  }

  fn remaining(&self) -> usize {
    self.tokens.len().saturating_sub(self.index)
  }

  fn current_position(&self) -> TokenElement<'p> {
    let current = self.current();

    TokenElement::Pos(
      current.line,
      current.slice
    )
  }

  fn span_from(&self, left_position: TokenElement<'p>) -> TokenElement<'p> {
    match left_position {
      TokenElement::Pos(ref line, ref slice) => if let TokenElement::Pos(_, ref slice2) = self.current_position() {
        TokenElement::Pos(*line, (slice.0, if slice2.1 < line.1.len() { slice2.1 } else { line.1.len() } ))
      } else {
        left_position.clone()
      },

      _ => left_position.clone(),
    }
  }

  fn current(&self) -> &'p Token<'p> {
    if self.index > self.tokens.len() - 1 {
      &self.tokens[self.tokens.len() - 1]
    } else {
      &self.tokens[self.index]
    }
  }

  fn eat(&mut self) -> Result<String, ()> {
    let lexeme = self.current().lexeme.clone();
    self.next()?;

    Ok(lexeme)
  }

  fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
    if self.current_lexeme() == lexeme {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn eat_type(&mut self, token_type: &TokenType) -> Result<String, ()> {
    if self.current_type() == token_type {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  fn current_type(&self) -> &TokenType {
    &self.current().token_type
  }

  fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
    if self.current_type() == &token_type {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file
        )
      )
    }
  }

  fn expect_lexeme(&self, lexeme: &str) -> Result<(), ()> {
    if self.current_lexeme() == lexeme {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file
        )
      )
    }
  }



  // A helper method for parsing sequences defined by provided static methods,
  // for as long as given static method returns Some(B)
  fn parse_block_of<B>(&mut self, delimeters: (&str, &str), parse_with: &Fn(&mut Self) -> Result<Option<B>, ()>) -> Result<Vec<B>, ()> {
    self.eat_lexeme(delimeters.0)?;

    let mut block_tokens = Vec::new();
    let mut nest_count   = 1;

    while nest_count > 0 {
      if self.current_lexeme() == delimeters.1 {
        nest_count -= 1
      } else if self.current_lexeme() == delimeters.0 {
        nest_count += 1
      }

      if nest_count == 0 {
        break
      } else {
        block_tokens.push(self.current());

        self.next()?
      }
    }

    self.eat_lexeme(delimeters.1)?;

    if !block_tokens.is_empty() {
      let mut parser = Parser::new(block_tokens, self.source);
      let mut block  = Vec::new();

      while let Some(element) = parse_with(&mut parser)? {
        block.push(element)
      }

      Ok(block)
    } else {
      Ok(Vec::new())
    }
  }



  fn _parse_statement(self: &mut Self) -> Result<Option<Statement<'p>>, ()> {
    if self.remaining() > 0 {
      Ok(Some(self.parse_statement()?))
    } else {
      Ok(None)
    }
  }



  fn _parse_expression(self: &mut Self) -> Result<Option<Expression<'p>>, ()> {
    let expression = self.parse_expression()?;

    match expression.node {
      ExpressionNode::EOF => Ok(None),
      _                   => Ok(Some(expression)),
    }
  }



  fn _parse_name(self: &mut Self) -> Result<Option<String>, ()> {
    if self.remaining() == 0 {
      Ok(None)
    } else {
      let t = self.eat_type(&TokenType::Identifier)?;

      if self.remaining() > 0 {
        self.eat_lexeme(",")?;

        if self.remaining() > 0 && self.current_lexeme() == "\n" {
          self.next()?
        }
      }

      Ok(Some(t))
    }
  }



  // Static method for parsing sequence `expr* ,* \n*` - for things like [1, 2, 3, 4,]
  fn _parse_expression_comma(self: &mut Self) -> Result<Option<Expression<'p>>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    let expression = Self::_parse_expression(self);

    if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }

    if self.remaining() > 0 {
      self.eat_lexeme(",")?;

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    expression
  }



  fn _parse_param_comma(self: &mut Self) -> Result<Option<(String, Type<'p>)>, ()> {
    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() == 0 {
      return Ok(None)
    }

    let mut splat = false;

    if self.current_lexeme() == ".." {
      splat = true;

      self.next()?;
      self.next_newline()?;
    }

    let name = self.eat_type(&TokenType::Identifier)?;
    
    self.eat_lexeme(":")?;

    let mut kind = self.parse_type()?;

    if splat {
      kind.mode = TypeMode::Splat(None)
    }

    let param = Some((name, kind));

    if self.remaining() > 0 && self.current_lexeme() == "\n" {
      self.next()?
    }

    if self.remaining() > 0 {
      self.eat_lexeme(",")?;

      if self.remaining() > 0 && self.current_lexeme() == "\n" {
        self.next()?
      }
    }

    Ok(param)
  }



  fn _parse_type_comma(self: &mut Self) -> Result<Option<Type<'p>>, ()> {
    if self.remaining() == 0 {
      Ok(None)
    } else {
      let t = self.parse_type()?;

      if self.remaining() > 0 {
        self.eat_lexeme(",")?;

        if self.remaining() > 0 && self.current_lexeme() == "\n" {
          self.next()?
        }
      }

      Ok(Some(t))
    }
  }



  pub fn fold_expression<'v>(expression: &Expression<'v>) -> Result<Expression<'v>, ()> {
    use self::ExpressionNode::*;
    use self::Operator::*;

    let node = match expression.node {
      Binary(ref left, ref op, ref right) => {
        let node = match (&Self::fold_expression(&*left)?.node, op, &Self::fold_expression(&*right)?.node) {
          (&Int(ref a),   &Add, &Int(ref b))     => Int(a + b),
          (&Float(ref a), &Add, &Float(ref b)) => Float(a + b),
          (&Int(ref a),   &Sub, &Int(ref b))     => Int(a - b),
          (&Float(ref a), &Sub, &Float(ref b)) => Float(a - b),
          (&Int(ref a),   &Mul, &Int(ref b))     => Int(a * b),
          (&Float(ref a), &Mul, &Float(ref b)) => Float(a * b),
          (&Int(ref a),   &Div, &Int(ref b))     => Int(a / b),
          (&Float(ref a), &Div, &Float(ref b)) => Float(a / b),

          _ => expression.node.clone()
        };

        Expression::new(
          node,
          expression.pos.clone()
        )
      },

      _ => expression.clone()
    };

    Ok(node)
  }
}