//! #17317: own 60 px strokes on an 800x400 logical frame, sampled without 8-bit quantization.

use photocraft_color::PixelFormat;
use photocraft_raster::Surface;

use crate::{BrushSettings, StrokePoint, render_stroke};

fn paint(noise: bool, hardness: f32, seed: u64) -> Surface {
    let brush = BrushSettings { size: 60.0, hardness, noise, seed, ..BrushSettings::default() };
    let mut surface = Surface::new(PixelFormat::RGBA32F);
    render_stroke(&mut surface, &brush, &[StrokePoint::new(100.0, 100.0, 1.0), StrokePoint::new(400.0, 100.0, 1.0)], None, false, 1.0);
    surface
}

#[test]
fn soft_noise_is_light_and_mean_neutral_over_the_complete_stroke_rectangle() {
    let plain = paint(false, 0.0, 17317);
    let noisy = paint(true, 0.0, 17317);
    let (mut sum, mut squares, mut count) = (0.0, 0.0, 0usize);
    // Fixed complete soft-tip support rectangle: 51.792 px reach plus pixel sampling,
    // around the whole100..400 stroke. Include every pixel, not only favourable alpha bins.
    for y in 48..152 {
        for x in 48..452 {
            let delta = 255.0 * f64::from(plain.rgba(x, y)[3] - noisy.rgba(x, y)[3]);
            sum += delta;
            squares += delta * delta;
            count += 1;
        }
    }
    let mean = sum / count as f64;
    let deviation = (squares / count as f64 - mean * mean).max(0.0).sqrt();
    assert!(mean.abs() <= 2.0, "Noise minus plain mean {mean}");
    assert!((10.0..=20.0).contains(&deviation), "Noise minus plain std {deviation}");
}

#[test]
fn noise_leaves_the_hard_interior_and_the_empty_frame_unchanged() {
    let plain = paint(false, 1.0, 17317);
    let noisy = paint(true, 1.0, 17317);
    for y in 80..120 {
        for x in 100..400 {
            assert_eq!(noisy.rgba(x, y), plain.rgba(x, y), "hard interior {x},{y}");
        }
    }
    for (x, y) in [(0, 0), (100, 40), (400, 160), (799, 399)] {
        assert_eq!(noisy.rgba(x, y), plain.rgba(x, y));
    }
}

#[test]
fn noise_replays_its_seed_and_different_seeds_get_different_grain() {
    let first = paint(true, 0.0, 1);
    let replay = paint(true, 0.0, 1);
    let other = paint(true, 0.0, 2);
    let plain_first = paint(false, 0.0, 1);
    let plain_other = paint(false, 0.0, 2);
    let mut changed = false;
    for y in 48..152 {
        for x in 48..452 {
            assert_eq!(first.rgba(x, y), replay.rgba(x, y));
            assert_eq!(plain_first.rgba(x, y), plain_other.rgba(x, y), "Noise off stays seed-independent");
            changed |= first.rgba(x, y) != other.rgba(x, y);
        }
    }
    assert!(changed, "different seeds must change the grain");
}
