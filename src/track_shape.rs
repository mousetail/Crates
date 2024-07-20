use std::f32::consts::{FRAC_PI_2, PI};

use glam::Vec2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackShape {
    Line {
        source: Vec2,
        direction: Vec2,
        length: f32,
    },
    Arc {
        start_angle: f32,
        angle_diff: f32,
        radius: f32,
        center: Vec2,
    },
}

impl TrackShape {
    pub fn get_transform_at_distance(&self, distance: f32) -> (Vec2, f32) {
        match self {
            TrackShape::Line {
                source, direction, ..
            } => (*source + *direction * distance, direction.to_angle()),
            TrackShape::Arc {
                start_angle,
                radius,
                center,
                angle_diff,
            } => {
                let angle = distance * angle_diff.signum() / radius + start_angle;

                (
                    *center + Vec2::from_angle(angle) * *radius,
                    angle + FRAC_PI_2,
                )
            }
        }
    }

    pub fn get_length(&self) -> f32 {
        match self {
            TrackShape::Line { length, .. } => *length,
            TrackShape::Arc {
                angle_diff, radius, ..
            } => (*angle_diff * *radius).abs(),
        }
    }

    pub fn subshape(&self, from: f32, to: f32) -> TrackShape {
        match self {
            TrackShape::Line {
                source, direction, ..
            } => TrackShape::Line {
                source: *source + *direction * from,
                direction: *direction,
                length: (to - from),
            },
            TrackShape::Arc {
                start_angle,
                radius,
                center,
                angle_diff,
            } => TrackShape::Arc {
                start_angle: start_angle + from / radius * angle_diff.signum(),
                angle_diff: (to - from) / radius * angle_diff.signum(),
                radius: *radius,
                center: *center,
            },
        }
    }

    pub fn normalize(self) -> TrackShape {
        match self {
            TrackShape::Line {
                source,
                direction,
                length,
            } => TrackShape::Line {
                source,
                direction,
                length,
            },
            TrackShape::Arc {
                start_angle,
                angle_diff,
                radius,
                center,
            } => {
                if radius < 0. {
                    TrackShape::Arc {
                        start_angle: start_angle + PI,
                        angle_diff,
                        radius: -radius,
                        center,
                    }
                } else {
                    TrackShape::Arc {
                        start_angle,
                        angle_diff,
                        radius,
                        center,
                    }
                }
            }
        }
    }
}
