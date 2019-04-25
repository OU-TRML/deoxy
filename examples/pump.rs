use std::io::{stdin, stdout, BufRead, Write};

use deoxy::{Pump, PumpDirection};

fn next(from: Option<PumpDirection>) -> Option<PumpDirection> {
    match from {
        Some(PumpDirection::Forward) => Some(PumpDirection::Backward),
        Some(PumpDirection::Backward) => None,
        None => Some(PumpDirection::Forward),
    }
}

fn main() {
    pretty_env_logger::init();
    let mut pump = Pump::try_new([24, 25, 5, 6]).unwrap();
    pump.stop().unwrap();
    let mut direction = None;
    println!("Press return to cycle pump.");
    stdout().lock().flush().unwrap();
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    loop {
        stdin.read_line(&mut buf).unwrap();
        direction = next(direction);
        pump.set_direction(direction).unwrap();
    }
}
