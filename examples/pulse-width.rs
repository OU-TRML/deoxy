use deoxy::{actix::*, Pin, Pwm};
use futures::future::Future;
use std::str::FromStr;
use std::{
    io::{stdin, stdout, Write},
    ops::Deref,
    time::Duration,
};
use termion::input::TermRead;

#[derive(Default)]
struct Prompt;

impl Prompt {
    fn prompt<S, T>(&self, message: S) -> Option<T>
    where
        T: FromStr,
        S: Deref<Target = str>,
    {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        let stdin = stdin();
        let mut stdin = stdin.lock();
        stdout.write_all(message.to_string().as_bytes()).ok()?;
        stdout.write_all(b": ").ok()?;
        stdout.flush().ok()?;
        let input = stdin.read_line().ok()??;
        input.parse::<T>().ok()
    }
}

enum PromptMessage {
    Start,
    GetPeriod { pin: u16 },
    GetInitial { pin: u16, period: Duration },
    GetNext { motor: Addr<Motor> },
}

impl Actor for Prompt {
    type Context = Context<Self>;
    fn started(&mut self, context: &mut Self::Context) {
        context.notify(PromptMessage::Start);
    }
}

impl ActixMessage for PromptMessage {
    type Result = ();
}

impl Handle<PromptMessage> for Prompt {
    type Result = ();
    fn handle(&mut self, message: PromptMessage, context: &mut Self::Context) -> Self::Result {
        match message {
            PromptMessage::Start => {
                let pin: Option<u16> = self.prompt("Pin number");
                if let Some(pin) = pin {
                    context.notify(PromptMessage::GetPeriod { pin });
                } else {
                    context.notify(PromptMessage::Start);
                }
            }
            PromptMessage::GetPeriod { pin } => {
                let period: Option<u64> = self.prompt("Duty cycle/period (ms)");
                if let Some(period) = period {
                    context.notify(PromptMessage::GetInitial {
                        pin,
                        period: Duration::from_millis(period),
                    });
                } else {
                    context.notify(PromptMessage::GetPeriod { pin });
                }
            }
            PromptMessage::GetInitial { pin, period } => {
                let start: Option<u64> = self.prompt("Start position (µs)");
                if let Some(start) = start {
                    let motor = Motor::try_new(pin, period, context.address())
                        .map(|mut motor| {
                            motor.pulse_width = Duration::from_micros(start);
                            motor
                        })
                        .unwrap();
                    let _addr = Arbiter::start(move |_| motor);
                } else {
                    context.notify(PromptMessage::GetInitial { pin, period });
                }
            }
            PromptMessage::GetNext { motor } => {
                let s: Option<String> = self.prompt("Set pulse width [number/+/-/^C]");
                if let Some(s) = s {
                    let pulse_width = match s.as_str() {
                        "+" => motor.send(MotorMessage::Increase),
                        "-" => motor.send(MotorMessage::Decrease),
                        x => {
                            if let Ok(width) = x.parse::<u64>() {
                                motor.send(MotorMessage::Set(Duration::from_micros(width)))
                            } else {
                                motor.send(MotorMessage::Get)
                            }
                        }
                    };
                    let pulse_width = pulse_width.wait().unwrap();
                    println!("Current pulse width: {} µs", pulse_width);
                    context.notify(PromptMessage::GetNext { motor });
                }
            }
        }
    }
}

struct Motor {
    pin: Pin,
    period: Duration,
    pulse_width: Duration,
    prompt: Addr<Prompt>,
}

impl Motor {
    fn try_new(pin: u16, period: Duration, prompt: Addr<Prompt>) -> Option<Self> {
        let pin = Pin::try_new(pin).ok()?;
        Some(Self {
            pin,
            period,
            pulse_width: Duration::default(),
            prompt,
        })
    }
}

impl Actor for Motor {
    type Context = Context<Self>;
    fn started(&mut self, context: &mut Self::Context) {
        self.prompt.do_send(PromptMessage::GetNext {
            motor: context.address(),
        });
    }
}

impl ActixMessage for MotorMessage {
    type Result = u64;
}

enum MotorMessage {
    Increase,
    Decrease,
    Set(Duration),
    Get,
}

impl Handle<MotorMessage> for Motor {
    type Result = u64;
    fn handle(&mut self, message: MotorMessage, context: &mut Self::Context) -> Self::Result {
        match message {
            MotorMessage::Increase => {
                self.pulse_width += Duration::from_micros(1);
            }
            MotorMessage::Decrease => {
                self.pulse_width -= Duration::from_micros(1);
            }
            MotorMessage::Set(duration) => {
                self.pulse_width = duration;
            }
            MotorMessage::Get => {}
        };
        self.pin.set_pwm(self.period, self.pulse_width).unwrap();
        // Stop signaling the motor after five seconds
        context.run_later(Duration::new(5, 0), move |motor, _| {
            motor
                .pin
                .set_pwm(motor.period, Duration::new(0, 0))
                .unwrap();
        });
        self.pulse_width.as_micros() as u64
    }
}

fn main() {
    let system = System::new("pulse-width");
    let prompt = Prompt;
    let _ = prompt.start();
    system.run();
}
