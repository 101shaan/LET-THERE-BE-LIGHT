use std::f64::consts::PI;
use crate::hittable::{Hittable, HitRecord};
use crate::ray::Ray;
use crate::vec3::{Vec3, Point3};

pub struct RotateY {
    object:   Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
}

impl RotateY {
    pub fn new(object: impl Hittable + 'static, angle_deg: f64) -> Self {
        let radians   = angle_deg * PI / 180.0;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        Self { object: Box::new(object), sin_theta, cos_theta }
    }

    fn rotate_vec(&self, v: Vec3) -> Vec3 {
        Vec3::new(
             self.cos_theta * v.x + self.sin_theta * v.z,
             v.y,
            -self.sin_theta * v.x + self.cos_theta * v.z,
        )
    }

    fn unrotate_vec(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.cos_theta * v.x - self.sin_theta * v.z,
            v.y,
            self.sin_theta * v.x + self.cos_theta * v.z,
        )
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Rotate ray into object local space
        let local_ray = Ray::new(
            self.rotate_vec(ray.origin),
            self.rotate_vec(ray.direction),
        );

        // Hit in local space
        let mut rec = self.object.hit(&local_ray, t_min, t_max)?;

        // Rotate hit point and normal back to world space
        rec.point  = self.unrotate_vec(rec.point);
        rec.normal = self.unrotate_vec(rec.normal);

        Some(rec)
    }
}