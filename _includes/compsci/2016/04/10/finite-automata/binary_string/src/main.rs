type State = u8;

type Input = bool;
const ZERO: Input = false;
const ONE: Input  = true;

fn transition(state: State, symbol: Input) -> Option<State> {
  match (state, symbol) {
    (0, ONE)  => Option::Some(1),
    (1, ZERO) => Option::Some(2),
    (2, ZERO) => Option::Some(3),
    (3, ZERO) => Option::Some(3),
    (3, ONE)  => Option::Some(4),
    (4, ONE)  => Option::Some(4),
    _ => Option::None,
  }
}

const START: State = 0;

const FINALS: [State; 1] = [4];


const INPUTS: [Input; 7] = [ONE, ZERO, ZERO, ZERO, ZERO, ONE, ONE];

fn main() {
  let mut state = Option::Some(START);
  for input in INPUTS.iter() {
    state = state.and_then(|st| transition(st, input.clone()));
  }
  if state.map_or(false, |st| FINALS.iter().any(|&x| x == st)) {
    println!("The input is accepted");
  } else {
    println!("The input is not accepted");
  }
}
