#[cfg(not(feature = "server"))]
use deoxy::Tui;

use futures::Future;
use std::time::Duration;

use deoxy::{actix::*, Config, CoordMessage, Coordinator, MotorConfig, Protocol, PumpConfig, Step};

fn main() {
    pretty_env_logger::init();

    let pump = PumpConfig {
        pins: [1, 2, 3, 4],
        invert: false,
    };
    let motor1 = MotorConfig {
        pin: 5,
        period: Duration::new(1, 0),
        range: [Duration::from_millis(500), Duration::from_millis(750)],
        label: None,
    };
    let motor2 = MotorConfig {
        pin: 6,
        period: Duration::new(1, 0),
        range: [Duration::from_millis(500), Duration::from_millis(750)],
        label: None,
    };
    let motor3 = MotorConfig {
        pin: 7,
        period: Duration::new(1, 0),
        range: [Duration::from_millis(500), Duration::from_millis(750)],
        label: None,
    };
    let motor4 = MotorConfig {
        pin: 8,
        period: Duration::new(1, 0),
        range: [Duration::from_millis(500), Duration::from_millis(750)],
        label: None,
    };
    let motors = vec![motor1, motor2, motor3, motor4];
    let config = Config {
        motors,
        pump,
        admins: vec![],
    };

    let step1 = Step::Perfuse(0, Some(Duration::new(5, 0)));
    let step2 = Step::Perfuse(1, None);
    let step3 = Step::Perfuse(3, Some(Duration::new(3, 0)));
    let step4 = Step::Perfuse(2, None);
    let steps = vec![step1, step2, step3, step4];
    let proto = Protocol { steps };

    let system = System::new("pause");

    let coord = Coordinator::try_new(config).unwrap().start();
    #[cfg(not(feature = "server"))]
    {
        let tui = Box::new(Tui {});
        coord.do_send(CoordMessage::Subscribe(tui));
    }
    coord.do_send(CoordMessage::Start(proto, None));
    system.run();
}
