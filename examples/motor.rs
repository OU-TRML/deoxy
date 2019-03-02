use std::time::Duration;

use deoxy::{Motor, MotorMessage};

use actix_web::actix::*;
use futures::future::Future;
use log::*;

fn main() {
    pretty_env_logger::init();
    let motor = Motor::try_new(
        Duration::new(1, 0),
        Duration::from_millis(250)..=Duration::from_millis(750),
        12,
    )
    .unwrap();
    let open = MotorMessage::Open;
    let close = MotorMessage::Close;
    let system = System::new("motor");
    let address = motor.start();
    let result = address.send(open);
    Arbiter::spawn(
        result
            .and_then(move |_| {
                std::thread::sleep(Duration::new(1, 0));
                address.send(close).map(|_| {})
            })
            .map_err(|err| {
                debug!("Got error: {:?}", err);
            }),
    );
    system.run();
}
