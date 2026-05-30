use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec3::{Color, Vec3};
use rand::Rng;

pub trait Material: Send + Sync {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
    
    fn emitted (&self) -> Color {
        Color::zero()
    }
}

// ── Lambertian (ideal diffuse) ────────────────────────────────────────────────

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self { Self { albedo } }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_dir = rec.normal + Vec3::random_unit_vector();

        // degenerate scatter direction guard — catches near-zero vectors
        // that would produce NaN normals further down the path
        if scatter_dir.near_zero() {
            scatter_dir = rec.normal;
        }

        let scattered = Ray::new(rec.point, scatter_dir);
        Some((scattered, self.albedo))
    }
}

// ── Metal (mirror with optional roughness) ────────────────────────────────────

pub struct Metal {
    pub albedo: Color,
    /// Fuzz radius [0, 1]. 0 = perfect mirror. 1 = very rough.
    pub fuzz:   f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz: fuzz.clamp(0.0, 1.0) }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let reflected = Vec3::reflect(ray_in.direction.normalize(), rec.normal);
        // perturb the reflection direction by a random point inside a unit sphere
        // scaled by fuzz. fuzz=0 → perfect mirror. remember this
        let scattered = Ray::new(
            rec.point,
            reflected + self.fuzz * Vec3::random_unit_vector(),
        );

        // if the fuzz pushes the ray below the surface, absorb it
        if scattered.direction.dot(rec.normal) > 0.0 {
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}

// ── Dielectric (glass / refractive) ──────────────────────────────────────────

pub struct Dielectric {
    /// index of refraction (glass ≈ 1.5, water ≈ 1.33, diamond ≈ 2.4)
    pub ir: f64,
}

impl Dielectric {
    pub fn new(ir: f64) -> Self { Self { ir } }

    /// schlick's approximation for reflectance at grazing angles.
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        // glass attenuates nothing - pure white attenuation.
        let attenuation = Color::one();

        // ratio flips depending on whether we're entering or leaving the medium.
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir   // air → glass
        } else {
            self.ir          // glass → air
        };

        let unit_dir  = ray_in.direction.normalize();
        let cos_theta = (-unit_dir).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        // total internal reflection: refraction is geometrically impossible.
        let must_reflect = refraction_ratio * sin_theta > 1.0;

        // schlick: probabilistically reflect at grazing angles even when
        // refraction is possible -- this is what gives glass its sheen.
        let direction = if must_reflect
            || Self::reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen::<f64>()
        {
            Vec3::reflect(unit_dir, rec.normal)
        } else {
            Vec3::refract(unit_dir, rec.normal, refraction_ratio)
        };

        Some((Ray::new(rec.point, direction), attenuation))
    }
}

pub struct DiffuseLight {
    pub emit: Color,
}
impl DiffuseLight {
    pub fn new(emit: Color) -> Self { Self { emit} }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray_in: &Ray, _rec: &HitRecord) -> Option<(Ray, Color)> {
        None    // light sources don't scatter
    }

    fn emitted(&self) -> Color {
        self.emit
    }
}