use yew::html;
use yew::prelude::*;

use deoxy_core::Step as CStep;

use uom::si::{f32::*, volume::liter};

use std::cell::RefCell;
use std::rc::Rc;

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
pub struct Step(Option<CStep>, Vec<Buffer>);
#[derive(Default)]
struct Protocol {
    steps: Rc<RefCell<Vec<Step>>>,
}

impl Component for Protocol {
    type Message = ();
    type Properties = ProtocolProps;
    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { steps: props.steps }
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
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
            { for steps.iter().map(|s| s.view()) }
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
        let (id, time) = if let Some(step) = self.0 {
            panic!();
            let CStep::Perfuse(id, time) = step;
            (Some(id), time)
        } else {
            (None, None)
        };
        let id = id.unwrap_or_default();
        let c = format!(
            "color {}",
            match (id + 1) {
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
        html! {
            <li>
                <span class=("verb", "perfuse"),>{"Perfuse"}</span>
                {" with "}
                <select class=c,>
                { for self.1.iter().filter(|buf| !buf.label.is_empty()).map(|buf| html! {
                    <option value=buf.index,>{&buf.label}</option>
                })}
                </select>
                {" for "}
                <input type={"number"}, class={"time"}, value=10, min=1, />
                {" "}
                <span class="time",>{"minutes"}</span>
                {"."}
            </li>
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
        let steps = vec![Step(None, vec![])];
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
                    let buffers = buffers.clone();
                    let mut steps = loop {
                        let steps = self.steps.try_borrow_mut();
                        if let Ok(steps) = steps {
                            break steps;
                        }
                    };
                    for step in steps.iter_mut() {
                        step.1 = buffers.clone().to_vec();
                    }
                    true
                }
                BufferMessage::Ignore => false,
            },
            Message::Protocol(_msg) => false,
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
