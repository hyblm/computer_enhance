pub mod json_parser;
pub mod profile;

#[derive(Debug, Clone, Copy)]
pub struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}
