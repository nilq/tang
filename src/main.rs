extern crate colored;

mod tang;
use tang::lexer::*;
use tang::source::*;

fn main() {
  let content = r#"
aw: int = 1000
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

  println!("{:#?}", tokens)
}
