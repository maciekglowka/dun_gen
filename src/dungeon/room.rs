use rand::prelude::*;
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
    pub fn intersects(&self, other: &Room, border: Option<i32>) -> bool {
        let b = match border {
            Some(a) => a,
            None => 0
        };
        !(
            other.a.x > self.b.x + b ||
            other.b.x < self.a.x - b ||
            other.a.y > self.b.y + b ||
            other.b.y < self.a.y - b
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

pub type GeneratorFunc<'a> = Box<dyn Fn() -> (Vec<Room>, Vec<(usize, usize)>) + 'a>;

pub enum RoomGenerator {
    Chamber { min_size: u32, max_size: u32 },
    Grow { count: u32, min_size: u32, max_size: u32 },
    GrowSeparated { count: u32, min_size: u32, max_size: u32 },
}
impl RoomGenerator {
    // returns a vec of rooms and a vec of connection indexes
    pub fn get_generator<'a>(&'a self) -> GeneratorFunc {
        match self {
            Self::Grow {count, min_size, max_size } => Box::new(|| {
                grow_generator(*count, *min_size, *max_size, None)
            }) as GeneratorFunc,
            Self::GrowSeparated {count, min_size, max_size } => Box::new(|| {
                grow_generator(*count, *min_size, *max_size, Some(2))
            }) as GeneratorFunc,
            Self::Chamber { min_size, max_size } => Box::new(|| {
                chamber_generator(*min_size, *max_size)
            }) as GeneratorFunc
        }
    }
}

pub fn chamber_generator(min_size: u32, max_size: u32)
-> (Vec<Room>, Vec<(usize, usize)>) {
    let w = get_random_dim(min_size, max_size);
    let h = get_random_dim(min_size, max_size);

    let chamber = Room::new(Vector2Int::new(0,0), Vector2Int::new(w, h));
    (vec![chamber], Vec::new())
}

pub fn grow_generator(
    count: u32, min_size: u32, max_size: u32, room_border: Option<i32>
) -> (Vec<Room>, Vec<(usize, usize)>) {
    let mut rng = thread_rng();
    let mut connections = Vec::new();

    // bounds const for searching new room's corner around the base room
    let d = match room_border {
        None => max_size as i32,
        Some(a) => max_size as i32 + a
    };

    // first room
    let mut rooms = vec![Room::new(
        Vector2Int::new(0, 0),
        Vector2Int::new(get_random_dim(min_size, max_size), get_random_dim(min_size, max_size))
    )];

    for _ in 0..count - 1 {
        loop {
            // take a random existing room as a reference
            let prev_idx = rng.gen_range(0..rooms.len());
            let prev = &rooms[prev_idx];
            let c = prev.centre();

            let a = Vector2Int::new(rng.gen_range(c.x-d..=c.x+d), rng.gen_range(c.y-d..=c.y+d));

            // find a direction for the second room corner (outwards from the reference room)
            let mut dv = (a - c).clamped();
            if dv.x == 0 { dv.x = *[-1, 1].choose(&mut rng).unwrap() }
            if dv.y == 0 { dv.y = *[-1, 1].choose(&mut rng).unwrap() }

            // get second corner
            let w = get_random_dim(min_size, max_size);
            let h = get_random_dim(min_size, max_size);
            let b = a + Vector2Int::new(dv.x * w, dv.y * h);

            let r = Room::new(a, b);
            // if the room overlaps another generate it again
            if rooms.iter().any(|other| r.intersects(other, room_border)) { continue };

            // add a connection to the base room
            let cur_idx = rooms.len();
            connections.push((prev_idx, cur_idx));

            // room is valid, push it and break the loop
            rooms.push(r);
            break;
        }    
    }
    (rooms, connections)
}

fn get_random_dim(min: u32, max: u32) -> i32 {
    let mut rng = thread_rng();
    rng.gen_range(min..=max) as i32
}