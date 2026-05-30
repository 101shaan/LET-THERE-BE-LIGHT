mod vec3;
mod ray;

use std::fs::File;
use std::io::{BufWriter, Write};

use vec3::{Vec3, Color};
use ray::Ray;

// ── Sky background ────────────────────────────────────────────────────────────
// A simple vertical lerp: white at the bottom, blue at the top.
// This is our ground-truth output for Step 1 - if the gradient looks right,
// the math, the ray casting, and the PPM writer all work

fn ray_color(ray: &Ray) -> Color {
    let unit_dir = ray.direction.normalize();
    let t = 0.5 * (unit_dir.y + 1.0); // remap [-1,1] → [0,1]
    let white = Color::new(1.0, 1.0, 1.0);
    let sky_blue = Color::new(0.5, 0.7, 1.0);
    (1.0 - t) * white + t * sky_blue
}
// this is pretty chill actually
fn main() {
    // ── Image dimensions ──────────────────────────────────────────────────────
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH:  u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;

    // ── Camera / viewport ─────────────────────────────────────────────────────
    // minimal camera here will be replaced dw
    let viewport_height = 2.0_f64;
    let viewport_width  = ASPECT_RATIO * viewport_height;
    let focal_length    = 1.0_f64;

    let origin     = Vec3::zero();
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical   = Vec3::new(0.0, viewport_height, 0.0);
    // bottom-left corner of the viewport plane
    let lower_left = origin
        - horizontal / 2.0
        - vertical   / 2.0
        - Vec3::new(0.0, 0.0, focal_length);

    // ── PPM output ────────────────────────────────────────────────────────────
    let file   = File::create("output.ppm").expect("Could not create output.ppm");
    let mut out = BufWriter::new(file);

    writeln!(out, "P3").unwrap();
    writeln!(out, "{IMAGE_WIDTH} {IMAGE_HEIGHT}").unwrap();
    writeln!(out, "255").unwrap();

    // Rows top → bottom (PPM origin is top-left)
    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {j:4} ");
        for i in 0..IMAGE_WIDTH {
            let u = i as f64 / (IMAGE_WIDTH  - 1) as f64;
            let v = j as f64 / (IMAGE_HEIGHT - 1) as f64;

            let direction = lower_left + u * horizontal + v * vertical - origin;
            let ray = Ray::new(origin, direction);

            let color = ray_color(&ray);
            let (r, g, b) = color.to_rgb_gamma2();
            writeln!(out, "{r} {g} {b}").unwrap();
        }
    }

    eprintln!("\nDone. → output.ppm");
}