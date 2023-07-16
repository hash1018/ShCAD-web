pub mod algorithm;
pub mod base;
pub mod client;
pub mod components;
pub mod pages;

#[derive(Default, Clone)]
pub struct Coordinates {
    pub scroll_v_pos: f64,
    pub scroll_h_pos: f64,
    pub zoom_rate: f64,
    pub center_x: f64,
    pub center_y: f64,
}

impl Coordinates {
    pub fn new() -> Coordinates {
        Self {
            scroll_v_pos: 0.0,
            scroll_h_pos: 0.0,
            zoom_rate: 1.0,
            center_x: 100.0,
            center_y: 100.0,
        }
    }
}
