use std::sync::Arc;
use rand::Rng;
use rand::RngCore;

use crate::vec3::{Vec3, Point3, Color};

// ── LightSample ───────────────────────────────────────────────────────────────
//
// Everything the integrator needs after sampling a point on a light source.
// Keeping this as a plain struct rather than baking the shadow-ray logic in here;
// the integrator owns the decision of what to do with it.

pub struct LightSample {
    /// Sampled point on the light surface (world space).
    pub point:     Point3,
    /// Outward unit normal at the sampled point.
    pub normal:    Vec3,
    /// Emitted radiance at the sampled point.
    pub emission:  Color,
    /// Solid-angle PDF at the shading point for this sample.
    /// Already converted from area measure: pdf_area * r² / |cosθ_light|.
    /// The integrator divides by this — never touch it after the fact.
    pub pdf:       f64,
    /// Pre-computed unit direction from shading point to light sample.
    /// Stored here so the integrator doesn't recompute it.
    pub direction: Vec3,
    /// Distance to the sampled point — used to clamp the shadow ray's t_max.
    pub distance:  f64,
}

// ── Light trait ───────────────────────────────────────────────────────────────

pub trait Light: Send + Sync {
    /// Uniform area sample on the light's surface.
    /// Returns: (point, outward_normal, emission, pdf_area).
    /// pdf_area = 1 / area for uniform sampling.
    fn sample_surface(&self, rng: &mut dyn RngCore) -> (Point3, Vec3, Color, f64);

    /// Surface area of the light — used by LightList for area-weighted selection.
    fn area(&self) -> f64;
}

// ── LightList ─────────────────────────────────────────────────────────────────
//
// Holds all scene emitters.  Selects among them proportional to surface area so
// that large lights (which contribute more energy) get sampled more often.
// This reduces variance compared to uniform selection at essentially zero cost.
//
// The per-light PDF must be multiplied by the selection probability to form the
// combined PDF.  We fold that into LightSample.pdf so callers never think about
// it separately.

pub struct LightList {
    lights:         Vec<Arc<dyn Light>>,
    /// Prefix-sum of areas — used for O(n) weighted selection.
    /// (n is always tiny in practice; a binary search would be premature.)
    cumulative_area: Vec<f64>,
    total_area:      f64,
}

impl LightList {
    pub fn new() -> Self {
        Self {
            lights:          Vec::new(),
            cumulative_area: Vec::new(),
            total_area:      0.0,
        }
    }

    pub fn add(&mut self, light: Arc<dyn Light>) {
        self.total_area += light.area();
        self.cumulative_area.push(self.total_area);
        self.lights.push(light);
    }

    pub fn is_empty(&self) -> bool { self.lights.is_empty() }

    /// Sample the light list from a shading point `origin`.
    ///
    /// Returns `None` only if the list is empty or the geometry term is
    /// degenerate (zero-length direction, light facing away, etc.).
    /// The caller is responsible for casting the shadow ray.
    pub fn sample(&self, origin: Point3, rng: &mut dyn RngCore) -> Option<LightSample> {
        if self.lights.is_empty() { return None; }

        // ── 1. Area-weighted light selection ──────────────────────────────────
        let dart   = rng.gen::<f64>() * self.total_area;
        let index  = self.cumulative_area
            .iter()
            .position(|&c| dart <= c)
            .unwrap_or(self.lights.len() - 1);

        let light              = &self.lights[index];
        let selection_prob     = light.area() / self.total_area;

        // ── 2. Sample a point on the chosen light ─────────────────────────────
        let (light_point, light_normal, emission, pdf_area) =
            light.sample_surface(rng);

        // ── 3. Geometry term conversion: area PDF → solid-angle PDF ───────────
        //
        //   pdf_solid_angle = pdf_area * r² / |cos θ_light|
        //
        // where θ_light is the angle between the shadow ray direction and the
        // light's outward normal.  If the light is facing away from us the
        // sample is invisible — return None rather than a negative contribution.

        let to_light    = light_point - origin;
        let dist_sq     = to_light.length_squared();
        let distance    = dist_sq.sqrt();

        if distance < 1e-8 { return None; }

        let direction   = to_light / distance;           // unit vector, shading→light
        let cos_light   = light_normal.dot(-direction);  // must be > 0 for visible face

        if cos_light <= 0.0 { return None; }

        // Combined PDF: area PDF / selection_prob, then converted to solid angle.
        let pdf_combined_area    = pdf_area / selection_prob;
        let pdf_solid_angle      = pdf_combined_area * dist_sq / cos_light;

        if pdf_solid_angle <= 0.0 { return None; }

        Some(LightSample {
            point: light_point,
            normal: light_normal,
            emission,
            pdf: pdf_solid_angle,
            direction,
            distance,
        })
    }
}