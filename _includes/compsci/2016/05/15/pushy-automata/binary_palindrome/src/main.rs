type State = u8;
type InputSymbol = u8;
type StackSymbol = u8;
type Stack = Vec<StackSymbol>;

const EOS: StackSymbol = 2; // End of Stack symbol

fn transition(state: State, input: InputSymbol, stack: Stack)
    -> Vec<(State, Stack)> {
  match state {
    0 => {
      let mut new_stack = stack.clone();
      new_stack.push(input);
      vec![ (1,new_stack) ]
    }
    1 => {
      let mut new_stack1 = stack.clone();
      new_stack1.push(input);
      match stack.last().map(|&top| top == input) {
        Some(true) => {
          let mut new_stack2 = stack.clone();
          new_stack2.pop();
          vec![ (1,new_stack1), (2,new_stack2) ]
        }
        Some(false) => {
          vec![ (1,new_stack1) ]
        }
        None => Vec::new()
      }
    }
    2 => {
      if stack.last().map_or(false, |&top| top == input) {
        let mut new_stack = stack.clone();
        new_stack.pop();
        vec![ (2,new_stack) ]
      }
      else {
        Vec::new()
      }
    }
    _ => Vec::new()
  }
}

fn epsilon_transition(state: State, stack: Stack) -> Vec<(State, Stack)> {
  match state {
    2 => {
      if stack.last().map_or(false, |&top| top == EOS) {
        let mut new_stack = stack.clone();
        new_stack.pop();
        vec![ (3,new_stack) ]
      }
      else {
        Vec::new()
      }
    }
    _ => Vec::new()
  }
}

const FINALS: [State; 2] = [0, 3];

const INPUT: [InputSymbol; 16] =
  [0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0];

fn main() {
  let mut pda_states = vec![(0,vec![EOS])];
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