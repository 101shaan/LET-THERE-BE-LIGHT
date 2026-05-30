use std::sync::Arc;
use crate::hittable::{Hittable, HitRecord};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;

pub struct Sphere {
    pub center:   Point3,
    pub radius:   f64,
    pub material: Arc<dyn Material>,   // ← NEW
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self { center, radius, material }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a  = ray.direction.length_squared();
        let h  = ray.direction.dot(oc);
        let c  = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 { return None; }

        let sqrt_d = discriminant.sqrt();
        let mut root = (-h - sqrt_d) / a;
        if root <= t_min || root >= t_max {
            root = (-h + sqrt_d) / a;
            if root <= t_min || root >= t_max { return None; }
        }

        let point          = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;

        let mut rec = HitRecord {
            point,
            normal:     outward_normal,
            t:          root,
            front_face: true,
            material:   Arc::clone(&self.material),   // ← NEW
        };
        rec.set_face_normal(ray, outward_normal);
        Some(rec)
    }
}