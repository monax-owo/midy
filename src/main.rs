use std::{
  collections::HashMap,
  error::Error,
  io::{stdin, stdout, Write},
  sync::{Arc, Mutex},
};

use device_query::{DeviceEvents, DeviceState, Keycode};
use midir::{MidiIO, MidiInput, MidiOutput};

fn main() {
  println!("Hello, world!");
  match run() {
    Ok(()) => (),
    Err(err) => println!("error: {}", err.to_string()),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let device_state = DeviceState::new();
  let _key_map = HashMap::<Keycode, u32>::new();

  let midi_output = MidiOutput::new("out")?;
  for port in midi_output.ports() {
    println!("2 ports: {:?}", port.id());
    stdout().flush()?;
  }
  let port_output = select_port(&midi_output, "out")?;
  println!("test: {:?}", port_output.id());
  let connect_output = midi_output.connect(&port_output, "out")?;
  let arc_output = Arc::new(Mutex::new(connect_output));

  let _guard_output = device_state.on_key_down({
    let arc = Arc::clone(&arc_output);

    move |key_code| {
      println!("key_down: {:?}", key_code);
      let mut arc = arc.lock().unwrap();
      if let Keycode::F = key_code {
        arc.send(&[144, 60, 100]).unwrap();
      }
    }
  });

  let _guard = device_state.on_key_up({
    let arc = Arc::clone(&arc_output);

    move |key_code| {
      println!("key_up: {:?}", key_code);
      let mut arc = arc.lock().unwrap();
      if let Keycode::F = key_code {
        arc.send(&[0x80, 60, 100]).unwrap();
      }
    }
  });
  loop {}
  Ok(())
}

// https://github.com/Boddlnagg/midir/blob/master/examples/test_forward.rs
fn select_port<T: MidiIO>(midi_io: &T, desc: &str) -> Result<T::Port, Box<dyn Error>> {
  println!("Available {} ports:", desc);
  let midi_ports = midi_io.ports();
  for (i, p) in midi_ports.iter().enumerate() {
    println!("{}: {}", i, midi_io.port_name(p)?);
  }
  print!("Please select {} port: ", desc);
  stdout().flush()?;
  let mut input = String::new();
  stdin().read_line(&mut input)?;
  let port = midi_ports
    .get(input.trim().parse::<usize>()?)
    .ok_or("Invalid port number")?;
  Ok(port.clone())
}
