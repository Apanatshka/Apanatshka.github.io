enum Input {
  ZERO,
  ONE,
}

fn transition0(inputs: &[Input]) {
  inputs
    .split_first()
    .map_or_else(
      || println!("The input is not accepted"),
      |(input, inputs)| match input {
        &Input::ONE  => transition1(inputs),
        &Input::ZERO => println!("The input is not accepted"),
      });
}

fn transition1(inputs: &[Input]) {
  inputs
    .split_first()
    .map_or_else(
      || println!("The input is not accepted"),
      |(input, inputs)| match input {
        &Input::ZERO => transition2(inputs),
        &Input::ONE => println!("The input is not accepted"),
      });
}

fn transition2(inputs: &[Input]) {
  inputs
    .split_first()
    .map_or_else(
      || println!("The input is not accepted"),
      |(input, inputs)| match input {
        &Input::ZERO => transition3(inputs),
        &Input::ONE => println!("The input is not accepted"),
      });
}

fn transition3(inputs: &[Input]) {
  inputs
    .split_first()
    .map_or_else(
      || println!("The input is not accepted"),
      |(input, inputs)| match input {
        &Input::ZERO => transition3(inputs),
        &Input::ONE  => transition4(inputs),
      });
}

fn transition4(inputs: &[Input]) {
  inputs
    .split_first()
    .map_or_else(
      || println!("The input is accepted"), 
      |(input, inputs)| match input {
        &Input::ONE => transition4(inputs),
        &Input::ZERO => println!("The input is not accepted"),
      });
}


const INPUTS: [Input; 7] = [Input::ONE, Input::ZERO, Input::ZERO, Input::ZERO, Input::ZERO, Input::ONE, Input::ONE];

fn main() {
  transition0(&INPUTS);
}
