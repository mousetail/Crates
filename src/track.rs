use std::{
    collections::VecDeque,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use glam::Vec2;
use rand::Rng;

use crate::{minivec::Minivec, track_shape::TrackShape};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StationID(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TrackID(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct JunctionId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TrainId(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StationId;

const MAX_RADIUS: f32 = 4.0;
const IDEAL_SEGMENT_LENGTH: f32 = 2.0;
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


pub struct Train {
    id: TrainId,
    track: TrackID,
    distance: f32,
}

struct Station {
    position: Vec2,
    length: f32,
    track: TrackID,
    angle: f32,
}

pub struct Network {
    tracks: Vec<Track>,
    junctions: Vec<Junction>,
    trains: Vec<Train>,
    stations: Vec<Station>,
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

    fn assert_correctness(&self, message: &'static str) {
        for (id, junction) in self.junctions.iter().enumerate() {
            assert_eq!(JunctionId(id), junction.id)
        }

        for track in &self.tracks {
            let expected_destination = self.junctions[track.destiation.0].position;
            let actual_destination = track.shape.get_transform_at_distance(track.length).0;

            assert!(expected_destination.distance(actual_destination) < 0.01, "{message} Expected: {expected_destination:?} Actual: {actual_destination:?}\ndestination: {:?} source: {:?} id: {:?}",
                track.destiation, track.source, track.id
        );
        }
    }

    fn create_line(&mut self, source_id: JunctionId, destination_id: JunctionId) -> TrackID {
        let source = self.junctions[source_id.0];
        let destination = self.junctions[destination_id.0];

        let angle = (destination.position - source.position).normalize();
        let track = self.add_track(
            source_id,
            destination_id,
            TrackShape::Line {
                source: self.junctions[source_id.0].position,
                direction: angle,
            },
        );
        self.junctions[source_id.0].angle.get_or_insert(angle);
        self.junctions[destination_id.0].angle.get_or_insert(angle);

        track
    }

    fn connect_track(&mut self, source_id: JunctionId, destination_id: JunctionId) -> TrackID {
        let source = self.junctions[source_id.0];
        let destiation = self.junctions[destination_id.0];

        #[derive(Debug)]
        struct ArcInfo {
            radius: f32,
            final_angle: Vec2,
            angle_distance: f32,
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

            let angle_distance = start_angle.angle_to(final_angle);

            let angle_distance = if radius < 0.0 {
                (angle_distance + TAU) % TAU - TAU
            } else {
                (angle_distance + TAU) % TAU
            };

            return Some(ArcInfo {
                radius,
                final_angle,
                center,
                angle_distance,
            });
        }

        match (source.angle, destiation.angle) {
            (None, None) => self.create_line(source_id, destination_id),
            (Some(ang), None) => {
                if let Some(arc_info) = radius_to(source.position, ang, destiation.position) {
                    self.junctions[destiation.id.0].angle = Some(arc_info.final_angle);

                    self.add_track(
                        source_id,
                        destination_id,
                        TrackShape::Arc {
                            start_angle: ang.to_angle() - FRAC_PI_2,
                            angle_diff: arc_info.angle_distance,
                            radius: arc_info.radius,
                            center: arc_info.center,
                        }
                        .normalize(),
                    )
                } else {
                    self.create_line(source_id, destination_id)
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
                            angle_diff: -arc_info.angle_distance,
                            radius: arc_info.radius,
                            center: arc_info.center,
                        }
                        .normalize(),
                    )
                } else {
                    self.create_line(source_id, destination_id)
                }
            }
            (Some(a1), Some(a2)) => {
                let (midpoint_1, midpoint_2) = ([-1., 1.])
                    .into_iter()
                    .flat_map(|a| [-1.0, 1.0].map(|b| (a, b)))
                    .filter_map(|(sign_1, sign_2)| {
                        let c1 = source.position + sign_1 * (a1.perp()) * MAX_RADIUS;
                        let c2 = destiation.position + sign_2 * (a2.perp()) * MAX_RADIUS;

                        let direction = (c2 - c1).normalize();

                        let midpoint_1 = c1 - direction.perp() * MAX_RADIUS * sign_1;
                        let midpoint_2 = c2 - direction.perp() * MAX_RADIUS * sign_2;

                        let target_direction_vector = destiation.position - source.position;

                        if (midpoint_1 - source.position).dot(target_direction_vector) > 0.0
                            && (midpoint_2 - midpoint_1).dot(target_direction_vector) > 0.0
                            && (destiation.position - midpoint_2).dot(target_direction_vector) > 0.0
                        // && (midpoint_1 - source.position).dot(a1) >= 0.0
                        // && (destiation.position - midpoint_2).dot(-a2) >= 0.0
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

                self.create_line(junction_1, junction_2)
            }
        }
    }

    fn get_start_junction(&self, station: StationID) -> JunctionId {
        self.tracks[self.stations[station.0].track.0].source
    }

    fn get_end_junction(&self, station: StationID) -> JunctionId {
        self.tracks[self.stations[station.0].track.0].destiation
    }

    fn add_station(&mut self, position: Vec2, length: f32, angle: f32) -> StationID {
        let station_id = StationID(self.stations.len());

        let start_junction = self.add_junction(position);
        let end_junction = self.add_junction(position + Vec2::from_angle(angle) * length);

        let segment = self.add_track(
            start_junction,
            end_junction,
            TrackShape::Line {
                source: position,
                direction: Vec2::from_angle(angle),
            },
        );

        self.stations.push(Station {
            position,
            length,
            angle,
            track: segment,
        });

        return station_id;
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
                TrackShape::Line { .. } => (destination.position - source.position).length(),
                TrackShape::Arc {
                    angle_diff, radius, ..
                } => angle_diff.abs() * radius,
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

            track.shape.get_transform_at_distance(train.distance)
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

            if train.distance > track.length.abs() {
                train.distance %= track.length.abs();

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
        stations: vec![],
    };

    let width = 64.0;
    let height = 40.0;
    let border_radius = 12.0;

    let junctions = [
        network.add_junction(Vec2::new(-width * 0.5 + border_radius, -height * 0.5)),
        network.add_junction(Vec2::new(-width * 0.5, -height * 0.5 + border_radius)),
        network.add_junction(Vec2::new(-width * 0.5, height * 0.5 - border_radius)),
        network.add_junction(Vec2::new(-width * 0.5 + border_radius, height * 0.5)),
        network.add_junction(Vec2::new(width * 0.5 - border_radius, height * 0.5)),
        network.add_junction(Vec2::new(width * 0.5, height * 0.5 - border_radius)),
        network.add_junction(Vec2::new(width * 0.5, -height * 0.5 + border_radius)),
        network.add_junction(Vec2::new(width * 0.5 - border_radius, -height * 0.5)),
    ];

    let center_junction = network.add_junction(Vec2::ZERO);

    let tracks = [1, 2, 3, 4, 5, 6, 7, 0].map(|i| {
        network.connect_track(
            junctions[i],
            junctions[(i + junctions.len() + 1) % junctions.len()],
        )
    });

    network.connect_track(junctions[3], center_junction);
    network.connect_track(junctions[6], center_junction);
    network.connect_track(center_junction, junctions[4]);

    network.assert_correctness("After new tracks");

    network.add_train(tracks[0]);
    network.add_train(tracks[1]);

    return network;
}
