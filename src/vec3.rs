use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, MulAssign, DivAssign};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self { Self::new(0.0, 0.0, 0.0) }
    pub fn one()  -> Self { Self::new(1.0, 1.0, 1.0) }

    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f64 { self.length_squared().sqrt() }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn normalize(self) -> Self { self / self.length() }

    pub fn near_zero(self) -> bool {
        let eps = 1e-8;
        self.x.abs() < eps && self.y.abs() < eps && self.z.abs() < eps
    }

    pub fn to_rgb_gamma2(self) -> (u8, u8, u8) {
        let r = self.x.sqrt().clamp(0.0, 1.0);
        let g = self.y.sqrt().clamp(0.0, 1.0);
        let b = self.z.sqrt().clamp(0.0, 1.0);
        ((r * 255.999) as u8, (g * 255.999) as u8, (b * 255.999) as u8)
    }

    // ── NEW: random utilities ─────────────────────────────────────────────────

    /// Uniform random vector with components in [min, max).
    pub fn random_range(min: f64, max: f64) -> Self {
        let mut rng = rand::thread_rng();
        Self::new(
            rng.gen_range(min..max),
            rng.gen_range(min..max),
            rng.gen_range(min..max),
        )
    }

    /// Random unit vector via rejection sampling (uniform on sphere surface).
    /// Rejection loop converges in ~1.27 iterations on average.
    pub fn random_unit_vector() -> Self {
        loop {
            let v = Self::random_range(-1.0, 1.0);
            let len_sq = v.length_squared();
            // Reject points outside the unit sphere AND near the origin
            // (near-zero would produce NaN on normalise)
            if 1e-160 < len_sq && len_sq <= 1.0 {
                return v / len_sq.sqrt();
            }
        }
    }

    /// Random unit vector guaranteed to be in the same hemisphere as `normal`.
    pub fn random_on_hemisphere(normal: Vec3) -> Self {
        let v = Self::random_unit_vector();
        if v.dot(normal) > 0.0 { v } else { -v }
    }

    /// Reflect `v` around surface normal `n`.  Both assumed unit length.
    pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - 2.0 * v.dot(n) * n
    }

    /// Refract `uv` through a surface with normal `n`.
    /// `etai_over_etat` = η_incident / η_transmitted (Snell's law ratio).
    pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta  = (-uv).dot(n).min(1.0);
        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_para = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
        r_out_perp + r_out_para
    }
}

pub type Point3 = Vec3;
pub type Color  = Vec3;

// ── operator overloads (unchanged) ───────────────────────────────────────────

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x+rhs.x, self.y+rhs.y, self.z+rhs.z) }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x-rhs.x, self.y-rhs.y, self.z-rhs.z) }
}
impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) }
}
impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, t: f64) -> Self { Self::new(self.x*t, self.y*t, self.z*t) }
}
impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 { v * self }
}
impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { Self::new(self.x*rhs.x, self.y*rhs.y, self.z*rhs.z) }
}
impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, t: f64) -> Self { self * (1.0/t) }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) { self.x+=rhs.x; self.y+=rhs.y; self.z+=rhs.z; }
}
impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, t: f64) { self.x*=t; self.y*=t; self.z*=t; }
}
impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) { *self *= 1.0/t; }
}