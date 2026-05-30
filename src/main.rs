mod vec3;
mod ray;
mod camera;
mod hittable;
mod sphere;
mod material;
mod quad;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use::rayon::prelude::*;
use rand::Rng;

use vec3::{Vec3, Color};
use ray::Ray;
use camera::Camera;
use hittable::{Hittable, HittableList};
use material::{Material, Lambertian, Metal, Dielectric, DiffuseLight};
use sphere::Sphere;
use quad::Quad;

use crate::vec3::Point3;

// ── Constants ─────────────────────────────────────────────────────────────────

const ASPECT_RATIO:      f64 = 1.0;
const IMAGE_WIDTH:       u32 = 600;
const IMAGE_HEIGHT:      u32 = 600;
const SAMPLES_PER_PIXEL: u32 = 200;   //raise for cleaner image
const MAX_DEPTH:         u32 = 50;    // max ray bounces before forcing black

// ── Ray colour ────────────────────────────────────────────────────────────────
// hits → surface normal visualised as RGB (usual thingy debug view).
// miss  → sky gradient.

fn ray_color(ray: &Ray, world: &HittableList) -> Color {
    let mut current_ray  = *ray;
    let mut accumulated = Color::zero();
    let mut attenuation  = Color::one();

    for _ in 0..MAX_DEPTH {
        if let Some(rec) = world.hit(&current_ray, 0.001, f64::INFINITY) {
            accumulated = accumulated + attenuation * rec.material.emitted();

            if let Some((scattered, albedo)) = rec.material.scatter(&current_ray, &rec){
                attenuation = attenuation * albedo;
                current_ray = scattered;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    accumulated
}

// ── Cornell 🙃 ─────────────────────────────────────────────────────────
fn build_cornell_box() -> HittableList {
    let mut world = HittableList::new();

    // ── Materials ─────────────────────────────────────────────────────────────
    let red: Arc<dyn Material>    = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let green: Arc<dyn Material>  = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let white: Arc<dyn Material>  = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let light: Arc<dyn Material>  = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));
    let mirror: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.85, 0.9), 0.0));
    let glass: Arc<dyn Material>  = Arc::new(Dielectric::new(1.5));

    // ── Walls ─────────────────────────────────────────────────────────────────
    // each Quad: corner point, then two edge vectors
    // the box spans x∈[0,555], y∈[0,555], z∈[0,555]

    // Left wall (red) — at x=0, facing +X
    world.add(Quad::new(
        Point3::new(0.0,   0.0,   0.0),
        Vec3::new(  0.0, 555.0,   0.0),
        Vec3::new(  0.0,   0.0, 1555.0),
        Arc::clone(&red),
    ));

    // Right wall (green) — at x=555, facing -X
    world.add(Quad::new(
        Point3::new(555.0, 555.0,   0.0),
        Vec3::new(    0.0,-555.0,   0.0),
        Vec3::new(    0.0,   0.0, 1555.0),
        Arc::clone(&green),
    ));

    // Floor — at y=0, facing +Y
    world.add(Quad::new(
        Point3::new(  0.0, 0.0,   0.0),
        Vec3::new( 555.0, 0.0,   0.0),
        Vec3::new(   0.0, 0.0, 1555.0),
        Arc::clone(&white),
    ));

    // Ceiling — at y=555, facing -Y
    world.add(Quad::new(
        Point3::new(555.0, 555.0,   0.0),
        Vec3::new(-555.0,   0.0,   0.0),
        Vec3::new(   0.0,   0.0, 1555.0),
        Arc::clone(&white),
    ));

    // Back wall — at z=555, facing -Z
    world.add(Quad::new(
        Point3::new(  0.0,   0.0, 555.0),
        Vec3::new( 555.0,   0.0,   0.0),
        Vec3::new(   0.0, 555.0,   0.0),
        Arc::clone(&white),
    ));

    // ── Ceiling light ─────────────────────────────────────────────────────────
    // Centred at x∈[183,373], z∈[127,432] — roughly the classic Cornell proportions.
    world.add(Quad::new(
        Point3::new(183.0, 554.0, 127.0),
        Vec3::new( 190.0,   0.0,   0.0),
        Vec3::new(   0.0,   0.0, 305.0),
        Arc::clone(&light),
    ));

    // ── Spheres ───────────────────────────────────────────────────────────────
    // Mirror sphere — left side
    world.add(Sphere::new(
        Point3::new(165.0, 130.0, 165.0),
        130.0,
        Arc::clone(&mirror),
    ));

    // Glass sphere — right side
    world.add(Sphere::new(
        Point3::new(390.0, 100.0, 350.0),
        100.0,
        Arc::clone(&glass),
    ));

    world
}


fn main() {
    // Camera positioned to look straight down the -Z axis into the box.
    // lookfrom z=1344 is derived from: tan(FOV/2) = 277.5 / (z - 277.5) → FOV=40°
    let camera = Camera::new(
        Point3::new(277.5, 277.5, -800.0),  // lookfrom — in front of the open face
        Point3::new(277.5, 277.5,  555.0),  // lookat   — centre of back wall
        Vec3::new(  0.0,   1.0,    0.0),    // vup
        40.0,                                // vfov
        ASPECT_RATIO,
    );

    let world = build_cornell_box();

    eprintln!(
        "Rendering {}×{} at {} spp...",
        IMAGE_WIDTH, IMAGE_HEIGHT, SAMPLES_PER_PIXEL
    );

    let rows: Vec<Vec<Color>> = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .rev()
        .map(|j| {
            let mut r = rand::thread_rng();
            (0..IMAGE_WIDTH).map(|i| {
                let mut pixel = Color::zero();
                for _ in 0..SAMPLES_PER_PIXEL {
                    let u = (i as f64 + r.gen::<f64>()) / (IMAGE_WIDTH  - 1) as f64;
                    let v = (j as f64 + r.gen::<f64>()) / (IMAGE_HEIGHT - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    pixel += ray_color(&ray, &world);
                }
                pixel / SAMPLES_PER_PIXEL as f64
            }).collect()
        })
        .collect();

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