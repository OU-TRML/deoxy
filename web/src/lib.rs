use yew::html;
use yew::prelude::*;

use deoxy_core::Step as CStep;

use uom::si::{f32::*, volume::liter};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod messages;
use self::messages::*;
mod buffers;
use self::buffers::{Buffer, Buffers};

const SLOTS: usize = 10;
const WASTE: usize = 1;
pub const BUFFERS: usize = SLOTS - WASTE;
pub const VOLUME_LITERS: f32 = 0.5;

fn reaction_volume() -> Volume {
    Volume::new::<liter>(VOLUME_LITERS)
}

#[derive(Clone, Default, PartialEq)]
pub struct ProtocolProps {
    pub steps: Rc<RefCell<Vec<Step>>>,
    pub buffers: Rc<RefCell<[Buffer; BUFFERS]>>,
    pub onchange: Option<Callback<ProtocolMessage>>,
}

#[derive(Default, PartialEq)]
pub struct Step(usize, Option<CStep>, Vec<Buffer>);
#[derive(Default)]
struct Protocol {
    steps: Rc<RefCell<Vec<Step>>>,
    onchange: Option<Callback<ProtocolMessage>>,
}

impl Component for Protocol {
    type Message = ProtocolMessage;
    type Properties = ProtocolProps;
    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            steps: props.steps,
            onchange: props.onchange,
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(ref mut onchange) = self.onchange {
            onchange.emit(msg);
        }
        true
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.steps = props.steps;
        true
    }
}

impl Renderable<Self> for Protocol {
    fn view(&self) -> Html<Self> {
        let steps = loop {
            let steps = self.steps.try_borrow();
            if let Ok(steps) = steps {
                break steps;
            }
        };
        html! {
            <div id={"protocol"},>
            <h1>{"Protocol"}</h1>
            <ol>
            { for steps.iter().map(Renderable::view) }
            </ol>
            <input type={"button"}, id={"start"}, value={"Start"}, />
            </div>
        }
    }
}

impl Component for Step {
    type Message = ();
    type Properties = ();
    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self::default()
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }
}

impl Renderable<Protocol> for Step {
    fn view(&self) -> Html<Protocol> {
        let index = self.0;
        let real = self
            .2
            .iter()
            .filter(|buf| !buf.label.is_empty())
            .collect::<Vec<_>>();
        if real.is_empty() {
            html! {
                <li>{"Add a buffer!"}</li>
            }
        } else {
            let (id, time) = if let Some(step) = self.1 {
                let CStep::Perfuse(id, time) = step;
                (Some(id), time)
            } else {
                (None, None)
            };
            let chosen = id.is_some();
            let id = id.unwrap_or(3);
            let time = if let Some(time) = time {
                let secs = time.as_secs();
                let mins = secs / 60;
                format!("{}", mins)
            } else {
                "".to_string()
            };
            let c = format!(
                "color {}",
                match id + 1 {
                    1 => "one",
                    2 => "two",
                    3 => "three",
                    4 => "four",
                    5 => "five",
                    6 => "six",
                    7 => "seven",
                    8 => "eight",
                    9 => "nine",
                    10 => "ten",
                    _ => "",
                }
            );
            let selected = move |pos: usize| {
                move |event: yew::html::ChangeData| match event {
                    yew::html::ChangeData::Select(sel) => {
                        if let Some(val) = sel.value() {
                            ProtocolMessage::Selected(index, pos, val)
                        } else {
                            ProtocolMessage::Ignore
                        }
                    }
                    _ => ProtocolMessage::Ignore,
                }
            };
            let input = move |event: yew::html::ChangeData| match event {
                yew::html::ChangeData::Value(val) => ProtocolMessage::Input(index, 0, val),
                _ => ProtocolMessage::Ignore,
            };
            let sel = if chosen {
                html! {
                    <select class=c, onchange=|e| (selected(0))(e), >
                    { for self.2.iter().filter(|buf| !buf.label.is_empty()).map(|buf| html! {
                        <option value=buf.index,>{&buf.label}</option>
                    })}
                    </select>
                }
            } else {
                html! {
                    <select class=c, onchange=|e| (selected(0))(e), >
                    <option disabled=true,></option>
                    { for self.2.iter().filter(|buf| !buf.label.is_empty()).map(|buf| html! {
                        <option value=buf.index,>{&buf.label}</option>
                    })}
                    </select>
                }
            };
            html! {
                <li>
                    <span class=("verb", "perfuse"),>{"Perfuse"}</span>
                    {" with "}
                    { sel }
                    {" for "}
                    <input type="number", class="time", min=1, value=time, onchange=|e| input(e), />
                    {" "}
                    <span class="time",>{"minutes"}</span>
                    {"."}
                </li>
            }
        }
    }
}

struct Root {
    buffers: Rc<RefCell<[Buffer; BUFFERS]>>,
    steps: Rc<RefCell<Vec<Step>>>,
}

impl Default for Root {
    fn default() -> Self {
        let mut buffers: [Buffer; BUFFERS] = Default::default();
        for (i, buf) in buffers.iter_mut().enumerate() {
            buf.index = i;
        }
        let buffers = Rc::new(RefCell::new(buffers));
        let steps = vec![Step(0, None, vec![])];
        let steps = Rc::new(RefCell::new(steps));
        Self { buffers, steps }
    }
}

impl Component for Root {
    type Message = Message;
    type Properties = ();
    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self::default()
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Buffer(msg) => match msg {
                BufferMessage::Input(index, label) => {
                    let mut buffers = loop {
                        let buffers = self.buffers.try_borrow_mut();
                        if let Ok(buffers) = buffers {
                            break buffers;
                        }
                    };
                    buffers[index].label = label;
                    let buffers = buffers.clone().to_vec();
                    let mut steps = loop {
                        let steps = self.steps.try_borrow_mut();
                        if let Ok(steps) = steps {
                            break steps;
                        }
                    };
                    for step in steps.iter_mut() {
                        step.2 = buffers.clone();
                    }
                    true
                }
                BufferMessage::Ignore => false,
            },
            Message::Protocol(msg) => match msg {
                ProtocolMessage::Selected(row, _pos, val) => {
                    let id = val.parse::<usize>().unwrap();
                    let mut steps = loop {
                        let steps = self.steps.try_borrow_mut();
                        if let Ok(steps) = steps {
                            break steps;
                        }
                    };
                    let CStep::Perfuse(_, time) =
                        steps[row].1.unwrap_or_else(|| CStep::Perfuse(0, None));
                    steps[row].1 = Some(CStep::Perfuse(id, time));
                    true
                }
                ProtocolMessage::Input(row, _pos, val) => {
                    let mut steps = loop {
                        let steps = self.steps.try_borrow_mut();
                        if let Ok(steps) = steps {
                            break steps;
                        }
                    };
                    let CStep::Perfuse(id, _) =
                        steps[row].1.unwrap_or_else(|| CStep::Perfuse(0, None));
                    steps[row].1 = Some(CStep::Perfuse(
                        id,
                        Some(Duration::from_secs(60 * val.parse::<u64>().unwrap())),
                    ));
                    let mut next = Step::default();
                    next.2 = steps.last().map(|s| s.2.clone()).unwrap_or_default();
                    steps.push(next);
                    for (i, s) in steps.iter_mut().enumerate() {
                        s.0 = i;
                    }
                    true
                }
                ProtocolMessage::Ignore => false,
            },
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        // The root never expects to have any properties, so ignore everything
        false
    }
}

impl Renderable<Self> for Root {
    fn view(&self) -> Html<Self> {
        html! {
            <>
            <Buffers: onchange=|e: BufferMessage| e.into(), buffers=self.buffers.clone(), />
            <Protocol: onchange=|e: ProtocolMessage| e.into(), steps=self.steps.clone(), buffers=self.buffers.clone(), />
            </>
        }
    }
}

/// Initializes an app and the driving run loop.
pub fn run() {
    yew::initialize();
    App::<Root>::new().mount_to_body();
    yew::run_loop();
}
