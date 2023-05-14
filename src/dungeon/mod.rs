use image::{ImageBuffer};
use rand::{
    distributions::WeightedIndex,
    prelude::*,
    seq::SliceRandom
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::vectors::{ORTHO_DIRECTIONS, Vector2Int};

#[derive(Debug)]
struct Room {
    a: Vector2Int,
    b: Vector2Int
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
}

struct Area {
    pub rooms: Vec<Room>,
    pub paths: Vec<Vec<Vector2Int>>
}
impl Area {
    pub fn new() -> Area {
        Area { rooms: Vec::new(), paths: Vec::new() }
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
        let mut rng = thread_rng();
        let min_size = 4;
        let max_size = 8;

        self.paths = Vec::new();

        // first room
        let mut rooms = vec![Room::new(
            Vector2Int::new(0, 0),
            Vector2Int::new(rng.gen_range(min_size..=max_size), rng.gen_range(min_size..=max_size))
        )];

        for _ in 0..4 {
            loop {
                let prev = rooms.choose(&mut rng).unwrap();
                let c = prev.centre();
                let d = max_size;

                let a = Vector2Int::new(rng.gen_range(c.x-d..=c.x+d), rng.gen_range(c.y-d..=c.y+d));
                let mut dv = (a - c).clamped();
                if dv.x == 0 {
                    dv.x = *[-1, 1].choose(&mut rng).unwrap()
                }
                if dv.y == 0 {
                    dv.y = *[-1, 1].choose(&mut rng).unwrap()
                }
                // let dx = if a.x - c.x < 0 { -1 } else { 1 };
                // let dy = if a.y - c.y < 0 { -1 } else { 1 };
                let w = rng.gen_range(min_size..=max_size);
                let h = rng.gen_range(min_size..=max_size);
                let b = a + Vector2Int::new(dv.x * w, dv.y * h);
  
                let r = Room::new(a, b);
                if rooms.iter().any(|other| r.intersects(other)) { continue };
                self.join_rooms(prev, &r, None);
                // second connection
                self.join_rooms(rooms.choose(&mut rng).unwrap(), &r, Some(2 * max_size));
                rooms.push(r);
                break;
            }    
        }
        self.rooms = rooms;
    }
    fn join_rooms(&mut self, a: &Room, b: &Room, max_length: Option<i32>) {
        if let Some(path) = join_rooms(a, b, max_length) {
            self.paths.push(path);
        }
    }
    fn get_closest_rooms<'a>(&'a self, other: &'a Area) -> (&'a Room, &'a Room) {
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
}

pub struct Dungeon {
    pub tiles: HashSet<Vector2Int>
}
impl Dungeon {
    pub fn new() -> Self {
        Dungeon { tiles: HashSet::new() }
    }
    fn get_bounds(&self) -> (Vector2Int, Vector2Int) {
        let min_x = self.tiles.iter().map(|a| a.x).min().unwrap();
        let max_x = self.tiles.iter().map(|a| a.x).max().unwrap();
        let min_y = self.tiles.iter().map(|a| a.y).min().unwrap();
        let max_y = self.tiles.iter().map(|a| a.y).max().unwrap();
        (Vector2Int::new(min_x, min_y), Vector2Int::new(max_x, max_y))
    }
    fn get_dim(&self) -> Vector2Int {
        let bounds = self.get_bounds();
        Vector2Int::new(bounds.1.x - bounds.0.x, bounds.1.y - bounds.0.y)
    }
    pub fn print(&self) {
        // for test purposes only
        let bounds = self.get_bounds();

        for y in bounds.0.y..=bounds.1.y {
            for x in bounds.0.x..=bounds.1.x {
                if self.tiles.contains(&Vector2Int::new(x, y)) { print!(".")} else { print!("#")};
            }
            print!("\n");
        }
    }
    pub fn save_img(&self, path: &str, scale: u32) {
        let size = self.get_dim();
        let bounds = self.get_bounds();
        let mut buf: image::RgbImage = ImageBuffer::new(size.x as u32, size.y as u32);
        let dx = bounds.0.x;
        let dy = bounds.0.y;

        for (x, y, pixel) in buf.enumerate_pixels_mut() {
            if self.tiles.contains(&Vector2Int::new(x as i32 + dx, y as i32 + dy)) {
                *pixel = image::Rgb([150, 150, 50]);
            } else {
                *pixel = image::Rgb([0, 0, 0])
            }
        }
        let resized = image::imageops::resize(&buf, size.x as u32 * scale, size.y as u32 * scale, image::imageops::FilterType::Nearest);
        resized.save(path).unwrap();
    }
    fn insert_room(&mut self, room: &Room) {
        for y in room.a.y..=room.b.y {
            for x in room.a.x..=room.b.x {
                self.tiles.insert(Vector2Int::new(x, y));
            }
        }
    }
    fn insert_path(&mut self, path: &Vec<Vector2Int>) {
        for v in path.iter() {
            self.tiles.insert(*v);
        }
    }
    pub fn generate(&mut self) {
        let area_count = 6;
        let row_count = 2;

        let mut areas = (0..area_count).map(|_| Area::new()).collect::<Vec<_>>();
        let mut rows = (0..row_count).map(|_| Vec::new()).collect::<Vec<_>>();

        for (idx, area) in areas.iter_mut().enumerate() {
            area.generate_rooms();
            rows[idx % row_count].push(area);
        }
        let column_widths = (0..rows[0].len()).map(|i| 
                rows.iter().map(|r| match r.get(i) {
                    None => 0,
                    Some(a) => a.get_size().x
                }).max().unwrap()
            )
            .collect::<Vec<_>>();
        let row_heights = rows.iter()
            .map(|r| 
                r.iter().map(|a| a.get_size().y).max().unwrap()
            )
            .collect::<Vec<_>>();
        let column_shifts = (0..column_widths.len())
            .map(|i| column_widths[..i].iter().sum())
            .collect::<Vec<i32>>();
        let row_shifts = (0..row_heights.len())
            .map(|i| row_heights[..i].iter().sum())
            .collect::<Vec<i32>>();

        for (y, row) in rows.iter_mut().enumerate() {
            for (x, area) in row.iter_mut().enumerate() {
                area.shift(column_shifts[x] + 1, row_shifts[y] + 1);
                for room in area.rooms.iter() {
                    self.insert_room(room);
                }
                for path in area.paths.iter() {
                    self.insert_path(path);
                }
            }
        }
       
        // connect areas
        for (y, row) in rows.iter().enumerate() {
            for (x, area) in row.iter().enumerate() {
                let mut pairs = Vec::new();
                if x != 0 {
                    pairs.push(area.get_closest_rooms(&row[x - 1]))
                };
                if y != 0 {
                    pairs.push(area.get_closest_rooms(&rows[y-1][x]))
                };
                for pair in pairs.iter() {
                    println!("{:?}", pair);
                    let path = join_rooms(pair.0, pair.1, None).unwrap();
                    self.insert_path(&path);
                }
            }
        }
    }
}


fn join_rooms(a: &Room, b: &Room, max_length: Option<i32>) -> Option<Vec<Vector2Int>> {
    let va = a.random_point();
    let vb = b.random_point();
    let path = make_path(va, vb);
    if let Some(max_length) = max_length {
        if path.len() > max_length as usize { return None }
    }
    Some(path)
}


fn make_path(a: Vector2Int, b: Vector2Int) -> Vec<Vector2Int> {
    let mut cur = a;
    let mut path = Vec::new();
    let mut rng = thread_rng();

    while cur != b {
        path.push(cur);
        let dirs = vec![b.x - cur.x, b.y - cur.y];

        let dist = WeightedIndex::new(dirs.iter().map(|d| d.abs())).unwrap();
        let dir_idx = dist.sample(&mut rng);
        let dv = match dir_idx {
            0 => Vector2Int::new(dirs[0] / dirs[0].abs(), 0),
            1 => Vector2Int::new(0, dirs[1] / dirs[1].abs()),
            _ => panic!()
        };
        cur += dv;
    }
    path
}