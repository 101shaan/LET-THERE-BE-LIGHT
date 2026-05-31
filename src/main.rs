mod vec3;
mod ray;
mod camera;
mod hittable;
mod sphere;
mod material;
mod quad;
mod transform;
mod light;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use rayon::prelude::*;
use rand::Rng;
use rand::RngCore;

use vec3::{Vec3, Color};
use ray::Ray;
use camera::Camera;
use hittable::{Hittable, HittableList};
use light::{Light, LightList};
use material::{Material, Lambertian, DiffuseLight};
use quad::Quad;
use transform::RotateY;
use crate::vec3::Point3;

// ── Constants ─────────────────────────────────────────────────────────────────

const ASPECT_RATIO:      f64 = 1.0;
const IMAGE_WIDTH:       u32 = 1080;
const IMAGE_HEIGHT:      u32 = 1080;
const SAMPLES_PER_PIXEL: u32 = 1000;
const MAX_DEPTH:         u32 = 50;

// ── Integrator ────────────────────────────────────────────────────────────────
//
// Next Event Estimation (NEE) path integrator.
//
// At each diffuse bounce we do two things:
//
//   1. Direct (NEE):  sample a point on a light, cast a shadow ray, accumulate
//      the direct contribution weighted by the BSDF, geometry term, and light PDF.
//
//   2. Indirect:  scatter a new ray via the BSDF as usual and continue the path.
//
// To avoid double-counting emitted radiance we only add `emitted()` on the very
// first intersection (camera ray hitting a light directly).  On all subsequent
// bounces the emitter contribution is handled exclusively through NEE.
// This is the "direct only on first hit" convention — simple, correct, and
// debug-friendly before MIS is added.

fn ray_color(
    ray:    &Ray,
    world:  &HittableList,
    lights: &LightList,
    rng:    &mut impl Rng,
    depth:  u32,
) -> Color {
    if depth == 0 { return Color::zero(); }

    let rec = match world.hit(ray, 0.001, f64::INFINITY) {
        None      => return Color::zero(),
        Some(rec) => rec,
    };

    // Light hit — return emission directly (camera ray seeing the quad).
    // Bounce rays never reach here because ray_color_bounce zeros emission.
    let (scattered, albedo) = match rec.material.scatter(ray, &rec) {
        None         => return rec.material.emitted(),
        Some(result) => result,
    };

    // NEE: sample a light, cast shadow ray, accumulate direct contribution.
    let direct = if let Some(ls) = lights.sample(rec.point, rng) {
        let cos_surface = rec.normal.dot(ls.direction);
        let shadow_ray  = Ray::new(rec.point, ls.direction);
        let in_shadow   = world.hit(&shadow_ray, 0.001, ls.distance - 1e-4).is_some();
        if !in_shadow && cos_surface > 0.0 {
            albedo * ls.emission * (cos_surface / ls.pdf)
        } else {
            Color::zero()
        }
    } else {
        Color::zero()
    };

    // Indirect: scatter and recurse, but suppress emitted() on light hits
    // to avoid double-counting what NEE already paid for.
    let indirect = albedo * ray_color_bounce(&scattered, world, lights, rng, depth - 1);

    direct + indirect
}

// Identical to ray_color but returns black when scatter() returns None.
// Used for indirect bounces so that a bounce ray landing on the light
// doesn't add emitted() on top of what NEE already contributed.
fn ray_color_bounce(
    ray:    &Ray,
    world:  &HittableList,
    lights: &LightList,
    rng:    &mut impl Rng,
    depth:  u32,
) -> Color {
    if depth == 0 { return Color::zero(); }

    let rec = match world.hit(ray, 0.001, f64::INFINITY) {
        None      => return Color::zero(),
        Some(rec) => rec,
    };

    // Hit a light on a bounce — return black, NEE already counted this.
    let (scattered, albedo) = match rec.material.scatter(ray, &rec) {
        None         => return Color::zero(),
        Some(result) => result,
    };

    let direct = if let Some(ls) = lights.sample(rec.point, rng) {
        let cos_surface = rec.normal.dot(ls.direction);
        let shadow_ray  = Ray::new(rec.point, ls.direction);
        let in_shadow   = world.hit(&shadow_ray, 0.001, ls.distance - 1e-4).is_some();
        if !in_shadow && cos_surface > 0.0 {
            albedo * ls.emission * (cos_surface / ls.pdf)
        } else {
            Color::zero()
        }
    } else {
        Color::zero()
    };

    let indirect = albedo * ray_color_bounce(&scattered, world, lights, rng, depth - 1);

    direct + indirect
}

// ── Scene geometry ────────────────────────────────────────────────────────────

fn build_box(
    world: &mut HittableList,
    p_min: Point3,
    p_max: Point3,
    mat:   Arc<dyn Material>,
) {
    let (x0, y0, z0) = (p_min.x, p_min.y, p_min.z);
    let (x1, y1, z1) = (p_max.x, p_max.y, p_max.z);

    world.add(Quad::new(Point3::new(x0,y0,z1), Vec3::new(x1-x0,0.0,0.0), Vec3::new(0.0,y1-y0,0.0), Arc::clone(&mat))); // front
    world.add(Quad::new(Point3::new(x1,y0,z0), Vec3::new(x0-x1,0.0,0.0), Vec3::new(0.0,y1-y0,0.0), Arc::clone(&mat))); // back
    world.add(Quad::new(Point3::new(x0,y0,z0), Vec3::new(0.0,0.0,z1-z0), Vec3::new(0.0,y1-y0,0.0), Arc::clone(&mat))); // left
    world.add(Quad::new(Point3::new(x1,y0,z1), Vec3::new(0.0,0.0,z0-z1), Vec3::new(0.0,y1-y0,0.0), Arc::clone(&mat))); // right
    world.add(Quad::new(Point3::new(x0,y1,z1), Vec3::new(x1-x0,0.0,0.0), Vec3::new(0.0,0.0,z0-z1), Arc::clone(&mat))); // top
    world.add(Quad::new(Point3::new(x0,y0,z0), Vec3::new(x1-x0,0.0,0.0), Vec3::new(0.0,0.0,z1-z0), Arc::clone(&mat))); // bottom
}

// Returns both the hittable world and the light list.
// The ceiling light quad is added to both: world (so shadow rays can hit it)
// and lights (so NEE can sample it).  The light list only holds emitters —
// no wall geometry.
fn build_cornell_box() -> (HittableList, LightList) {
    let mut world  = HittableList::new();
    let mut lights = LightList::new();

    // ── Materials ─────────────────────────────────────────────────────────────
    let red:   Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let green: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let white: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let blue:  Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.1,  0.2,  0.6)));
    let light: Arc<dyn Material> = Arc::new(DiffuseLight::new(Color::new(4.0, 4.0, 4.0)));

    // ── Walls ─────────────────────────────────────────────────────────────────

    // Left wall (red)
    world.add(Quad::new(
        Point3::new(0.0,   0.0,   -800.0),
        Vec3::new(  0.0, 555.0,    0.0),
        Vec3::new(  0.0,   0.0, 1355.0),
        Arc::clone(&red),
    ));

    // Right wall (green)
    world.add(Quad::new(
        Point3::new(555.0, 555.0,   -800.0),
        Vec3::new(    0.0,-555.0,    0.0),
        Vec3::new(    0.0,   0.0, 1355.0),
        Arc::clone(&green),
    ));

    // Floor
    world.add(Quad::new(
        Point3::new(  0.0, 0.0,   -800.0),
        Vec3::new( 555.0, 0.0,    0.0),
        Vec3::new(   0.0, 0.0, 1355.0),
        Arc::clone(&white),
    ));

    // Ceiling
    world.add(Quad::new(
        Point3::new(555.0, 555.0,   -800.0),
        Vec3::new(-555.0,   0.0,    0.0),
        Vec3::new(   0.0,   0.0, 1355.0),
        Arc::clone(&white),
    ));

    // Back wall
    world.add(Quad::new(
        Point3::new(  0.0,   0.0, 555.0),
        Vec3::new( 555.0,   0.0,   0.0),
        Vec3::new(   0.0, 555.0,   0.0),
        Arc::clone(&white),
    ));

    // Front wall (blue, behind camera)
    world.add(Quad::new(
        Point3::new(0.0,   0.0, -800.0),
        Vec3::new(555.0,   0.0,    0.0),
        Vec3::new(  0.0, 555.0,    0.0),
        Arc::clone(&blue),
    ));

    // ── Ceiling light ─────────────────────────────────────────────────────────
    // Added to both world and lights.
    // The Quad is constructed identically in both so NEE samples the same
    // geometry that the ray tracer can intersect.
    let light_quad = Arc::new(Quad::new(
        Point3::new(183.0, 554.0, 127.0),
        Vec3::new( 190.0,   0.0,   0.0),
        Vec3::new(   0.0,   0.0, 305.0),
        Arc::clone(&light),
    ));

    world.add(Quad::new(
        Point3::new(183.0, 554.0, 127.0),
        Vec3::new( 190.0,   0.0,   0.0),
        Vec3::new(   0.0,   0.0, 305.0),
        Arc::clone(&light),
    ));
    lights.add(light_quad as Arc<dyn Light>);

    // ── Boxes ─────────────────────────────────────────────────────────────────

    // Tall box — back right, rotated -18°
    world.add(RotateY::new(
        {
            let mut tmp = HittableList::new();
            build_box(&mut tmp,
                Point3::new(170.0, 0.0, 330.0),
                Point3::new(330.0, 330.0, 470.0),
                Arc::clone(&white));
            tmp
        },
        -18.0,
    ));

    // Short box — front left, rotated 25°
    world.add(RotateY::new(
        {
            let mut tmp = HittableList::new();
            build_box(&mut tmp,
                Point3::new(150.0, 0.0, -40.0),
                Point3::new(310.0, 165.0, 80.0),
                Arc::clone(&white));
            tmp
        },
        25.0,
    ));

    (world, lights)
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let camera = Camera::new(
        Point3::new(277.5, 277.5, -800.0),
        Point3::new(277.5, 277.5,  555.0),
        Vec3::new(  0.0,   1.0,    0.0),
        40.0,
        ASPECT_RATIO,
    );

    let (world, lights) = build_cornell_box();

    eprintln!(
        "Rendering {}×{} at {} spp (NEE)...",
        IMAGE_WIDTH, IMAGE_HEIGHT, SAMPLES_PER_PIXEL
    );

    let rows: Vec<Vec<Color>> = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .rev()
        .map(|j| {
            let mut rng = rand::thread_rng();
            (0..IMAGE_WIDTH).map(|i| {
                let mut pixel = Color::zero();
                for _ in 0..SAMPLES_PER_PIXEL {
                    let u   = (i as f64 + rng.gen::<f64>()) / (IMAGE_WIDTH  - 1) as f64;
                    let v   = (j as f64 + rng.gen::<f64>()) / (IMAGE_HEIGHT - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    pixel  += ray_color(&ray, &world, &lights, &mut rng, MAX_DEPTH);
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