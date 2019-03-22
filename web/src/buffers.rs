use yew::html;
use yew::prelude::*;

use uom::{
    fmt::DisplayStyle,
    si::{f32::*, volume::liter},
};

use std::{cell::RefCell, rc::Rc};

use crate::messages::*;
use crate::Step;
use crate::BUFFERS;

#[derive(Clone, PartialEq)]
pub struct BuffersProps {
    pub onchange: Option<Callback<BufferMessage>>,
    pub buffers: Rc<RefCell<[Buffer; BUFFERS]>>,
}

#[derive(Clone, Default)]
pub struct Buffers {
    spec: BuffersProps,
}

impl Default for BuffersProps {
    fn default() -> Self {
        let mut buffers: [Buffer; BUFFERS] = Default::default();
        for (i, buf) in buffers.iter_mut().enumerate() {
            buf.index = i;
        }
        Self {
            buffers: Rc::new(RefCell::new(buffers)),
            onchange: None,
        }
    }
}

impl Component for Buffers {
    type Message = BufferMessage;
    type Properties = BuffersProps;
    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { spec: props }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(ref mut onchange) = self.spec.onchange {
            onchange.emit(msg);
        }
        true
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.spec = props;
        true
    }
}

impl Renderable<Self> for Buffers {
    fn view(&self) -> Html<Self> {
        let buffers = loop {
            let buffers = self.spec.buffers.try_borrow();
            if let Ok(buffers) = buffers {
                break buffers;
            }
        };
        html! {
            <div id={"buffers"},>
            <h1>{"Buffers"}</h1>
            <table>
                <tr><th>{"Index"}</th><th>{"Label"}</th><th>{"Volume"}</th></tr>
                { for buffers.iter().map(|b| b.view()) }
            </table>
            </div>
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Buffer {
    pub(crate) label: String,
    pub(crate) index: usize,
    pub(crate) volume: Option<Volume>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BufferProps {
    pub(crate) label: Option<String>,
    pub(crate) index: usize,
    pub(crate) volume: Option<Volume>,
}

impl Buffer {
    fn new(index: usize) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }
}

impl Component for Buffer {
    type Message = ();
    type Properties = BufferProps;
    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            label: props.label.unwrap_or_default(),
            index: props.index,
            volume: props.volume,
        }
    }
    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.label = props.label.unwrap_or_default();
        self.index = props.index;
        self.volume = props.volume;
        true
    }
}

impl Renderable<Buffers> for Buffer {
    fn view(&self) -> Html<Buffers> {
        let index = self.index;
        let onchange = move |event: yew::html::ChangeData| {
            if let yew::html::ChangeData::Value(val) = event {
                BufferMessage::Input(index, val)
            } else {
                BufferMessage::Ignore
            }
        };
        let volume = if let Some(volume) = self.volume {
            let fmt = Volume::format_args(liter, DisplayStyle::Abbreviation);
            format!("{}", fmt.with(volume))
        } else {
            "".to_string()
        };
        html! {
            <tr class={"buffer"},>
                <td class={"index"},>{index + 1}</td>
                <td class={"label"},>
                    <input type={"text"},
                        name={"label[]"},
                        // placeholder={"Water"},
                        value=&self.label,
                        oninput=|e| BufferMessage::Input(index, e.value),
                        onchange=|e| onchange(e), />
                </td>
                <td>{volume}</td>
            </tr>
        }
    }
}
