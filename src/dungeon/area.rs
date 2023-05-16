use rand::{
    prelude::*,
    seq::SliceRandom
};
use std::collections::HashSet;
use crate::vectors::Vector2Int;

use super::tunnels::Tunneler;

#[derive(Debug, Eq, PartialEq)]
pub struct Room {
    pub a: Vector2Int,
    pub b: Vector2Int
}
impl Room {
    pub fn new(a: Vector2Int, b: Vector2Int) -> Self {
        Room {
            a: Vector2Int::new(a.x.min(b.x), a.y.min(b.y)),
            b: Vector2Int::new(a.x.max(b.x), a.y.max(b.y))
        }
    }
    pub fn corners(&self) -> [Vector2Int; 4] {
        [
            Vector2Int::new(self.a.x, self.a.y), Vector2Int::new(self.b.x, self.a.y),
            Vector2Int::new(self.b.x, self.b.y), Vector2Int::new(self.a.x, self.b.y)
        ]
    }
    pub fn random_point(&self) -> Vector2Int {
        let mut rng = thread_rng();
        let x = rng.gen_range(self.a.x..=self.b.x);
        let y = rng.gen_range(self.a.y..=self.b.y);
        Vector2Int::new(x, y)
    }
    pub fn centre(&self) -> Vector2Int {
        Vector2Int::new((self.b.x+self.a.x) / 2, (self.b.y+self.a.y) / 2)
    }
    pub fn intersects(&self, other: &Room) -> bool {
        !(
            other.a.x > self.b.x ||
            other.b.x < self.a.x ||
            other.a.y > self.b.y ||
            other.b.y < self.a.y
        )
    }
    pub fn join(&self, other: &Room, tunneler: &Tunneler) -> Vec<Vector2Int> {
        // make a connection between two rooms
        let va = self.random_point();
        let vb = other.random_point();
        tunneler.get_connector()(va, vb)
    }
    pub fn get_tiles(&self) -> HashSet<Vector2Int> {
        (self.a.y..=self.b.y).map(|y| {
                (self.a.x..=self.b.x).map(move |x| {
                    Vector2Int::new(x, y)
                })
            })
            .flatten()
            .collect()
    }
}

#[derive(Default)]
pub struct Area {
    pub rooms: Vec<Room>,
    pub paths: Vec<Vec<Vector2Int>>,
    pub tunneler: Tunneler,
    pub room_count: u32,
    pub room_dim_min: u32,
    pub room_dim_max: u32
}
impl Area {
    pub fn new(
        room_count: u32,
        room_dim_min: u32,
        room_dim_max: u32,
        tunneler: Tunneler
    ) -> Area {
        Area {
            room_count,
            room_dim_min,
            room_dim_max,
            tunneler,
            ..Default::default()
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
    fn get_room_dim<R: Rng>(&self, rng: &mut R) -> i32 {
        rng.gen_range(self.room_dim_min..=self.room_dim_max) as i32
    }
    pub fn generate_rooms(&mut self) {
        let mut rng = thread_rng();
        self.paths = Vec::new();

        // first room
        let mut rooms = vec![Room::new(
            Vector2Int::new(0, 0),
            Vector2Int::new(self.get_room_dim(&mut rng), self.get_room_dim(&mut rng))
        )];

        for _ in 0..self.room_count - 1 {
            loop {
                // take a random existing room as a reference
                let prev = rooms.choose(&mut rng).unwrap();
                let c = prev.centre();
                // define bounds for the new room's corner
                let d = self.room_dim_max as i32;

                let a = Vector2Int::new(rng.gen_range(c.x-d..=c.x+d), rng.gen_range(c.y-d..=c.y+d));

                // find a direction for the second room corner (outwards from the reference room)
                let mut dv = (a - c).clamped();
                if dv.x == 0 { dv.x = *[-1, 1].choose(&mut rng).unwrap() }
                if dv.y == 0 { dv.y = *[-1, 1].choose(&mut rng).unwrap() }

                // get second corner
                let w = self.get_room_dim(&mut rng);
                let h = self.get_room_dim(&mut rng);
                let b = a + Vector2Int::new(dv.x * w, dv.y * h);
  
                let r = Room::new(a, b);
                // if the room overlaps another generate it again
                if rooms.iter().any(|other| r.intersects(other)) { continue };
                self.join_internal_rooms(prev, &r, None);

                // try second connection
                let other = rooms.choose(&mut rng).unwrap();
                if other != prev {
                    self.join_internal_rooms(rooms.choose(&mut rng).unwrap(), &r, Some(3*self.room_dim_max));
                }

                // room is valid, push it and break the loop
                rooms.push(r);
                break;
            }    
        }
        self.rooms = rooms;
    }
    fn join_internal_rooms(&mut self, a: &Room, b: &Room, max_length: Option<u32>) {
        let path = a.join(b, &self.tunneler);
        if let Some(max_length) = max_length {
            if path.len() > max_length as usize { return }
        }
        self.paths.push(path);
    }
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
