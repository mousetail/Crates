use std::f32::consts::TAU;

use glam::Vec2;
use macroquad::{color::Color, shapes::draw_line};

pub fn draw_arc(
    center: Vec2,
    radius: f32,
    from_angle: f32,
    span: f32,
    steps: u8,
    width: f32,
    color: Color,
) {
    macroquad::shapes::draw_arc(
        center.x,
        center.y,
        steps,
        radius - width / 2.0,
        from_angle.to_degrees(),
        width,
        span.to_degrees(),
        color,
    )
    // let steps = (steps as f32 * span / TAU + 1.0).ceil().max(2.0);
    // let step_span = span / (steps - 1.0);

    // for i in 0..steps as usize-1 {
    //     let angle_before = from_angle + i as f32 * step_span;
    //     let angle_after = angle_before + step_span;

    //     let line_source = center + Vec2::from_angle(angle_before) * radius;
    //     let line_destination = center + Vec2::from_angle(angle_after) * radius;

    //     draw_line(line_source.x, line_source.y, line_destination.x, line_destination.y, width, color);
    // }
}
