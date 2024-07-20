use macroquad::{
    camera::{set_camera, Camera2D},
    color::{DARKBLUE, GREEN, WHITE},
    math::Rect,
    miniquad::window,
    shapes::{draw_arc, draw_line, draw_rectangle_ex, DrawRectangleParams},
    window::{clear_background, next_frame},
};
use track::generate_network;
use track_shape::TrackShape;

mod minivec;
mod track;
mod track_shape;

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
                TrackShape::Line { source, .. } => draw_line(
                    source.x,
                    source.y,
                    curve.destination.x,
                    curve.destination.y,
                    1.0,
                    DARKBLUE,
                ),
                TrackShape::Arc {
                    start_angle,
                    angle_diff,
                    radius,
                    center,
                } => {
                    let (start_angle, angle_diff) = if angle_diff < 0. {
                        (start_angle + angle_diff, -angle_diff)
                    } else {
                        (start_angle, angle_diff)
                    };

                    draw_arc(
                        center.x,
                        center.y,
                        40,
                        radius + 0.5,
                        start_angle.to_degrees(),
                        1.0,
                        angle_diff.to_degrees(),
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
