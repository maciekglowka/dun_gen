use rand::{
    distributions::WeightedIndex,
    prelude::*
};

use crate::vectors::Vector2Int;

#[derive(Default)]
pub enum Tunneler {
    #[default]
    LShape,
    Weighted
}

impl Tunneler {
    pub fn get_connector(&self) -> impl Fn(Vector2Int, Vector2Int) -> Vec<Vector2Int> {
        match self {
            Self::LShape => l_shape_connector,
            Self::Weighted => weighted_connector
        }
    }
}

pub fn weighted_connector (a: Vector2Int, b: Vector2Int) -> Vec<Vector2Int> {
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

pub fn l_shape_connector (a: Vector2Int, b: Vector2Int) -> Vec<Vector2Int> {
    let d = b - a;
    let (hor_y, ver_x) = match d.x > d.y {
        true => (a.y, b.x),
        false => (b.y, a.x)
    };

    let hor = (a.x.min(b.x)..=a.x.max(b.x))
        .map(|x| Vector2Int::new(x, hor_y))
        .collect::<Vec<_>>();

    let ver = (a.y.min(b.y)..=a.y.max(b.y))
        .map(|y| Vector2Int::new(ver_x, y))
        .collect::<Vec<_>>();
    
    [ver, hor].concat()
}