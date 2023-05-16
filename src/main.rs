
mod dungeon;
mod vectors;

use dungeon::{Area, Dungeon, Tunneler, RoomGenerator};

fn main() {
    let mut d = Dungeon::new();
    d.add_area(Area::new(RoomGenerator::Grow { count: 4, min_size: 3, max_size: 6 }, Tunneler::Weighted));
    d.add_area(Area::new(RoomGenerator::Grow { count: 3, min_size: 6, max_size: 6 }, Tunneler::Weighted));
    d.add_area(Area::new(RoomGenerator::Grow { count: 5, min_size: 2, max_size: 4 }, Tunneler::LShape));
    d.add_area(Area::new(RoomGenerator::Grow { count: 4, min_size: 3, max_size: 6 }, Tunneler::Weighted));

    d.generate();
    d.save_img("output.png", 8);
    // for i in 0..12 {
    //     let mut d = dungeon::Dungeon::new();
    //     d.generate();
    //     d.save_img(&format!("img_{}.png", i), 8);
    // }
}
