
mod dungeon;
mod vectors;

fn main() {
    for i in 0..12 {
        let mut d = dungeon::Dungeon::new();
        d.generate();
        d.save_img(&format!("img_{}.png", i), 4);
    }
}
