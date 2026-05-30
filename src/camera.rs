use crate::ray::Ray;
use crate::vec3::{Vec3, Point3};

pub struct Camera {
    origin:      Point3,
    lower_left:  Point3,
    horizontal:  Vec3,
    vertical:    Vec3,
}

impl Camera {
    /// * `lookfrom`  — camera position
    /// * `lookat`    — point the camera aims at
    /// * `vup`       — world "up" vector (usually Y)
    /// * `vfov`      — vertical field-of-view in degrees
    /// * `aspect`    — width / height
    pub fn new(
        lookfrom: Point3,
        lookat:   Point3,
        vup:      Vec3,
        vfov:     f64,
        aspect:   f64,
    ) -> Self {
        let theta      = vfov.to_radians();
        let h          = (theta / 2.0).tan();
        let vp_height  = 2.0 * h;
        let vp_width   = aspect * vp_height;

        // Orthonormal camera basis
        let w = (lookfrom - lookat).normalize(); // points *away* from scene
        let u = vup.cross(w).normalize();        // points right
        let v = w.cross(u);                      // points up

        let horizontal = vp_width  * u;
        let vertical   = vp_height * v;
        let lower_left = lookfrom
            - horizontal / 2.0
            - vertical   / 2.0
            - w;

        Self { origin: lookfrom, lower_left, horizontal, vertical }
    }

    /// cheeky ray through normalised image coordinates (u, v) ∈ [0,1]².
    /// u goes left→right, v goes bottom→top.
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let direction = self.lower_left
            + u * self.horizontal
            + v * self.vertical
            - self.origin;
        Ray::new(self.origin, direction)
    }
}