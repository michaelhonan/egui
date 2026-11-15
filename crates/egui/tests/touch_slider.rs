//! On a touch screen a finger that lands on a slider may be scrolling past it: only a drag that
//! sets off along the slider moves it, and a `DragValue` takes taps only (a tap edits it).

use egui::{
    Context, Event, PointerButton, Pos2, RawInput, Rect, TouchDeviceId, TouchId, TouchPhase, vec2,
};

struct Screen {
    ctx: Context,
    time: f64,
    value: f64,
    slider: Rect,
}

impl Screen {
    fn new() -> Self {
        let mut s = Self {
            ctx: Context::default(),
            time: 0.0,
            value: 50.0,
            slider: Rect::NOTHING,
        };
        s.frame(vec![]);
        // a touch somewhere else first: egui knows it has a touch screen from then on
        let away = Pos2::new(390.0, 290.0);
        s.frame(finger(TouchPhase::Start, away));
        s.frame(finger(TouchPhase::End, away));
        s
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 1.0 / 60.0;
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut value = self.value;
        let mut slider = Rect::NOTHING;
        let mut out = self.ctx.run_ui(raw, |ui| {
            slider = ui.add(egui::Slider::new(&mut value, 0.0..=100.0)).rect;
        });
        out.textures_delta.clear();
        self.value = value;
        self.slider = slider;
    }

    fn drag(&mut self, from: Pos2, by: egui::Vec2) {
        self.frame(finger(TouchPhase::Start, from));
        let mut p = from;
        for _ in 0..8 {
            p += by / 8.0;
            self.frame(finger(TouchPhase::Move, p));
        }
        self.frame(finger(TouchPhase::End, p));
    }
}

fn finger(phase: TouchPhase, pos: Pos2) -> Vec<Event> {
    let touch = Event::Touch {
        device_id: TouchDeviceId(1),
        id: TouchId(1),
        phase,
        pos,
        force: None,
    };
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    match phase {
        TouchPhase::Start => vec![touch, Event::PointerMoved(pos), button(true)],
        TouchPhase::Move => vec![touch, Event::PointerMoved(pos)],
        _ => vec![touch, button(false), Event::PointerGone],
    }
}

#[test]
fn a_finger_moving_up_or_down_over_a_slider_leaves_it() {
    let mut s = Screen::new();
    let on = s.slider.left_center() + vec2(20.0, 0.0);
    s.drag(on, vec2(2.0, 60.0));
    assert_eq!(
        s.value, 50.0,
        "a press and a drag across the slider don't set it"
    );
    s.drag(on, vec2(60.0, 2.0));
    assert_ne!(s.value, 50.0, "a drag along it moves it");
}
