use macroquad::{
    camera::{set_camera, Camera2D},
    color::{Color, BLACK, DARKBLUE, DARKGRAY, GRAY, GREEN, LIGHTGRAY, WHITE},
    math::Rect,
    miniquad::{window, Context},
    shapes::{draw_line, draw_rectangle_ex, DrawRectangleParams},
    window::{clear_background, next_frame},
};
use proper_draw_arc::draw_arc;
use track::{generate_network, Network};
use track_shape::TrackShape;

mod minivec;
mod proper_draw_arc;
mod track;
mod track_shape;

fn draw_all_arcs(network: &Network, thickness: f32, color: Color) {
    for curve in network.curves() {
        match curve.shape {
            TrackShape::Line { source, .. } => draw_line(
                source.x,
                source.y,
                curve.destination.x,
                curve.destination.y,
                thickness,
                color,
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
                    center,
                    radius,
                    start_angle,
                    angle_diff,
                    40,
                    thickness,
                    color,
                );
            }
        }
    }
}

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Crates".to_owned(),
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
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

        draw_all_arcs(&network, 1.0, DARKBLUE);
        draw_all_arcs(&network, 0.8, Color::from_hex(0xFFFFFF));
        draw_all_arcs(&network, 0.1, DARKBLUE);

        for (train, angle) in network.train_positions() {
            draw_rectangle_ex(
                train.x,
                train.y,
                1.7,
                1.2,
                DrawRectangleParams {
                    color: GREEN,
                    rotation: angle,
                    offset: macroquad::math::Vec2::new(0.5, 0.5),
                    ..Default::default()
                },
            );

            draw_rectangle_ex(
                train.x,
                train.y,
                1.5,
                1.0,
                DrawRectangleParams {
                    color: WHITE,
                    rotation: angle,
                    offset: macroquad::math::Vec2::new(0.5, 0.5),
                    ..Default::default()
                },
            );
        }

        next_frame().await
    }
}
