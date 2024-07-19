use std::{default, f32::consts::PI};

use glam::Vec2;
use macroquad::{
    camera::{push_camera_state, set_camera, Camera2D},
    color::{Color, DARKBLUE, GREEN, WHITE, YELLOW},
    math::Rect,
    miniquad::window,
    shapes::{draw_arc, draw_circle, draw_line, draw_rectangle_ex, DrawRectangleParams},
    window::{clear_background, next_frame},
};
use track::{generate_network, TrackShape};

mod minivec;
mod track;

#[macroquad::main("Crates")]
async fn main() {
    let mut network = generate_network();

    let size = 64.0f32;

    loop {
        let delta_time = macroquad::time::get_frame_time();

        network.update(delta_time);

        let screen_size = window::screen_size();
        let aspect = screen_size.0 / screen_size.1;

        clear_background(WHITE);

        set_camera(&Camera2D::from_display_rect(Rect::new(
            -size * 0.5 * aspect,
            -size * 0.5,
            size * aspect,
            size,
        )));

        for curve in network.curves() {
            match curve.shape {
                TrackShape::Line => draw_line(
                    curve.source.x,
                    curve.source.y,
                    curve.destination.x,
                    curve.destination.y,
                    1.0,
                    DARKBLUE,
                ),
                TrackShape::Arc {
                    start_angle,
                    end_angle,
                    radius,
                    center,
                } => {
                    // let (start_angle, end_angle) = (
                    //     start_angle.min(end_angle),
                    //     start_angle.max(end_angle)
                    // );

                    let arc = ((end_angle - start_angle + PI * 2.) % (PI * 2.)).to_degrees();

                    draw_arc(
                        center.x,
                        center.y,
                        40,
                        radius + 0.5,
                        start_angle.to_degrees(),
                        1.0,
                        arc,
                        DARKBLUE,
                    );
                }
            }
        }

        for (train, angle) in network.train_positions() {
            draw_rectangle_ex(
                train.x,
                train.y,
                1.5,
                1.0,
                DrawRectangleParams {
                    color: GREEN,
                    rotation: angle,
                    offset: macroquad::math::Vec2::new(0.75, 0.5),
                    ..Default::default()
                },
            );
        }

        next_frame().await
    }
}
