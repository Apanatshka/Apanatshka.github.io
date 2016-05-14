use std::collections::HashMap;

// Sort -> set of rule alternatives
type Grammar = HashMap<Sort, Vec<RuleBody>>;
type RuleBody = Vec<RulePart>;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
enum RulePart {
  Lit { literal : &'static str },
  Srt { sort : Sort },
  Epsilon,
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
enum Sort {
  S,
}

fn main() {
  let mut binary_palindrome : Grammar = HashMap::new();
  
  let zero : RulePart = RulePart::Lit { literal : "0" };
  let one : RulePart = RulePart::Lit { literal : "1" };
  let s : RulePart = RulePart::Srt { sort : Sort::S };
  
  binary_palindrome.insert(Sort::S, vec![
    vec![ zero, s, zero],
  	vec![ one, s, one ],
  	vec![ RulePart::Epsilon ],
  ]);
  
  for (&sort, rules) in binary_palindrome.iter() {
    println!("{:?} -> {:?}", sort, rules);
  }
}
