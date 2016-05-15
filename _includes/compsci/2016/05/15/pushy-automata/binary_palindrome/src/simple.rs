/*
Simpler (or at least smaller) -- still more or less directly encoded -- PDA. Has a bit of overhead compared to the one in main, see output. 
*/

static INPUT: [u8; 16] = [0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0];

type State = u8;
type Input = u8;
type Stack = Vec<u8>;

fn transition(state: State, stack: Stack, input: Input) -> Vec<(State, Stack)> {
  match state {
    0 => {
      let mut new_stack = stack.clone();
      new_stack.push(input);
      vec![ (0,new_stack.clone()), (1,new_stack) ]
    }
    1 => {
      if stack.last().map_or(false, |top| *top == input) {
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

fn main() {
  let mut pda_states = vec![(0,vec![]), (1, vec![])];
  for input in INPUT.iter() {
    println!("{:?}", pda_states);
    let mut new_pda_states = vec![];
    for (state, stack) in pda_states {
      new_pda_states.append(&mut transition(state, stack, *input));
    }
    pda_states = new_pda_states;
  }
  for (state, stack) in pda_states {
    if state == 1 && stack.is_empty() {
      println!("The input is accepted");
      return;
    }
  }
  println!("The input is not accepted");
}
