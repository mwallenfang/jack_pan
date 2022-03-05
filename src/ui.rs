use std::sync::atomic::Ordering;
use vizia::{Application, Context, Event, HStack, Knob, Model, WindowDescription, Lens, Label, VStack};
use crate::PAN_ATOMIC;

const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct UIData {
    pan: f32
}

pub enum UIEvents {
    PanChange(f32)
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(gain_event) = event.message.downcast() {
            match gain_event {
                UIEvents::PanChange(n) => {
                    self.pan = *n;
                    PAN_ATOMIC.store(*n, Ordering::Relaxed);
                }
            }
        }
    }
}

pub fn ui() {
    let window_description = WindowDescription::new()
        .with_inner_size(300,300)
        .with_title("jack_gain");

    Application::new(window_description, move |cx| {
        UIData{pan: 0.5}.build(cx);

        cx.add_theme(STYLE);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Knob::new(cx, 0.5, UIData::pan, true)
                    .on_changing(move |cx, val| cx.emit(UIEvents::PanChange(val)));
                Label::new(cx, UIData::pan);
            });
        });
    }).run();
}