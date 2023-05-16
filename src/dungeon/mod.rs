use image::{ImageBuffer};
use std::collections::HashSet;

use crate::vectors::Vector2Int;

mod area;
mod room;
mod tunnels;

pub use area::Area;
pub use room::{Room, RoomGenerator};
pub use tunnels::Tunneler;


pub struct Dungeon {
    pub tiles: HashSet<Vector2Int>,
    pub areas: Vec<Area>,
    pub row_count: usize,
    rows: Vec<Vec<usize>>
}
impl Dungeon {
    pub fn new() -> Self {
        let row_count = 2;
        let rows = (0..row_count).map(|_| Vec::new()).collect::<Vec<_>>();
        Dungeon { 
            row_count,
            tiles: HashSet::new(),
            areas: Vec::new(),
            rows
        }
    }
    pub fn add_area(&mut self, area: Area) {
        self.areas.push(area);
        let idx = self.areas.len() - 1;
        // insert index to appropriate row table
        self.rows[idx % self.row_count].push(idx);
    }
    fn get_dim(&self) -> Vector2Int {
        // assumes that the areas have been shifted
        let max_x = self.tiles.iter().map(|a| a.x).max().unwrap();
        let max_y = self.tiles.iter().map(|a| a.y).max().unwrap();
        Vector2Int::new(max_x, max_y)
    }
    pub fn save_img(&self, path: &str, scale: u32) {
        let size = self.get_dim();
        let mut buf: image::RgbImage = ImageBuffer::new(size.x as u32, size.y as u32);

        for (x, y, pixel) in buf.enumerate_pixels_mut() {
            if self.tiles.contains(&Vector2Int::new(x as i32, y as i32)) {
                *pixel = image::Rgb([150, 150, 50]);
            } else {
                *pixel = image::Rgb([0, 0, 0])
            }
        }
        let resized = image::imageops::resize(&buf, size.x as u32 * scale, size.y as u32 * scale, image::imageops::FilterType::Nearest);
        resized.save(path).unwrap();
    }
    fn write_areas(&mut self) {
        // persist areas to tiles
        for area in self.areas.iter() {
            for room in area.rooms.iter() {
                self.tiles.extend(room.get_tiles());
            }
            for path in area.paths.iter() {
                self.tiles.extend(path);
            }
        }
    }
    fn connect_areas(&mut self) {
        for (y, row) in self.rows.iter().enumerate() {
            for (x, idx) in row.iter().enumerate() {
                if x != 0 {
                    // join to area at x - 1
                    let target_idx = row[x-1];
                    self.tiles.extend(&self.areas[*idx].join(&self.areas[target_idx]));
                };
                if y != 0 {
                    // join to area at y - 1
                    let target_idx = self.rows[y-1][x];
                    self.tiles.extend(&self.areas[*idx].join(&self.areas[target_idx]));
                };
            }
        }
    }
    fn position_areas(&mut self) {
        let column_count = self.rows[0].len();
        let spacing = 2;

        // calculate area offsets based on row / column
        let column_widths = (0..column_count).map(|i| 
                self.rows.iter().map(|r| match r.get(i) {
                    None => 0,
                    Some(a) => self.areas[i].get_size().x
                }).max().unwrap() + spacing
            )
            .collect::<Vec<_>>();
        let row_heights = self.rows.iter()
            .map(|r| 
                r.iter().map(|i| self.areas[*i].get_size().y).max().unwrap() + spacing
            )
            .collect::<Vec<_>>();
        let column_shifts = (0..column_widths.len())
            .map(|i| column_widths[..i].iter().sum())
            .collect::<Vec<i32>>();
        let row_shifts = (0..row_heights.len())
            .map(|i| row_heights[..i].iter().sum())
            .collect::<Vec<i32>>();

        // reposition areas
        for (y, row) in self.rows.iter().enumerate() {
            for (x, idx) in row.iter().enumerate() {
                self.areas[*idx].shift(column_shifts[x], row_shifts[y]);
            }
        }
    }
    pub fn generate(&mut self) {
        for area in self.areas.iter_mut() {
            area.generate_rooms();
        }
        self.position_areas();
        self.write_areas();
        self.connect_areas();
    }
}
