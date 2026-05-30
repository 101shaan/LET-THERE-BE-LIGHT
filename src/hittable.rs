use crate::ray::Ray;
use crate::vec3::{Vec3, Point3};

/// everything the path tracer needs to know about a ray–surface intersection
#[derive(Clone, Copy)]
pub struct HitRecord {
    pub point:      Point3,
    pub normal:     Vec3,   // always points against the incoming ray
    pub t:          f64,
    pub front_face: bool,   // did the ray hit the outside face?
}

impl HitRecord {
    /// set the normal so it always opposes the ray direction
    /// store which face was hit so materials can use it
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

/// any geometry the ray can intersect implements this trait
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

// ── HittableList ──────────────────────────────────────────────────────────────

pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result  = None;

        for object in &self.objects {
            if let Some(rec) = object.hit(ray, t_min, closest) {
                closest = rec.t;
                result  = Some(rec);
            }
        }

        result
    }
}