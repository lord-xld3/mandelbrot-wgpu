use super::*;

#[cfg(test)]
mod lib_test {
    const MAX_ITERATIONS: u32 = 1000;
    const TILE_SIZE: usize = 256;
    const IMAGE_SIDE_LENGTH: usize = 800;

    fn measure_execution_time<F>(f: F) -> super::Duration
        where
            F: FnOnce(),
        {
            let start = super::Instant::now();
            f();
            let end = super::Instant::now();
            end - start
        }
    
    #[test]
    fn get_escape_iterations_if_not_in_set_escapes() {
        let escapes_iterations_top_left =
            super::get_escape_iterations(-2.0, 1.0, MAX_ITERATIONS, 3.0, 2).0;
        assert_ne!(escapes_iterations_top_left, MAX_ITERATIONS);

        let escapes_iterations_center_right =
            super::get_escape_iterations(1.0, 0.0, MAX_ITERATIONS, 3.0, 2).0;
        assert_ne!(escapes_iterations_center_right, MAX_ITERATIONS);
    }

    #[test]
    fn get_escape_iterations_if_in_set_stays_bounded() {
        let bounded_iterations_origin = super::get_escape_iterations(0.0, 0.0, MAX_ITERATIONS, 3.0, 2).0;
        assert_eq!(bounded_iterations_origin, MAX_ITERATIONS);

        let bounded_iterations_bulb = super::get_escape_iterations(-1.0, 0.0, MAX_ITERATIONS, 3.0, 2).0;
        assert_eq!(bounded_iterations_bulb, MAX_ITERATIONS);
    }

    #[test]
    fn get_tile_outputs_correct_length() {
        let image_size: usize = 256 * 256 * 4;

        let image = super::get_tile(0.0, 0.0, 2.0, MAX_ITERATIONS, 2, 256);
        assert_eq!(image.len(), image_size);

        let zoomed_image = super::get_tile(8476.0, 9507.0, 12.0, MAX_ITERATIONS, 2, 256);
        assert_eq!(zoomed_image.len(), image_size);
    }

    #[test]
    fn get_tile_outputs_valid_colors() {
        let image = super::get_tile(0.0, 0.0, 2.0, MAX_ITERATIONS, 2, 256);
        for n in image.clone().iter_mut() {
            assert!(n >= &mut 0 && n <= &mut 255);
        }

        let zoomed_image = super::get_tile(8476.0, 9507.0, 12.0, MAX_ITERATIONS, 2, 256);
        for n in zoomed_image.clone().iter_mut() {
            assert!(n >= &mut 0 && n <= &mut 255);
        }
    }

    #[test]
    fn measure_rendering_time() {
        let num_iterations = 10;
        let mut min_duration = std::time::Duration::MAX;
        let mut max_duration = std::time::Duration::ZERO;
        let mut total_duration = std::time::Duration::ZERO;

        println!("Number of Iterations: {} | Maximum Iterations: {} | Tile Size: {} | Image Side Length: {}", num_iterations, MAX_ITERATIONS, TILE_SIZE, IMAGE_SIDE_LENGTH);

        for i in 0..num_iterations {
            let test_duration = measure_execution_time(|| {
                // Calculate the entire screen
                let _ = super::get_tile(0.0, 0.0, 2.0, MAX_ITERATIONS, 2, IMAGE_SIDE_LENGTH);
            });

            if test_duration < min_duration {
                min_duration = test_duration;
            }

            if test_duration > max_duration {
                max_duration = test_duration;
            }

            total_duration += test_duration;

            println!("Iteration {}: Duration: {:?}", i + 1, test_duration);
        }

        let avg_duration = total_duration / num_iterations as u32;

        println!("Min: {:?} | Avg: {:?} | Max: {:?}", min_duration, avg_duration, max_duration);
    }

}
