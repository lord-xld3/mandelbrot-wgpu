#[cfg(test)]
use std::time::{Instant, Duration};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod utils;
use wasm_bindgen::prelude::*;

use std::ops::Range;
use rayon::prelude::*;
use itertools_num::linspace;
use num::complex::Complex64;

// radius should be >= 3.0 for smoothed coloring
const ESCAPE_RADIUS: f64 = 3.0;
const NUM_COLOR_CHANNELS: usize = 4;
const PALETTE: colorous::Gradient = colorous::TURBO;
const PALETTE_SCALE_FACTOR: f64 = 20.0;

// how many iterations does it take to escape?
fn get_escape_iterations(
    x: &f64,
    y: &f64,
    max_iterations: u32,
    escape_radius: f64,
    exponent: u32,
) -> (u32, f64) {
    let c: Complex64 = Complex64::new(*x, *y);
    let mut z: Complex64 = c;

    let mut iter: u32 = 0;

    while z.norm() < escape_radius && iter < max_iterations {
        iter += 1;
        z = z.powu(exponent) + c;
    }

    (iter, z.norm())
}

fn check_range(
    re_range: &Box<[f64]>,
    im_range: &Box<[f64]>,
    row_range: &Range<usize>,
    col_range: &Range<usize>,
    max_iterations: u32,
    escape_radius: f64,
    exponent: u32,
    mask: &mut Vec<Vec<(u32, f64)>>,
) {
    mask.par_iter_mut().enumerate().for_each(|(i, row)| {
        if row_range.contains(&i) {
            row.par_iter_mut().enumerate().for_each(|(j, pixel)| {
                if col_range.contains(&j) {
                    *pixel = get_escape_iterations(
                        &re_range[j],
                        &im_range[i],
                        max_iterations,
                        escape_radius,
                        exponent,
                    );
                }
            });
        }
    });
}

// map leaflet coordinates to complex plane
// I have no idea where these "magic" values come from but it works
fn map_coordinates(x: f64, y: f64, z: f64, tile_size: usize) -> (f64, f64) {
    let scale_factor = tile_size as f64 / 128.5;
    let d: f64 = 2.0f64.powf(z - 2.0);
    let re = x / d * scale_factor - 4.0;
    let im = y / d * scale_factor - 4.0;

    (re, im)
}

#[wasm_bindgen]
pub fn get_tile(
    center_x: f64,
    center_y: f64,
    z: f64,
    max_iterations: u32,
    exponent: u32,
    tile_len: usize,
) -> Vec<u8> {
    
    let output_size: usize = tile_len * tile_len * NUM_COLOR_CHANNELS;
    let scaled_max_iterations: usize = (max_iterations * PALETTE_SCALE_FACTOR as u32) as usize;

    // Canvas API expects UInt8ClampedArray
    let mut img: Vec<u8> = vec![0; output_size]; // [ r, g, b, a, r, g, b, a, r, g, b, a... ]
    let mut mask: Vec<Vec<(u32, f64)>> = 
    vec![
        vec![(0, 0.0); tile_len];
        tile_len // Inner vec * tile_len
    ];

    let (re_min, im_min) = map_coordinates(center_x, center_y, z, tile_len);
    let (re_max, im_max) = map_coordinates(center_x + 1.0, center_y + 1.0, z, tile_len);

    let re_range = linspace(re_min, re_max, tile_len).collect::<Box<_>>();
    let im_range = linspace(im_min, im_max, tile_len).collect::<Box<_>>();
    
    // Get the top-left pixel as a reference
    mask[0][0] = get_escape_iterations(
        &re_range[0],
        &im_range[0],
        max_iterations,
        ESCAPE_RADIUS,
        exponent,
    );

    let ref_iter: u32 = mask[0][0].0;

    let mut check_closure = |row_range: Range<usize>, col_range: Range<usize>| {
        check_range(
            &re_range,
            &im_range,
            &row_range,
            &col_range,
            max_iterations,
            ESCAPE_RADIUS,
            exponent,
            &mut mask,
        );
    };

    check_closure(1..tile_len, 0..1); // top
    check_closure(1..tile_len, tile_len - 1..tile_len); // bottom
    check_closure(0..1, 1..tile_len); // left
    check_closure(tile_len - 1..tile_len, 1..tile_len); // right

    //TODO: Check if the borders of the mask all have the same amount of iterations
 
    if true {
        if ref_iter == max_iterations {
            img.par_chunks_mut(NUM_COLOR_CHANNELS)
            .enumerate()
            .for_each(|(_, chunk)| {
                chunk[0] = 0;
                chunk[1] = 0;
                chunk[2] = 0;
                chunk[3] = 255;
            });
        } else {
            // Fill the mask excluding the borders by interpolating the f64 values from the borders
            mask.par_iter_mut().skip(1).take(tile_len - 2)
            .for_each(|row| {
                let lin_row = 
                    linspace(row[0].1, row[tile_len - 1].1, tile_len)
                    .collect::<Box<_>>();
                row.par_iter_mut().enumerate().skip(1).take(tile_len - 2)
                .for_each(|(j, col)| {
                    col.0 = ref_iter;
                    col.1 = lin_row[j];
                })
            });

            img.par_chunks_mut(NUM_COLOR_CHANNELS)
            .enumerate()
            .for_each(|(index, chunk)| {
                let i = index / tile_len;
                let j = index % tile_len;

                let smoothed_value = f64::from(mask[i][j].0)
                    - ((mask[i][j].1.ln() / ESCAPE_RADIUS.ln()).ln() / f64::from(exponent).ln());

                let scaled_value = (smoothed_value * PALETTE_SCALE_FACTOR) as usize;
                let color = PALETTE.eval_rational(scaled_value, scaled_max_iterations);

                let [r, g, b] = color.as_array();
                chunk[0] = r;
                chunk[1] = g;
                chunk[2] = b;
                chunk[3] = 255;
            });
        }
    } else {
        // TODO: Recursively sub-divide the tile until the borders are all the same amount of iterations
    }

    img
}

#[wasm_bindgen]
pub fn init() {
    utils::init();
}

#[test]
fn avg_runtime() {
    const RUN_ITERATIONS: usize = 100;
    let mut durations: [Duration; RUN_ITERATIONS] = [Duration::ZERO; RUN_ITERATIONS];
    
    for index in 0..RUN_ITERATIONS {
        let start = Instant::now();
        let _ = get_tile(0.0, 0.0, 1.0, 2000, 2, 2000);
        println!("Iteration {} complete in {:?}", index, start.elapsed());
        durations[index as usize] = start.elapsed();
    }
    println!("Min: {:?}ms, Avg: {:?}ms, Max: {:?}ms", durations.iter().min().unwrap(), durations.iter().sum::<Duration>() / RUN_ITERATIONS as u32, durations.iter().max().unwrap());
}