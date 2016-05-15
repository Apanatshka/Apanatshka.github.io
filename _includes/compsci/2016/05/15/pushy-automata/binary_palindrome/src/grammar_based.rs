type State = u8;
type InputSymbol = u8;
type StackSymbol = u8;
type Stack = Vec<StackSymbol>;

const EOS: StackSymbol = 2; // End of Stack symbol
const S: StackSymbol = 3; // The sort S from the grammar

fn transition(state: State, input: InputSymbol, stack: Stack)
    -> Vec<(State, Stack)> {
  match state {
    0 => {
      if let Some(&top) = stack.last() {
        if top == input {
          let mut new_stack = stack.clone();
          new_stack.pop();
          vec![ (0, new_stack) ]
        } else if top == S {
          let mut new_stack1 = stack.clone();
          new_stack1.pop();
          new_stack1.push(input);
          let mut new_stack2 = new_stack1.clone();
          new_stack2.push(S);
          vec![ (0, new_stack2), (0, new_stack1) ] // this order gives nicer debug output
        } else {
          Vec::new()
        }
      } else {
        Vec::new()
      }
    }
    _ => Vec::new()
  }
}

fn epsilon_transition(state: State, stack: Stack) -> Vec<(State, Stack)> {
  match state {
    0 => {
      if stack.last().map_or(false, |&top| top == EOS) {
        let mut new_stack = stack.clone();
        new_stack.pop();
        vec![ (1,new_stack) ]
      }
      else {
        Vec::new()
      }
    }
    _ => Vec::new()
  }
}

const FINALS: [State; 1] = [1];

const INPUT: [InputSymbol; 16] =
  [0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0];

fn main() {
  let mut pda_states = vec![(0, vec![EOS, S]), (1,vec![])];
  for &input in INPUT.iter() {
    println!("{:?}", pda_states);
    let mut new_pda_states = vec![];
    for (state, stack) in pda_states {
      new_pda_states.append(&mut transition(state, input, stack));
    }
    pda_states = new_pda_states;
  }
  while !pda_states.is_empty() {
    println!("{:?}", pda_states);
    let mut new_pda_states = vec![];
    for (state, stack) in pda_states {
      if FINALS.iter().any(|&x| x == state) {
        println!("The input is accepted");
        return;
      }
      else {
        new_pda_states.append(&mut epsilon_transition(state, stack));
      }
    }
    pda_states = new_pda_states;
  }
  println!("The input is not accepted");
}