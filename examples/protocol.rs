use std::error::Error;
use std::time::Duration;

use deoxy::{actix::*, Config, CoordMessage, Coordinator, MotorConfig, Protocol, PumpConfig, Step};

macro_rules! motor {
    ($pin:expr) => {
        MotorConfig {
            label: None,
            period: Duration::from_millis(50),
            pin: $pin,
            range: [Duration::from_millis(1), Duration::from_millis(100)],
        }
    };
}

macro_rules! secs {
    ($s:expr) => {
        Some(Duration::new($s, 0))
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let config = Config {
        pump: PumpConfig {
            pins: [24, 25, 5, 6],
            invert: false,
        },
        motors: vec![motor!(4), motor!(27), motor!(21), motor!(13)],
    };
    let proto = Protocol {
        steps: vec![
            Step::Perfuse(0, secs!(5)),
            Step::Perfuse(1, secs!(10)),
            Step::Perfuse(2, secs!(5)),
            Step::Perfuse(3, None),
        ],
    };
    let coord = Coordinator::try_new(config)?;
    let system = System::new("deoxy-protocol-example");
    let addr = coord.start();
    addr.do_send(CoordMessage::Start(proto, None));
    system.run();
    Ok(())
}
