//! #16806: Up and Down in the painting tools' % fields (Opacity, Flow, Smoothing; the Mixer Brush's
//! Wet, Load, Mix and Flow) step on every press. The brush keeps these as `f32` fractions, so 0.51
//! reads back as 50.999999..., which the field took for a value off its grid and stepped to 51 again.

use egui::accesskit::Role;
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

use crate::PhotocraftApp;
use crate::state::Tool;

/// The options bar of a new document with `tool`.
fn bar(tool: Tool) -> Harness<'static, Box<PhotocraftApp>> {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 80, "height": 80})).unwrap();
    app.ui.tool = tool;
    crate::paint_mouse::sync_tool_brush(&mut app);
    crate::theme::install_fonts(app.ctx());
    crate::theme::apply(app.ctx(), &crate::theme::AppTheme::pro());
    let mut h = Harness::builder().with_context(app.ctx().clone()).with_size(vec2(2400.0, 300.0)).with_step_dt(1.0 / 60.0).build_ui_state(
        move |ui, app: &mut Box<PhotocraftApp>| {
            crate::panels::options_bar(app, ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

/// The field named `label`, with or without its caption's colon.
fn field<'a>(h: &'a Harness<'static, Box<PhotocraftApp>>, label: &str) -> egui_kittest::Node<'a> {
    h.query_all_by_role(Role::SpinButton)
        .find(|n| n.accesskit_node().label().is_some_and(|l| l.trim_end_matches(':') == label))
        .unwrap_or_else(|| panic!("the {label} field"))
}

/// The value the field named `label` holds, to a billionth (the field's own grid tolerance), so
/// 50.999999 (an `f32` read as it is) stays apart from 51.
fn shown(h: &Harness<'static, Box<PhotocraftApp>>, label: &str) -> f64 {
    let value = field(h, label).accesskit_node().numeric_value().unwrap_or_else(|| panic!("the {label} field's value"));
    (value * 1e9).round() / 1e9
}

/// Clicks into the field named `label`, types `text` over it and presses Enter; the focus stays in it.
fn type_into(h: &mut Harness<'static, Box<PhotocraftApp>>, label: &str, text: &str) {
    let at = field(h, label).rect().center();
    h.hover_at(at);
    h.run_steps(1);
    h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(1);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(egui::Event::Text(text.into()));
    h.run_steps(1);
    h.key_press(Key::Enter);
    h.run_steps(3);
}

/// Presses `key` with `mods` `times` times, letting the bar redraw from the stored value in between.
fn press(h: &mut Harness<'static, Box<PhotocraftApp>>, key: Key, mods: Modifiers, times: usize) {
    for _ in 0..times {
        h.key_press_modifiers(mods, key);
        h.run_steps(3);
    }
}

/// From 50, Up x3 shows 53 and Down x3 (from 50 again) shows 47 in the field named `label`.
fn steps_every_press(tool: Tool, label: &str) {
    let mut h = bar(tool);
    type_into(&mut h, label, "50");
    assert_eq!(shown(&h, label), 50.0, "{tool:?} {label} typed 50");
    press(&mut h, Key::ArrowUp, Modifiers::NONE, 3);
    assert_eq!(shown(&h, label), 53.0, "{tool:?} {label}: 50, Up x3");
    type_into(&mut h, label, "50");
    press(&mut h, Key::ArrowDown, Modifiers::NONE, 3);
    assert_eq!(shown(&h, label), 47.0, "{tool:?} {label}: 50, Down x3");
}

#[test]
fn brush_opacity_flow_and_smoothing_step_on_every_press() {
    for label in ["Opacity", "Flow", "Smoothing"] {
        steps_every_press(Tool::Brush, label);
    }
}

#[test]
fn pencil_and_eraser_percent_fields_step_on_every_press() {
    steps_every_press(Tool::Pencil, "Opacity");
    steps_every_press(Tool::Pencil, "Smoothing");
    steps_every_press(Tool::Eraser, "Opacity");
    steps_every_press(Tool::Eraser, "Flow");
}

#[test]
fn mixer_brush_percent_fields_step_on_every_press() {
    for label in ["Wet", "Load", "Mix", "Flow"] {
        steps_every_press(Tool::MixerBrush, label);
    }
}

#[test]
fn shift_up_steps_ten_on_every_press() {
    let mut h = bar(Tool::Brush);
    type_into(&mut h, "Opacity", "50");
    press(&mut h, Key::ArrowUp, Modifiers::SHIFT, 2);
    assert_eq!(shown(&h, "Opacity"), 70.0, "50, Shift+Up x2");
    assert_eq!((h.state().session.tools.brush.opacity * 100.0).round(), 70.0, "the brush holds 70 %");
}

#[test]
fn an_f32_opacity_of_0_51_steps_up_twice_to_53() {
    let mut h = bar(Tool::Brush);
    type_into(&mut h, "Opacity", "40");
    h.state_mut().session.tools.brush.opacity = 0.51_f32;
    h.run_steps(3);
    assert_eq!(shown(&h, "Opacity"), 51.0);
    press(&mut h, Key::ArrowUp, Modifiers::NONE, 2);
    assert_eq!(shown(&h, "Opacity"), 53.0, "0.51 (f32), Up x2");
    assert_eq!((h.state().session.tools.brush.opacity * 100.0).round(), 53.0, "the brush holds 53 %");
}
