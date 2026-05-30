use crate::vec3::{Vec3, Point3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin:    Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// P(t) = origin + t * direction
    pub fn at(self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}