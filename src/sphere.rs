use crate::hittable::{Hittable, HitRecord};
use crate::ray::Ray;
use crate::vec3::Point3;

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;

        // quadratic coefficients for |ray.at(t) - center|² = radius²
        // optimised form: b = 2h where h = dot(d, oc), so discriminant = h²-ac
        let a  = ray.direction.length_squared();
        let h  = ray.direction.dot(oc);         // half-b
        let c  = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 { return None; }

        let sqrt_d = discriminant.sqrt();

        // find the nearest root in [t_min, t_max]
        let mut root = (-h - sqrt_d) / a;
        if root <= t_min || root >= t_max {
            root = (-h + sqrt_d) / a;
            if root <= t_min || root >= t_max {
                return None;
            }
        }

        let point          = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;

        let mut rec = HitRecord {
            point,
            normal:     outward_normal, // will be overwritten below
            t:          root,
            front_face: true,           // will be overwritten below
        };
        rec.set_face_normal(ray, outward_normal);

        Some(rec)
    }
}