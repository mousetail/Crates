use std::{
    collections::VecDeque,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use glam::Vec2;
use rand::Rng;

use crate::minivec::Minivec;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StationID(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TrackID(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JunctionId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TrainId(usize);

const MAX_RADIUS: f32 = 4.0;
const MIN_RADIUS: f32 = 2.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Junction {
    #[allow(unused)]
    id: JunctionId,
    position: Vec2,
    enterances: Minivec<2, TrackID>,
    exits: Minivec<2, TrackID>,
    angle: Option<Vec2>,
}

pub struct Track {
    id: TrackID,
    source: JunctionId,
    destiation: JunctionId,
    trains: VecDeque<TrainId>,
    direction: Vec2,
    length: f32,
    shape: TrackShape,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrackInfo {
    pub source: Vec2,
    pub destination: Vec2,
    pub shape: TrackShape,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackShape {
    Line,
    Arc {
        start_angle: f32,
        end_angle: f32,
        radius: f32,
        center: Vec2,
    },
}

impl TrackShape {
    fn normalize(self) -> TrackShape {
        match self {
            TrackShape::Line => TrackShape::Line,
            TrackShape::Arc {
                start_angle,
                end_angle,
                radius,
                center,
            } => {
                if radius < 0. {
                    TrackShape::Arc {
                        start_angle: start_angle + PI,
                        end_angle: end_angle + PI,
                        radius: -radius,
                        center: center,
                    }
                } else {
                    TrackShape::Arc {
                        start_angle,
                        end_angle,
                        radius,
                        center,
                    }
                }
            }
        }
    }
}

pub struct Train {
    id: TrainId,
    track: TrackID,
    distance: f32,
}

pub struct Network {
    tracks: Vec<Track>,
    junctions: Vec<Junction>,
    trains: Vec<Train>,
}

impl Network {
    fn add_junction(&mut self, position: Vec2) -> JunctionId {
        let junction_id = JunctionId(self.junctions.len());

        self.junctions.push(Junction {
            position,
            id: junction_id,
            exits: Minivec::new(),
            enterances: Minivec::new(),
            angle: None,
        });

        return junction_id;
    }

    fn connect_track(&mut self, source_id: JunctionId, destination_id: JunctionId) -> TrackID {
        let source = self.junctions[source_id.0];
        let destiation = self.junctions[destination_id.0];

        #[derive(Debug)]
        struct ArcInfo {
            radius: f32,
            final_angle: Vec2,
            center: Vec2,
        }

        fn radius_to(start: Vec2, start_angle: Vec2, point: Vec2) -> Option<ArcInfo> {
            let x = start_angle.dot(point - start);
            let y = start_angle.perp().dot(point - start);

            if y.abs() < 0.01 {
                return None;
            }

            let radius = (x * x + y * y) / (2.0 * y);
            let center = start + start_angle.perp() * radius;

            let final_angle = ((point - center).perp()).normalize() * radius.signum();

            return Some(ArcInfo {
                radius,
                final_angle,
                center,
            });
        }

        let create_line = |this: &mut Self, source_id: JunctionId, destination_id: JunctionId| {
            let track = this.add_track(source_id, destination_id, TrackShape::Line);
            let angle = (destiation.position - source.position).normalize();
            this.junctions[source_id.0].angle = Some(angle);
            this.junctions[destination_id.0].angle = Some(-angle);

            track
        };

        match (source.angle, destiation.angle) {
            (None, None) => create_line(self, source_id, destination_id),
            (Some(ang), None) => {
                if let Some(arc_info) = radius_to(source.position, ang, destiation.position) {
                    self.junctions[destiation.id.0].angle = Some(arc_info.final_angle);

                    self.add_track(
                        source_id,
                        destination_id,
                        TrackShape::Arc {
                            start_angle: ang.to_angle() - FRAC_PI_2,
                            end_angle: arc_info.final_angle.to_angle() - FRAC_PI_2,
                            radius: arc_info.radius,
                            center: arc_info.center,
                        }
                        .normalize(),
                    )
                } else {
                    create_line(self, source_id, destination_id)
                }
            }
            (None, Some(ang)) => {
                if let Some(arc_info) = radius_to(destiation.position, -ang, source.position) {
                    self.junctions[source.id.0].angle = Some(-arc_info.final_angle);

                    self.add_track(
                        source_id,
                        destination_id,
                        TrackShape::Arc {
                            start_angle: (arc_info.final_angle).to_angle() - FRAC_PI_2,
                            end_angle: (-ang).to_angle() - FRAC_PI_2,
                            radius: arc_info.radius,
                            center: arc_info.center,
                        }
                        .normalize(),
                    )
                } else {
                    create_line(self, source_id, destination_id)
                }
            }
            (Some(a1), Some(a2)) => {
                let (midpoint_1, midpoint_2) = ([1., -1.])
                    .into_iter()
                    .flat_map(|a| [1.0, -1.0].map(|b| (a, b)))
                    .filter_map(|(sign_1, sign_2)| {
                        let c1 = source.position + sign_1 * (a1.perp()) * MAX_RADIUS;
                        let c2 = destiation.position + sign_2 * (a2.perp()) * MAX_RADIUS;

                        let direction = (c2 - c1).normalize();

                        let midpoint_1 = c1 - direction.perp() * MAX_RADIUS;
                        let midpoint_2 = c2 - direction.perp() * MAX_RADIUS;

                        if (midpoint_1 - source.position).dot(destiation.position - source.position)
                            > 0.0
                            && (destiation.position - midpoint_2)
                                .dot(destiation.position - source.position)
                                > 0.0
                            && (midpoint_2 - midpoint_1).dot(destiation.position - source.position)
                                > 0.0
                        {
                            Some((midpoint_1, midpoint_2))
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap();

                let junction_1 = self.add_junction(midpoint_1);
                let junction_2 = self.add_junction(midpoint_2);

                self.connect_track(source_id, junction_1);
                self.connect_track(junction_2, destination_id);

                create_line(self, junction_1, junction_2)
            }
        }
    }

    fn add_track(
        &mut self,
        source_id: JunctionId,
        destination_id: JunctionId,
        shape: TrackShape,
    ) -> TrackID {
        let track_id = TrackID(self.tracks.len());

        self.junctions[source_id.0].exits.push(track_id).unwrap();
        self.junctions[destination_id.0]
            .enterances
            .push(track_id)
            .unwrap();
        let source = &self.junctions[source_id.0];
        let destination = &self.junctions[destination_id.0];

        let track = Track {
            source: source_id,
            destiation: destination_id,
            trains: VecDeque::new(),

            direction: (destination.position - source.position).normalize(),
            length: match shape {
                TrackShape::Line => (destination.position - source.position).length(),
                TrackShape::Arc {
                    start_angle,
                    end_angle,
                    radius,
                    ..
                } => (end_angle - start_angle + TAU) % TAU * radius,
            },
            id: track_id,
            shape,
        };

        self.tracks.push(track);

        return track_id;
    }

    fn add_train(&mut self, track: TrackID) -> TrainId {
        let train_id = TrainId(self.trains.len());

        let train = Train {
            track: track,
            distance: 0.0,
            id: train_id,
        };

        self.trains.push(train);

        self.tracks[track.0].trains.push_back(train_id);

        return train_id;
    }

    pub fn train_positions<'a>(&'a self) -> impl Iterator<Item = (Vec2, f32)> + 'a {
        self.trains.iter().map(|train| {
            let track = &self.tracks[train.track.0];

            match track.shape {
                TrackShape::Line => (
                    self.junctions[track.source.0].position + track.direction * train.distance,
                    track.direction.to_angle(),
                ),
                TrackShape::Arc {
                    start_angle,
                    radius,
                    center,
                    ..
                } => {
                    let angle = train.distance / radius + start_angle;

                    (center + Vec2::from_angle(angle) * radius, angle + FRAC_PI_2)
                }
            }
        })
    }

    pub fn curves<'a>(&'a self) -> impl Iterator<Item = TrackInfo> + 'a {
        self.tracks.iter().map(|track| TrackInfo {
            source: self.junctions[track.source.0].position,
            destination: self.junctions[track.destiation.0].position,
            shape: track.shape,
        })
    }

    pub fn update(&mut self, delta_time: f32) {
        for train in &mut self.trains {
            let track = &self.tracks[train.track.0];

            train.distance += delta_time * 8.0;

            if train.distance > track.length {
                train.distance %= track.length;

                let junction = &self.junctions[track.destiation.0];
                let next_track_id =
                    junction.exits[rand::thread_rng().gen_range(0..junction.exits.len())];
                let next_track = &mut self.tracks[next_track_id.0];
                next_track.trains.push_back(train.id);

                train.track = next_track.id;
            }
        }
    }
}

pub fn generate_network() -> Network {
    let mut network = Network {
        tracks: vec![],
        trains: vec![],
        junctions: vec![],
    };

    let junctions = [
        network.add_junction(Vec2::new(-8., -8.)),
        network.add_junction(Vec2::new(8., -8.)),
        network.add_junction(Vec2::new(8., 8.)),
        network.add_junction(Vec2::new(-8., 8.)),
        network.add_junction(Vec2::new(-16., 0.)),
        network.add_junction(Vec2::new(-16., -16.)),
    ];

    let tracks = [
        network.connect_track(junctions[0], junctions[1]),
        network.connect_track(junctions[1], junctions[2]),
        network.connect_track(junctions[2], junctions[3]),
        network.connect_track(junctions[3], junctions[0]),
        network.connect_track(junctions[3], junctions[1]),
        network.connect_track(junctions[0], junctions[4]),
        network.connect_track(junctions[4], junctions[5]),
        network.connect_track(junctions[5], junctions[2]),
        // network.connect_track(junctions[4], junctions[3]),
        // network.connect_track(junctions[3], junctions[0]),
    ];

    network.add_train(tracks[0]);
    network.add_train(tracks[1]);

    return network;
}
