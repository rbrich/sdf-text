extern crate roots;

mod curve;
mod rasterizer;

pub use curve::*;
pub use rasterizer::*;

use std::f32;

pub fn min(farr: &[f32]) -> f32 {
    farr.iter().cloned().fold(f32::INFINITY, f32::min)
}

pub fn max(farr: &[f32]) -> f32 {
    farr.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

