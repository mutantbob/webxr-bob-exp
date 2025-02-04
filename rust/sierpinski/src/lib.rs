mod utils;

use std::f32::consts::SQRT_2;
use wasm_bindgen::prelude::*;

//pub mod gl_exp;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, sierpinski!");
}

pub fn tetrahedron_raw([x, y, z]: [f32; 3], scale: f32) -> [[f32; 3]; 4] {
    let sqrt_3: f32 = 3.0f32.sqrt();
    let dy = scale * sqrt_3 * 0.5;
    let dx = scale * -0.5;
    let p0 = [x + scale, y, z];
    let p1 = [x + dx, y + dy, z];
    let p2 = [x + dx, y - dy, z];
    let p3 = [x, y, z + scale * SQRT_2];
    [p0, p1, p2, p3]
}

pub fn tetrahedron(xyz: [f32; 3], scale: f32) -> Vec<f32> {
    let [p0, p1, p2, p3] = tetrahedron_raw(xyz, scale);
    [
        p0, p2, p1, //face
        p0, p1, p3, //face
        p0, p3, p2, //face
        p1, p2, p3, //face
    ]
    .iter()
    .flatten()
    .copied()
    .collect::<Vec<f32>>()
}

fn average(p1: &[f32; 3], p2: &[f32; 3]) -> [f32; 3] {
    [(p1[0] + p2[0])*0.5, (p1[1] + p2[1])*0.5, (p1[2] + p2[2])*0.5]
}

#[wasm_bindgen]
pub fn sierpinski(base: &[f32], scale: f32, levels: u8) -> Vec<f32> {
    let base = [base[0], base[1], base[2]];
    if levels > 0 {
        tetrahedron_raw(base, scale)
            .iter()
            .flat_map(|corner| sierpinski(&average(&base, corner), scale * 0.5, levels - 1))
            .collect::<Vec<f32>>()
    } else {
        tetrahedron(base, scale)
    }
}
