use std::sync::Arc;
use crate::hittable::{Hittable, HitRecord};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Vec3, Point3};

pub struct Quad {
    q:        Point3,   // corner
    u:        Vec3,     // first edge
    v:        Vec3,     // second edge
    normal:   Vec3,     // unit normal of the plane
    d:        f64,      // plane constant: normal · q
    w:        Vec3,     // cached: normal / (normal · normal), for uv projection
    material: Arc<dyn Material>,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, material: Arc<dyn Material>) -> Self {
        let n      = u.cross(v);          // non-unit normal
        let normal = n.normalize();
        let d      = normal.dot(q);
        let w      = n / n.dot(n);        // projection denominator, cached

        Self { q, u, v, normal, d, w, material }
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);

        // Ray is parallel to the plane — no intersection.
        if denom.abs() < 1e-8 { return None; }

        // t at which ray hits the infinite plane.
        let t = (self.d - self.normal.dot(ray.origin)) / denom;
        if t < t_min || t > t_max { return None; }

        // Hit point on the plane. Check it's inside the quad's bounds using planar coordinates (α, β) ∈ [0,1]².
        let hit_point   = ray.at(t);
        let planar_vec  = hit_point - self.q;
        let alpha       = self.w.dot(planar_vec.cross(self.v));
        let beta        = self.w.dot(self.u.cross(planar_vec));

        if !(0.0..=1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) {
            return None;
        }

        let mut rec = HitRecord {
            point:    hit_point,
            normal:   self.normal,
            t,
            front_face: true,
            material: Arc::clone(&self.material),
        };
        rec.set_face_normal(ray, self.normal);
        Some(rec)
    }
}