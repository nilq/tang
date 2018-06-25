extern crate colored;

mod tang;
use tang::lexer::*;
use tang::source::*;
use tang::parser::*;
use tang::visitor::*;

fn main() {
  let content = r#"
foo: def(a: str) {
  return
}

print: def<T> (..splat: T) {
  foo(*splat)
}

print("hey", "hey")
  "#;

  let source = Source::from("<static.wu>", content.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(content.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let tokens_ref = tokens.iter().map(|x| &*x).collect::<Vec<&Token>>();

  let mut parser = Parser::new(tokens_ref, &source);
  
  match parser.parse() {
    Ok(ast) => {
      println!("{:#?}", ast);

      let mut visitor = Visitor::new(&source, &ast);

      visitor.visit();
    },

    _ => return,
  }
}
