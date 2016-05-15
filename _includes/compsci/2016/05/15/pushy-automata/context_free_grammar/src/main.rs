use std::collections::HashMap;

// Sort -> set of rule alternatives
type Grammar = HashMap<Variable, Vec<RuleBody>>;
type RuleBody = Vec<RulePart>;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
enum RulePart {
  Lit(&'static str),
  Var(Variable),
  Epsilon,
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
enum Variable {
  S,
}

fn main() {
  let mut binary_palindrome : Grammar = HashMap::new();
  
  let zero : RulePart = RulePart::Lit("0");
  let one : RulePart = RulePart::Lit("1");
  let s : RulePart = RulePart::Var(Variable::S);
  
  binary_palindrome.insert(Variable::S, vec![
    vec![ zero, s, zero],
  	vec![ one, s, one ],
  	vec![ RulePart::Epsilon ],
  ]);
  
  for (&sort, rules) in binary_palindrome.iter() {
    println!("{:?} -> {:?}", sort, rules);
  }
}
