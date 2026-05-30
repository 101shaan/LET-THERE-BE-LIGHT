mod vec3;
mod ray;
mod camera;
mod hittable;
mod sphere;
mod material;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use::rayon::prelude::*;
use rand::Rng;
use vec3::{Vec3, Color};
use ray::Ray;
use camera::Camera;
use hittable::{Hittable, HittableList};
use material::{Lambertian, Metal, Dielectric};
use sphere::Sphere;

use crate::vec3::Point3;

// ── Constants ─────────────────────────────────────────────────────────────────

const ASPECT_RATIO:      f64 = 16.0 / 9.0;
const IMAGE_WIDTH:       u32 = 800;
const IMAGE_HEIGHT:      u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;   // AA samples — raise for cleaner image
const MAX_DEPTH:         u32 = 50;    // max ray bounces before forcing black

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
    let mut current_ray  = *ray;
    let mut attenuation  = Color::one();

    for _ in 0..MAX_DEPTH {
        if let Some(rec) = world.hit(&current_ray, 0.001, f64::INFINITY) {
            if let Some((scattered, albedo)) = rec.material.scatter(&current_ray, &rec) {
                attenuation = attenuation * albedo;
                current_ray = scattered;
            } else {
                // Material absorbed the ray (shouldn't happen with our three
                // materials but correct behaviour is to return black).
                return Color::zero();
            }
        } else {
            // Ray escaped to sky — multiply accumulated attenuation by sky colour.
            return attenuation * sky_color(&current_ray);
        }
    }

    // Exceeded MAX_DEPTH — treat as fully absorbed.
    Color::zero()
}

fn build_scene() -> HittableList {
    let mut world = HittableList::new();

    let mat_ground  = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_centre  = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let mat_left    = Arc::new(Dielectric::new(1.50));          // glass
    let mat_bubble  = Arc::new(Dielectric::new(1.0 / 1.50));   // hollow — inverted IR
    let mat_right   = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.add(Sphere::new(Point3::new( 0.0, -100.5, -1.0), 100.0, mat_ground));
    world.add(Sphere::new(Point3::new( 0.0,    0.0, -1.2),   0.5, mat_centre));
    world.add(Sphere::new(Point3::new(-1.0,    0.0, -1.0),   0.5, mat_left));
    // Negative radius flips the normals — classic hollow-glass-bubble trick
    world.add(Sphere::new(Point3::new(-1.0,    0.0, -1.0),   0.4, mat_bubble));
    world.add(Sphere::new(Point3::new( 1.0,    0.0, -1.0),   0.5, mat_right));

    world
}

// this is pretty chill actually
fn main() {
    let camera = Camera::new(
        Point3::new(-2.0, 2.0, 1.0),   // lookfrom
        Point3::new( 0.0, 0.0, -1.0),  // lookat
        Vec3::new(  0.0, 1.0, 0.0),    // vup
        20.0,                           // vfov — narrow for a nice framing
        ASPECT_RATIO,
    );

    let world = build_scene();

    // ── Parallel render ───────────────────────────────────────────────────────
    // Collect one Vec<Color> per row, top→bottom.
    // rayon::par_iter parallelises across all available cores.
    // Each row gets its own thread_rng — no contention on the RNG.

    let rows: Vec<Vec<Color>> = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .rev()
        .map(|j| {
            let mut rng = rand::thread_rng();
            (0..IMAGE_WIDTH).map(|i| {
                let mut pixel = Color::zero();
                for _ in 0..SAMPLES_PER_PIXEL {
                    // Sub-pixel jitter for anti-aliasing
                    let u = (i as f64 + rng.gen::<f64>()) / (IMAGE_WIDTH  - 1) as f64;
                    let v = (j as f64 + rng.gen::<f64>()) / (IMAGE_HEIGHT - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    pixel += ray_color(&ray, &world);
                }
                pixel / SAMPLES_PER_PIXEL as f64
            }).collect()
        })
        .collect();

    // ── Write PPM ─────────────────────────────────────────────────────────────
    let file = File::create("output.ppm").expect("Could not create output.ppm");
    let mut out = BufWriter::new(file);

    writeln!(out, "P3").unwrap();
    writeln!(out, "{IMAGE_WIDTH} {IMAGE_HEIGHT}").unwrap();
    writeln!(out, "255").unwrap();

    for row in rows {
        for color in row {
            let (r, g, b) = color.to_rgb_gamma2();
            writeln!(out, "{r} {g} {b}").unwrap();
        }
    }

    eprintln!("Done. → output.ppm");
}