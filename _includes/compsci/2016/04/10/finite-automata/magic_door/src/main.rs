// 1
#[derive(Debug, Clone)]
enum State {
  Open,
  Closed,
}

// 2
#[derive(Debug, Clone)]
enum Input {
  Front,
  Back,
  Both,
  Neither,
}

// 3
fn transition(state: State, symbol: Input) -> State {
  match (state, symbol) {
    (_, Input::Front) => State::Open,
    _                 => State::Closed,
  }
}

// 4
const START: State = State::Closed;


// Execution
const INPUTS: [Input; 5] = [Input::Front, Input::Front, Input::Both, Input::Back, Input::Neither];

fn main() {
  let mut state = START;
  println!("The start state is: {:?}", state);
  for input in INPUTS.iter() {
    println!("The input is: {:?}", input);
    state = transition(state, input.clone());
    println!("The state is now: {:?}", state);
  }
}
