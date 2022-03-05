use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use crate::{ALGO_SELECTION, PAN_ATOMIC};
use vizia::*;
const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct UIData {
    pan: f32,
    algorithm: Options
}

pub enum UIEvents {
    PanChange(f32),
    AlgoChange(Options)
}

#[derive(Debug, Data, Clone, Copy, PartialEq)]
pub enum Options {
    Linear = 0,
    ConstantPower = 1,
    Db45 = 2
}

impl Model for UIData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(pan_event) = event.message.downcast() {
            match pan_event {
                UIEvents::PanChange(n) => {
                    self.pan = *n;
                    PAN_ATOMIC.store(*n, Ordering::Relaxed);
                },
                UIEvents::AlgoChange(opt) => {
                    self.algorithm = *opt;
                    ALGO_SELECTION.store(*opt as u8, Relaxed);
                }
            }
        }
    }
}

pub fn ui() {
    let window_description = WindowDescription::new()
        .with_inner_size(400,300)
        .with_title("jack_gain");

    Application::new(window_description, move |cx| {
        UIData{pan: 0.5, algorithm: Options::Linear}.build(cx);

        cx.add_theme(STYLE);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Knob::new(cx, 0.5, UIData::pan, true)
                    .on_changing(move |cx, val| cx.emit(UIEvents::PanChange(val)));
                Label::new(cx, UIData::pan);
            })
            .row_between(Pixels(10.0))
            .child_space(Stretch(1.0));

            VStack::new(cx , |cx| {
                Dropdown::new(
                    cx,
                    |cx| {
                        Label::new(cx, "Pan Law")
                            .background_color(Color::from("#292628"))
                            .width(Pixels(80.0))
                    },
                    |cx| {
                        Picker::new(
                            cx, UIData::algorithm,
                            |cx, value| {
                                let option = *value.get(cx);
                                picker_item(cx, "Linear", Options::Linear, option);
                                picker_item(cx, "Constant Power", Options::ConstantPower, option);
                                picker_item(cx, "4.5 db", Options::Db45, option);
                            }
                        );
                    }
                );
            })
            .row_between(Pixels(10.0))
            .child_space(Stretch(1.0));
        });
    }).run();
}

/// https://github.com/vizia/vizia/blob/main/examples/controls/picker.rs
pub fn picker_item(cx: &mut Context, text: &'static str, option: Options, current: Options) {
    Button::new(
        cx,
        move |cx| cx.emit(UIEvents::AlgoChange(option.clone())),
        move |cx| {
            Label::new(cx, text)
                .color(Color::from("#ffffff"))
                .background_color(if current == option { Color::from("#424242") } else { Color::from("#292628") })
        },
    )
    .background_color(if current == option { Color::from("#424242") } else { Color::from("#292628") });
}