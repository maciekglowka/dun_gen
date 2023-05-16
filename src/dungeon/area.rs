use rand::prelude::*;
use crate::vectors::Vector2Int;

use super::room::{Room, RoomGenerator};
use super::tunnels::Tunneler;


pub struct Area {
    pub rooms: Vec<Room>,
    pub paths: Vec<Vec<Vector2Int>>,
    pub tunneler: Tunneler,
    pub room_generator: RoomGenerator,
    pub connection_strategy: ConnectionStrategy
}
impl Area {
    pub fn new(
        room_generator: RoomGenerator,
        tunneler: Tunneler,
        connection_strategy: ConnectionStrategy
    ) -> Area {
        Area {
            room_generator,
            tunneler,
            connection_strategy,
            rooms: Vec::new(),
            paths: Vec::new()
        }
    }
    pub fn get_bounds(&self) -> (Vector2Int, Vector2Int) {
        let min_x = self.rooms.iter().map(|r| r.a.x).min().unwrap();
        let max_x = self.rooms.iter().map(|r| r.b.x).max().unwrap();
        let min_y = self.rooms.iter().map(|r| r.a.y).min().unwrap();
        let max_y = self.rooms.iter().map(|r| r.b.y).max().unwrap();
        (Vector2Int::new(min_x, min_y), Vector2Int::new(max_x, max_y))
    }
    pub fn get_size(&self) -> Vector2Int {
        let bounds = self.get_bounds();
        Vector2Int::new(bounds.1.x - bounds.0.x, bounds.1.y - bounds.0.y)
    }
    pub fn shift(&mut self, base_x: i32, base_y: i32) {
        // translate the entire area by offset
        let bounds = self.get_bounds();
        let dx = base_x - bounds.0.x;
        let dy = base_y - bounds.0.y;
        let d = Vector2Int::new(dx, dy);

        for room in self.rooms.iter_mut() {
            room.a += d;
            room.b += d;
        }
        for path in self.paths.iter_mut() {
            for v in path.iter_mut() {
                *v += d;
            }
        }
    }
    pub fn generate_rooms(&mut self) {
        let (rooms, connections) = self.room_generator.get_generator()();
        
        // generate connections
        self.paths = self.connection_strategy.get_connections_generator()(&self.tunneler, &rooms, &connections);
        self.rooms = rooms;
    }
    // fn join_internal_rooms(&mut self, a: &Room, b: &Room, max_length: Option<u32>) {
    //     let path = a.join(b, &self.tunneler);
    //     if let Some(max_length) = max_length {
    //         if path.len() > max_length as usize { return }
    //     }
    //     self.paths.push(path);
    // }
    fn get_closest_rooms<'a>(&'a self, other: &'a Area) -> (&'a Room, &'a Room) {
        // find closest room pair between two areas
        // based on corner distances
        let mut dists = Vec::new();
        for ra in self.rooms.iter() {
            for rb in other.rooms.iter() {
                // find min corner dist
                let d = ra.corners().iter()
                    .map(|ca| rb.corners().iter().map(|cb| ca.manhattan(*cb)).collect::<Vec<_>>())
                    .flatten()
                    .min()
                    .unwrap();
                dists.push((d, ra, rb));
            }
        }
        dists.sort_by(|a, b| a.0.cmp(&b.0));
        (dists[0].1, dists[0].2)
    }
    pub fn join(&self, other: &Area) -> Vec<Vector2Int> {
        // make a connection between two areas
        let rooms = self.get_closest_rooms(other);
        rooms.0.join(rooms.1, &self.tunneler)
    }
}

pub type ConnectionsGenerator<'a> = Box<dyn Fn(&Tunneler, &Vec<Room>, &Vec<(usize, usize)>) -> Vec<Vec<Vector2Int>> + 'a>;

pub enum ConnectionStrategy {
    Basic,
    Secondary(usize)
}
impl ConnectionStrategy {
    pub fn get_connections_generator(&self) -> ConnectionsGenerator {
        match self {
            Self::Basic => Box::new(
                |tunneler: &Tunneler, rooms: &Vec<Room>, required: &Vec<(usize, usize)>| { get_neccessary_connections(tunneler, rooms, required) }
            ) as ConnectionsGenerator,
            Self::Secondary(max_dist) => Box::new(
                |tunneler: &Tunneler, rooms: &Vec<Room>, required: &Vec<(usize, usize)>| { get_with_secondary(tunneler, rooms, required, *max_dist) }
            ) as ConnectionsGenerator,
        }
    }
}

fn get_neccessary_connections(
    tunneler: &Tunneler, rooms: &Vec<Room>, connections: &Vec<(usize, usize)>
) -> Vec<Vec<Vector2Int>> {
    connections.iter()
        .map(|conn| rooms[conn.0].join(&rooms[conn.1], tunneler))
        .collect()
}

fn get_with_secondary(
    tunneler: &Tunneler, rooms: &Vec<Room>, required: &Vec<(usize, usize)>, max_dist: usize
) -> Vec<Vec<Vector2Int>> {
    let mut rng = thread_rng();
    let mut paths = get_neccessary_connections(tunneler, rooms, required);
    for idx in 0..rooms.len() {
        let other_idx = rng.gen_range(0..rooms.len());
        if other_idx == idx { continue }
        let path = rooms[idx].join(&rooms[other_idx], tunneler);
        if path.len() > max_dist { continue };
        paths.push(path);
    };
    paths
}