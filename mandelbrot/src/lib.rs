#[cfg(test)]
use std::time::Instant;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod utils;
use wasm_bindgen::prelude::*;

use rayon::prelude::*;
use itertools_num::linspace;
use num::complex::Complex64;


// how many iterations does it take to escape?
fn get_escape_iterations(
    x: &f64,
    y: &f64,
    max_iterations: u32,
    escape_radius: f64,
    exponent: u32,
) -> (u32, Complex64) {
    let c: Complex64 = Complex64::new(*x, *y);
    let mut z: Complex64 = c;

    let mut iter: u32 = 0;

    while z.norm() < escape_radius && iter < max_iterations {
        iter += 1;
        z = z.powu(exponent) + c;
    }

    (iter, z)
}

fn check_range(
    ref_iter: u32,
    re_range: &Box<[f64]>,
    im_range: &Box<[f64]>,
    max_iterations: u32,
    escape_radius: f64,
    exponent: u32,
    mask: &mut Box<[Box<[(u32, num::Complex<f64>)]>]>,
) -> bool {
    let mut all_same = true;

    for (i, x) in re_range.iter().enumerate() {
        for (j, y) in im_range.iter().enumerate() {
            let (iter, complex) = get_escape_iterations(x, y, max_iterations, escape_radius, exponent);
            mask[i][j] = (iter, complex);

            if iter != ref_iter {
                all_same = false;
            }
        }
    }
    all_same
}

// map leaflet coordinates to complex plane
fn map_coordinates(x: f64, y: f64, z: f64, tile_size: usize) -> (f64, f64) {
    let scale_factor = tile_size as f64 / 128.5;
    let d: f64 = 2.0f64.powf(z - 2.0);
    let re = x / d * scale_factor - 4.0;
    let im = y / d * scale_factor - 4.0;

    (re, im)
}

const NUM_COLOR_CHANNELS: usize = 4;

#[wasm_bindgen]
pub fn get_tile(
    center_x: f64,
    center_y: f64,
    z: f64,
    max_iterations: u32,
    exponent: u32,
    image_side_length: usize,
) -> Box<[u8]> {
    let palette = colorous::TURBO;
    let output_size: usize = image_side_length * image_side_length * NUM_COLOR_CHANNELS;

    // Canvas API expects UInt8ClampedArray
    let mut img: Box<[u8]> = vec![0; output_size].into_boxed_slice(); // [ r, g, b, a, r, g, b, a, r, g, b, a... ]
    let mut mask: Box<[Box<[(u32, num::Complex<f64>)]>]> = 
    vec![
        vec![
            (0, Complex64::new(0.0, 0.0)); image_side_length
        ].into_boxed_slice();
        image_side_length // Inner vec * image_side_length
    ].into_boxed_slice();

    let (re_min, im_min) = map_coordinates(center_x, center_y, z, image_side_length);
    let (re_max, im_max) = map_coordinates(center_x + 1.0, center_y + 1.0, z, image_side_length);

    let re_range = linspace(re_min, re_max, image_side_length).collect::<Box<_>>();
    let im_range = linspace(im_min, im_max, image_side_length).collect::<Box<_>>();

    let palette_scale_factor = 20.0;
    let scaled_max_iterations = (max_iterations * palette_scale_factor as u32) as usize;

    // radius has to be >=3 for color smoothing
    let escape_radius = 3.0;

    // Get the top-left pixel as a reference
    let (ref_iter, ref_complex) = get_escape_iterations(
        &re_range[0],
        &im_range[0],
        max_iterations,
        escape_radius,
        exponent,
    );
    
    let all_true = {
        let im_start: Box<[f64]> = Box::new([im_range[0]]);
        let im_end: Box<[f64]> = Box::new([im_range[im_range.len() - 1]]);
        let re_start: Box<[f64]> = Box::new([re_range[0]]);
        let re_end: Box<[f64]> = Box::new([re_range[re_range.len() - 1]]);
    
        let result = {
            
            let mut check_closure = |x, y| {
                check_range(ref_iter, x, y, max_iterations, escape_radius, exponent, &mut mask)
            };
    
            check_closure(&re_range, &im_start) && // top
            check_closure(&re_range, &im_end) &&   // bottom
            check_closure(&re_start, &im_range) && // left
            check_closure(&re_end, &im_range)      // right
        };

        result
    };
    

    if all_true {
        // Parallelize the loop using Rayon
        img.par_chunks_exact_mut(NUM_COLOR_CHANNELS)
            .enumerate()
            .for_each(|(_index, pixel)| {
                let smoothed_value = f64::from(ref_iter)
                    - ((ref_complex.norm().ln() / escape_radius.ln()).ln()
                        / f64::from(exponent).ln());
                let scaled_value = (smoothed_value * palette_scale_factor) as usize;
                let color = palette.eval_rational(scaled_value, scaled_max_iterations);

                // Unpack the array and extract the color values into the pixel array
                let [r, g, b] = color.as_array();
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
                pixel[3] = 255;
            });
    }

    img
}

#[wasm_bindgen]
pub fn init() {
    utils::init();
}

#[test]
fn avg_runtime(){
    let run_iterations: u32 = 20;
    let start = Instant::now();
    for _ in 0..run_iterations {
        let _ = get_tile(0.0, 0.0, 1.0, 2000, 2, 2000);
    }
    println!("Average runtime: {:?}", start.elapsed() / run_iterations);
}