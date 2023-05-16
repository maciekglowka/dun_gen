
mod dungeon;
mod vectors;

use dungeon::{Area, Dungeon, Tunneler};

fn main() {
    let mut d = Dungeon::new();
    d.add_area(Area::new(4, 3, 6, Tunneler::Weighted));
    d.add_area(Area::new(3, 6, 6, Tunneler::Weighted));
    d.add_area(Area::new(5, 2, 4, Tunneler::LShape));
    d.add_area(Area::new(4, 3, 6, Tunneler::Weighted));

    d.generate();
    d.save_img("output.png", 8);
    // for i in 0..12 {
    //     let mut d = dungeon::Dungeon::new();
    //     d.generate();
    //     d.save_img(&format!("img_{}.png", i), 8);
    // }
}
