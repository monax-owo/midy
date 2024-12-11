use std::error::Error;

use device_query::{DeviceEvents, DeviceState};

fn main() {
  println!("Hello, world!");
  match run() {
    Ok(()) => (),
    Err(err) => println!("error: {}", err.to_string()),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let device_state = DeviceState::new();
  let _guard = device_state.on_key_down(|key_code| println!("key_down: {:?}", key_code));
  let _guard = device_state.on_key_up(|key_code| println!("key_up: {:?}", key_code));
  loop {}
  Ok(())
}
