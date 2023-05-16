
mod dungeon;
mod vectors;

use dungeon::{Area, Dungeon, Tunneler, RoomGenerator, ConnectionStrategy};

fn main() {
    let mut d = Dungeon::new();
    d.add_area(Area::new(RoomGenerator::Grow { count: 4, min_size: 3, max_size: 6 }, Tunneler::Weighted, ConnectionStrategy::Basic));
    d.add_area(Area::new(RoomGenerator::GrowSeparated { count: 4, min_size: 3, max_size: 6 }, Tunneler::Weighted, ConnectionStrategy::Secondary(12)));
    d.add_area(Area::new(RoomGenerator::Chamber { min_size: 12, max_size: 15 }, Tunneler::Weighted, ConnectionStrategy::Basic));
    d.add_area(Area::new(RoomGenerator::GrowSeparated { count: 5, min_size: 2, max_size: 4 }, Tunneler::LShape, ConnectionStrategy::Secondary(20)));
    d.add_area(Area::new(RoomGenerator::Grow { count: 4, min_size: 3, max_size: 6 }, Tunneler::Weighted, ConnectionStrategy::Secondary(20)));

    d.generate();
    d.save_img("output.png", 8);
    // for i in 0..12 {
    //     let mut d = dungeon::Dungeon::new();
    //     d.generate();
    //     d.save_img(&format!("img_{}.png", i), 8);
    // }
}
