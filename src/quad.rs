use std::sync::Arc;
use rand::{Rng, RngCore};

use crate::hittable::{Hittable, HitRecord};
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Vec3, Point3, Color};

pub struct Quad {
    q:        Point3,   // corner
    u:        Vec3,     // first edge
    v:        Vec3,     // second edge
    normal:   Vec3,     // unit normal of the plane
    d:        f64,      // plane constant: normal · q
    w:        Vec3,     // cached: n / (n · n), for uv projection
    area:     f64,      // |u × v| — cached for Light impl
    material: Arc<dyn Material>,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, material: Arc<dyn Material>) -> Self {
        let n      = u.cross(v);
        let normal = n.normalize();
        let d      = normal.dot(q);
        let w      = n / n.dot(n);
        let area   = n.length();   // |u × v| = area of the parallelogram

        Self { q, u, v, normal, d, w, area, material }
    }
}

// ── Hittable ──────────────────────────────────────────────────────────────────

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);

        if denom.abs() < 1e-8 { return None; }

        let t = (self.d - self.normal.dot(ray.origin)) / denom;
        if t < t_min || t > t_max { return None; }

        let hit_point  = ray.at(t);
        let planar_vec = hit_point - self.q;
        let alpha      = self.w.dot(planar_vec.cross(self.v));
        let beta       = self.w.dot(self.u.cross(planar_vec));

        if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let mut rec = HitRecord {
            point:      hit_point,
            normal:     self.normal,
            t,
            front_face: true,
            material:   Arc::clone(&self.material),
        };
        rec.set_face_normal(ray, self.normal);
        Some(rec)
    }
}

// ── Light ─────────────────────────────────────────────────────────────────────
//
// Uniform sampling over the parallelogram: pick (s, t) ∈ [0,1]² independently,
// point = q + s*u + t*v.  The area PDF is simply 1/area.

impl Light for Quad {
    fn sample_surface(&self, rng: &mut dyn RngCore) -> (Point3, Vec3, Color, f64) {
        let s     = rng.gen::<f64>();
        let t     = rng.gen::<f64>();
        let point = self.q + s * self.u + t * self.v;
        let pdf   = 1.0 / self.area;

        (point, self.normal, self.material.emitted(), pdf)
    }

    fn area(&self) -> f64 { self.area }
}