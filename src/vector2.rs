use std::ops::Range;

use rand::Rng;

pub struct Vector2 {
    pub x: f64,
    pub y: f64
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Vector2 {
        Vector2 { 
            x, 
            y
        }
    }

    pub fn new_rand(range: Range<f64>) -> Vector2 {
        let mut rng = rand::thread_rng();
        Vector2 { 
            x: rng.gen_range(range.clone()), 
            y: rng.gen_range(range) 
        }
    }
}