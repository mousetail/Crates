use std::f32::consts::{FRAC_PI_2, PI, TAU};

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
    pub fn from_source_direction_dest(
        source: Vec2,
        source_direction: Vec2,
        destination: Vec2,
    ) -> TrackShape {
        let direction_to_center = source_direction.perp();
        let total_distance = destination - source;
        let y = total_distance.dot(direction_to_center);
        let x = total_distance.dot(source_direction);

        if y.abs() < 0.001 {
            return TrackShape::Line {
                source,
                direction: source_direction,
                length: total_distance.length(),
            };
        }

        let signed_radius = (x * x + y * y) / (2.0 * y);
        let circle_center = source + direction_to_center * signed_radius;

        let initial_angle = (source - circle_center).to_angle();
        let final_angle = (destination - circle_center).to_angle();

        let angle_delta = if signed_radius < 0.0 {
            (final_angle - initial_angle).rem_euclid(TAU) - TAU
        } else {
            (final_angle - initial_angle).rem_euclid(TAU)
        };

        let shape = TrackShape::Arc {
            start_angle: initial_angle,
            angle_diff: angle_delta,
            radius: signed_radius.abs(),
            center: circle_center,
        };

        #[cfg(debug_assertions)]
        shape.assert_sanity(
            signed_radius,
            source,
            destination,
            angle_delta,
            circle_center,
            source_direction,
            initial_angle,
            final_angle,
        );

        shape
    }

    #[cfg(debug_assertions)]
    fn assert_sanity(
        &self,
        signed_radius: f32,
        source: Vec2,
        destination: Vec2,
        angle_delta: f32,
        circle_center: Vec2,
        source_direction: Vec2,
        initial_angle: f32,
        final_angle: f32,
    ) {
        let shape = self;
        assert!(
            shape.get_transform_at_distance(0.0).0.distance(source) < 0.01,
            "Test Failed: Point at 0.0 distance Must Return Source
            signed_radius={signed_radius}
            source={source} {}",
            shape.get_transform_at_distance(0.0).0
        );
        assert!(
            (shape.get_transform_at_distance(shape.get_length() / 6.0).0 - source)
                .dot(source_direction)
                > 0.0,
            "Test Failed: Moving a small distance must move towards source direction
            source={source}
            center={circle_center}
            destination={destination}
            direction={source_direction}
            start_angle={initial_angle}
            angle_delta={angle_delta}
            signed_radius={signed_radius}
            dot product of point at 1/3 length={}
            point at 1/3rd of length={}",
            (shape.get_transform_at_distance(shape.get_length() / 3.0).0 - source)
                .dot(source_direction),
            shape.get_transform_at_distance(shape.get_length() / 3.0).0
        );
        assert!(
            shape
                .get_transform_at_distance(shape.get_length())
                .0
                .distance(destination)
                < 0.01,
            "
            Test failed: Point at end must match destination point
            initial_angle={initial_angle}
            angle_diff={angle_delta}
            expected_angle={final_angle}
            length={}
            signed_radius={signed_radius}
            destination={destination}
            calculated_destination={}",
            shape.get_length(),
            shape.get_transform_at_distance(shape.get_length()).0
        );
        assert!(
            Vec2::from_angle(shape.get_transform_at_distance(0.0).1).distance(source_direction)
                < 0.01,
            "
            Test Failed: Angle at starting position must match given source angle
            {} {}",
            source_direction.to_angle(),
            shape.get_transform_at_distance(0.0).1,
        );
    }

    pub fn reverse(self) -> TrackShape {
        match self {
            TrackShape::Line {
                source,
                direction,
                length,
            } => TrackShape::Line {
                source: source + direction * length,
                direction: -direction,
                length,
            },
            TrackShape::Arc {
                start_angle,
                angle_diff,
                radius,
                center,
            } => TrackShape::Arc {
                start_angle: start_angle + angle_diff,
                angle_diff: -angle_diff,
                radius,
                center,
            },
        }
    }

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
                    angle + FRAC_PI_2 * angle_diff.signum(),
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
}
