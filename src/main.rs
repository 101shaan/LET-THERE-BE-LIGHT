mod vec3;
mod ray;
mod camera;
mod hittable;
mod sphere;

use std::fs::File;
use std::io::{BufWriter, Write};

use vec3::{Vec3, Color};
use ray::Ray;
use camera::Camera;
use hittable::{Hittable, HittableList};
use sphere::Sphere;

use crate::vec3::Point3;

// ── Sky background ────────────────────────────────────────────────────────────

fn sky_color(ray: &Ray) -> Color {
    let unit_dir = ray.direction.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    (1.0 - t) * Color::one() + t * Color::new(0.5, 0.7, 1.0)
}

// ── Ray colour ────────────────────────────────────────────────────────────────
// hits → surface normal visualised as RGB (usual thingy debug view).
// miss  → sky gradient.

fn ray_color(ray: &Ray, world: &HittableList) -> Color {
    if let Some(rec) = world.hit(ray, 0.001, f64::INFINITY) {
        // Map normal components [-1,1] → [0,1] for display
        return 0.5 * (rec.normal + Vec3::one());
    }
    sky_color(ray)
}

// this is pretty chill actually
fn main() {
    // ── Image dimensions ──────────────────────────────────────────────────────
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH:  u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
    const SAMPLES_PER_PIXEL: u32 = 1;

    // ── Camera ────────────────────────────────────────────────────────────────
    let camera = Camera::new(
        Point3::new(0.0, 0.0, 0.0),   // lookfrom
        Point3::new(0.0, 0.0, -1.0),  // lookat
        Vec3::new(0.0, 1.0, 0.0),     // vup
        90.0,                          // vfov (degrees)
        ASPECT_RATIO,
    );

    // ── Scene ─────────────────────────────────────────────────────────────────
    let mut world = HittableList::new();
    world.add(Sphere::new(Point3::new( 0.0,  0.0, -1.0),  0.5));   // main sphere
    world.add(Sphere::new(Point3::new( 0.0, -100.5, -1.0), 100.0)); // ground

    // ── PPM output ────────────────────────────────────────────────────────────
    let file = File::create("output.ppm").expect("Could not create output.ppm");
    let mut out = BufWriter::new(file);

    writeln!(out, "P3").unwrap();
    writeln!(out, "{IMAGE_WIDTH} {IMAGE_HEIGHT}").unwrap();
    writeln!(out, "255").unwrap();

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {j:4} ");
        for i in 0..IMAGE_WIDTH {
            let mut color = Color::zero();

            for _ in 0..SAMPLES_PER_PIXEL {
                let u = i as f64 / (IMAGE_WIDTH  - 1) as f64;
                let v = j as f64 / (IMAGE_HEIGHT - 1) as f64;
                let ray = camera.get_ray(u, v);
                color += ray_color(&ray, &world);
            }

            color = color / SAMPLES_PER_PIXEL as f64;
            let (r, g, b) = color.to_rgb_gamma2();
            writeln!(out, "{r} {g} {b}").unwrap();
        }
    }

    eprintln!("\nDone. → output.ppm");
}