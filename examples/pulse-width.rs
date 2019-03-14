use deoxy::{actix::*, Pin};
use std::str::FromStr;
use std::{
    io::{stdin, stdout, Write},
    ops::Deref,
    time::Duration,
};
use termion::input::TermRead;

fn prompt<T, S>(
    stdout: &mut std::io::StdoutLock,
    stdin: &mut std::io::StdinLock,
    message: S,
) -> Option<T>
where
    T: FromStr,
    S: Deref<Target = str>,
{
    stdout.write_all(message.to_string().as_bytes()).ok()?;
    stdout.write_all(b": ").ok()?;
    stdout.flush().ok()?;
    let input = stdin.read_line().ok()??;
    input.parse::<T>().ok()
}

struct Motor {
    pin: Pin,
    period: Duration,
    pulse_width: Duration,
}

impl Motor {
    fn try_new(pin: u16, period: Duration) -> Option<Self> {
        let pin = Pin::try_new(pin).ok()?;
        Some(Self {
            pin,
            period,
            pulse_width: Duration::default(),
        })
    }
}

impl Actor for Motor {
    type Context = Context<Self>;
    fn started(&mut self, context: &mut Self::Context) {
        context.run_interval(self.period, |motor, context| {
            motor.pin.set_high().unwrap();
            context.run_later(motor.pulse_width, |motor, _context| {
                motor.pin.set_low().unwrap();
            });
        });
        context.notify(Message::GetNext)
    }
}

impl ActixMessage for Message {
    type Result = ();
}

enum Message {
    Increase,
    Decrease,
    Set(Duration),
    GetNext,
}

impl Handle<Message> for Motor {
    type Result = ();
    fn handle(&mut self, message: Message, context: &mut Self::Context) {
        match message {
            Message::Increase => {
                self.pulse_width += Duration::from_micros(1);
            }
            Message::Decrease => {
                self.pulse_width -= Duration::from_micros(1);
            }
            Message::Set(duration) => {
                self.pulse_width = duration;
            }
            Message::GetNext => {
                let s: Option<String> = prompt(
                    &mut stdout().lock(),
                    &mut stdin().lock(),
                    "Set pulse width [number/+/-/^C]",
                );
                if let Some(s) = s {
                    match s.as_str() {
                        "+" => context.notify(Message::Increase),
                        "-" => context.notify(Message::Decrease),
                        x => {
                            if let Ok(width) = x.parse::<u64>() {
                                context.notify(Message::Set(Duration::from_micros(width)));
                            }
                        }
                    }
                }
                return;
            }
        };
        let stdout = stdout();
        let mut stdout = stdout.lock();
        stdout
            .write_all(
                format!(
                    "Current pulse width: {} µs\n",
                    self.pulse_width.as_micros()
                )
                .as_bytes(),
            )
            .unwrap();
        stdout.flush().unwrap();
        context.notify(Message::GetNext);
    }
}

fn motor(stdout: &mut std::io::StdoutLock, stdin: &mut std::io::StdinLock) -> Option<Motor> {
    let pin: u16 = prompt(stdout, stdin, "Pin number")?;
    let period: u64 = prompt(stdout, stdin, "Duty cycle/period (ms)")?;
    let period = Duration::from_millis(period);
    let start: u64 = prompt(stdout, stdin, "Start position (µs)")?;
    Motor::try_new(pin, period).map(|mut motor| {
        motor.pulse_width = Duration::from_micros(start);
        motor
    })
}

fn main() {
    let motor = motor(&mut stdout().lock(), &mut stdin().lock()).unwrap();
    let system = System::new("pulse-width");
    let _address = motor.start();
    system.run();
}
