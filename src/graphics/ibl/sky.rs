use crate::core::error::{ErrorCode, ReactorError, ReactorResult};
use std::path::Path;

pub fn load_hdr_equirect(path: &Path) -> ReactorResult<(Vec<u16>, u32, u32)> {
    let img = image::open(path).map_err(|e| {
        ReactorError::with_source(
            ErrorCode::VulkanImageCreation,
            "no se pudo abrir HDR equirect",
            e,
        )
    })?;
    let rgb32 = img.to_rgb32f();
    let (w, h) = (rgb32.width(), rgb32.height());
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for p in rgb32.pixels() {
        out.push(half::f16::from_f32(p[0]).to_bits());
        out.push(half::f16::from_f32(p[1]).to_bits());
        out.push(half::f16::from_f32(p[2]).to_bits());
        out.push(half::f16::from_f32(1.0).to_bits());
    }
    Ok((out, w, h))
}

const SKY_RAYLEIGH_COEFF: glam::Vec3 = glam::Vec3::new(5.802e-6, 1.3558e-5, 3.31e-5);
const SKY_MIE_COEFF: f32 = 21.0e-6;
const SKY_MIE_G: f32 = 0.78;

fn sky_phase_rayleigh(cos_theta: f32) -> f32 {
    3.0 / (16.0 * std::f32::consts::PI) * (1.0 + cos_theta * cos_theta)
}

fn sky_phase_mie(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    1.0 / (4.0 * std::f32::consts::PI) * (1.0 - g2) / (denom * denom.sqrt())
}

fn sky_transmittance(cos_zenith: f32, turbidity: f32) -> glam::Vec3 {
    let depth = 1.0 / (cos_zenith + 0.05).max(0.001);
    let optical_depth_rayleigh = SKY_RAYLEIGH_COEFF * depth;
    let optical_depth_mie = glam::Vec3::splat(SKY_MIE_COEFF) * depth * turbidity;
    glam::Vec3::new(
        (-optical_depth_rayleigh.x - optical_depth_mie.x).exp(),
        (-optical_depth_rayleigh.y - optical_depth_mie.y).exp(),
        (-optical_depth_rayleigh.z - optical_depth_mie.z).exp(),
    )
}

fn rust_evaluate_atmosphere(view_dir: glam::Vec3, sun_dir: glam::Vec3, turbidity: f32) -> glam::Vec3 {
    let v = view_dir.normalize();
    let s = sun_dir.normalize();
    let cos_theta = v.dot(s);
    let cos_zenith_v = v.y.max(0.0);
    let cos_zenith_s = s.y.max(0.0);
    let t_view = sky_transmittance(cos_zenith_v, turbidity);
    let t_sun = sky_transmittance(cos_zenith_s, turbidity);
    let beta_r = SKY_RAYLEIGH_COEFF * turbidity;
    let beta_m = glam::Vec3::splat(SKY_MIE_COEFF) * turbidity;
    let phase_r = sky_phase_rayleigh(cos_theta);
    let phase_m = sky_phase_mie(cos_theta, SKY_MIE_G);
    let scatter_r = beta_r * phase_r;
    let scatter_m = beta_m * phase_m;
    let beta_sum = beta_r + beta_m;
    let inscatter = glam::Vec3::new(
        (scatter_r.x + scatter_m.x) / beta_sum.x.max(1e-5),
        (scatter_r.y + scatter_m.y) / beta_sum.y.max(1e-5),
        (scatter_r.z + scatter_m.z) / beta_sum.z.max(1e-5),
    );
    let mut sky_color = inscatter * (glam::Vec3::ONE - t_view) * t_sun;
    let sun_intensity = 24.0;
    let sun_angular_diameter_cos = 0.9992;
    if cos_theta > sun_angular_diameter_cos {
        let sun_edge_blend = ((cos_theta - sun_angular_diameter_cos) / 0.0005).clamp(0.0, 1.0);
        sky_color += t_view * sun_intensity * sun_edge_blend;
    }
    let horizon_glow = (1.0 - cos_zenith_v).powi(4);
    sky_color += glam::Vec3::new(0.02, 0.04, 0.08) * t_view * horizon_glow;
    sky_color.max(glam::Vec3::ZERO)
}

pub fn procedural_studio_sky(width: u32, height: u32) -> (Vec<u16>, u32, u32) {
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    let sun_dir = glam::Vec3::new(-0.45, 0.85, 0.40).normalize();
    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;
        let theta = v * std::f32::consts::PI;
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;
            let phi = u * std::f32::consts::TAU - std::f32::consts::PI;
            let dir = glam::Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ).normalize();
            let col = if dir.y < 0.0 {
                let t = (-dir.y * 2.0).clamp(0.0, 1.0);
                let ground = glam::Vec3::new(0.04, 0.04, 0.05);
                let horizon_sky = rust_evaluate_atmosphere(glam::Vec3::new(dir.x, 0.0, dir.z), sun_dir, 2.0);
                horizon_sky.lerp(ground, t)
            } else {
                rust_evaluate_atmosphere(dir, sun_dir, 2.0)
            };
            out.push(half::f16::from_f32(col.x).to_bits());
            out.push(half::f16::from_f32(col.y).to_bits());
            out.push(half::f16::from_f32(col.z).to_bits());
            out.push(half::f16::from_f32(1.0).to_bits());
        }
    }
    (out, width, height)
}
