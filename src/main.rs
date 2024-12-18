use std::{
  collections::HashMap,
  error::Error,
  io::{stdin, stdout, Write},
  ops::DerefMut,
  sync::{Arc, Mutex},
};

use device_query::{DeviceEvents, DeviceState, Keycode};
use midir::{MidiIO, MidiInput, MidiOutput, MidiOutputConnection};

struct NoteKeyMap {
  start: u8,
  key_codes: Vec<Keycode>,
}

impl NoteKeyMap {
  fn new(start: u8, key_codes: Vec<Keycode>) -> Self {
    Self { start, key_codes }
  }
}

impl From<NoteKeyMap> for HashMap<u8, Keycode> {
  fn from(val: NoteKeyMap) -> Self {
    HashMap::<u8, Keycode>::from_iter(
      val
        .key_codes
        .into_iter()
        .enumerate()
        .map(|v| (v.0 as u8 + val.start, v.1)),
    )
  }
}

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

fn main() {
  println!("Hello, world!");
  match run() {
    Ok(()) => (),
    Err(err) => println!("error: {}", err),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let device_state = DeviceState::new();

  let key_map: HashMap<u8, Keycode> = {
    const START: u8 = 36;
    // TODO: selectable

    // let map = vec![
    //   Keycode::A,
    //   Keycode::W,
    //   Keycode::S,
    //   Keycode::E,
    //   Keycode::D,
    //   Keycode::F,
    //   Keycode::T,
    //   Keycode::G,
    //   Keycode::Y,
    //   Keycode::H,
    //   Keycode::U,
    //   Keycode::J,
    //   Keycode::K,
    //   Keycode::O,
    //   Keycode::L,
    //   Keycode::P,
    //   Keycode::Semicolon,
    // ];

    // let map = vec![
    //   Keycode::Numpad1,
    //   Keycode::Numpad2,
    //   Keycode::Numpad3,
    //   Keycode::Numpad4,
    //   Keycode::Numpad5,
    //   Keycode::Numpad6,
    //   Keycode::Numpad7,
    //   Keycode::Numpad8,
    //   Keycode::Numpad9,
    //   Keycode::Numpad0,
    // ];

    let map = vec![
      Keycode::J,
      Keycode::K,
      Keycode::L,
      Keycode::U,
      Keycode::I,
      Keycode::O,
      Keycode::Key7,
      Keycode::Key8,
      Keycode::Key9,
      Keycode::M,
    ];
    NoteKeyMap::new(START, map).into()
  };

  let midi_input = MidiInput::new("in")?;
  for in_port in midi_input.ports() {
    println!("in ports: {:?}", in_port.id());
    stdout().flush()?;
  }
  let port_input = select_port(&midi_input, "in")?;
  let _connect_input = midi_input.connect(
    &port_input,
    "in",
    |num, data, _| println!("{} data: {:?}", num, data),
    (),
  )?;

  let midi_output = MidiOutput::new("out")?;
  for out_port in midi_output.ports() {
    println!("out ports: {:?}", out_port.id());
    stdout().flush()?;
  }
  let port_output = select_port(&midi_output, "out")?;
  let connect_output = midi_output.connect(&port_output, "0")?;

  let arc_resources = Arc::new((Mutex::new(connect_output), key_map));
  let _guard = device_state.on_key_down({
    let arc = Arc::clone(&arc_resources);

    move |key_code| {
      println!("key_down: {:?}", key_code);
      let mut lock = arc.0.lock().unwrap();
      send(NOTE_ON_MSG, lock.deref_mut(), key_code, &arc.1).unwrap();
    }
  });

  let _guard = device_state.on_key_up({
    let arc = Arc::clone(&arc_resources);

    move |key_code| {
      println!("key_up: {:?}", key_code);
      let mut lock = arc.0.lock().unwrap();
      send(NOTE_OFF_MSG, lock.deref_mut(), key_code, &arc.1).unwrap();
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

fn send(
  msg: u8,
  midi_output: &mut MidiOutputConnection,
  key_code: &Keycode,
  key_map: &HashMap<u8, Keycode>,
) -> Result<(), Box<dyn Error>> {
  for (note, key_map_code) in key_map.iter() {
    if std::mem::discriminant(key_code) == std::mem::discriminant(key_map_code) {
      midi_output.send(&[msg, *note, 100])?;
    }
  }
  Ok(())
}
