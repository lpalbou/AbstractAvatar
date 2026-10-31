//! Small bounded raster vocabulary. All geometry and paint are trusted built-ins.
//! Colors are premultiplied internally; only the public boundary is straight RGBA.
use crate::{Material, Palette, Sample};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Color {
    pub rgb: [f32; 3],
    pub alpha: f32,
}
impl Color {
    pub fn from_sample(sample: Sample, palette: &Palette) -> Self {
        let rgba = palette[sample.material as usize]
            [(sample.light.clamp(0., 1.) * 255.).round() as usize]
            .to_be_bytes();
        let alpha = sample.coverage.clamp(0., 1.) * rgba[3] as f32 / 255.;
        Self {
            rgb: [
                rgba[0] as f32 / 255. * alpha,
                rgba[1] as f32 / 255. * alpha,
                rgba[2] as f32 / 255. * alpha,
            ],
            alpha,
        }
    }
    pub fn over(self, under: Self) -> Self {
        Self {
            rgb: std::array::from_fn(|i| self.rgb[i] + under.rgb[i] * (1. - self.alpha)),
            alpha: self.alpha + under.alpha * (1. - self.alpha),
        }
    }
    pub fn light(self, under: Self) -> Self {
        let alpha = (self.alpha + under.alpha).min(1.0);
        Self {
            rgb: std::array::from_fn(|i| (self.rgb[i] + under.rgb[i]).min(alpha)),
            alpha,
        }
    }
    pub fn rgba(self) -> [u8; 4] {
        if self.alpha <= 1. / 65535. {
            return [0; 4];
        }
        [
            (self.rgb[0] / self.alpha * 255.).round().clamp(0., 255.) as u8,
            (self.rgb[1] / self.alpha * 255.).round().clamp(0., 255.) as u8,
            (self.rgb[2] / self.alpha * 255.).round().clamp(0., 255.) as u8,
            (self.alpha * 255.).round().clamp(0., 255.) as u8,
        ]
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Shape {
    Ellipse {
        center: (f32, f32),
        radius: (f32, f32),
    },
    Rounded {
        center: (f32, f32),
        half: (f32, f32),
        radius: f32,
    },
    Triangle([(f32, f32); 3]),
}
#[derive(Clone, Debug)]
pub(crate) struct Surface {
    pub shape: Shape,
    pub material: Material,
    pub light: f32,
    pub opacity: f32,
    /// Lit curvature: 0 flat, 1 curved upper-left lighting, 2 soft elliptical glow.
    pub shading: u8,
    pub additive: bool,
    edges: [[f32; 3]; 3],
}
impl Surface {
    pub fn new(
        shape: Shape,
        material: Material,
        light: f32,
        opacity: f32,
        shading: u8,
        additive: bool,
    ) -> Self {
        let mut edges = [[0.; 3]; 3];
        if let Shape::Triangle(p) = shape {
            let sign = ((p[1].0 - p[0].0) * (p[2].1 - p[0].1)
                - (p[1].1 - p[0].1) * (p[2].0 - p[0].0))
                .signum();
            for (i, edge) in edges.iter_mut().enumerate() {
                let (a, b) = (p[i], p[(i + 1) % 3]);
                let inv = sign / (b.0 - a.0).hypot(b.1 - a.1).max(0.00001);
                *edge = [
                    (b.1 - a.1) * inv,
                    (a.0 - b.0) * inv,
                    (a.1 * (b.0 - a.0) - a.0 * (b.1 - a.1)) * inv,
                ];
            }
        }
        Self {
            shape,
            material,
            light,
            opacity,
            shading,
            additive,
            edges,
        }
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        match self.shape {
            Shape::Ellipse {
                center: (x, y),
                radius: (rx, ry),
            }
            | Shape::Rounded {
                center: (x, y),
                half: (rx, ry),
                ..
            } => (x - rx, y - ry, x + rx, y + ry),
            Shape::Triangle(p) => (
                p.iter().map(|p| p.0).fold(f32::INFINITY, f32::min),
                p.iter().map(|p| p.1).fold(f32::INFINITY, f32::min),
                p.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max),
                p.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max),
            ),
        }
    }
    pub fn sample(&self, x: f32, y: f32, pixel: f32) -> Sample {
        let (distance, nx, ny) = match self.shape {
            Shape::Ellipse {
                center: (cx, cy),
                radius: (rx, ry),
            } => {
                let (nx, ny) = ((x - cx) / rx, (y - cy) / ry);
                ((nx.hypot(ny) - 1.) * rx.min(ry), nx, ny)
            }
            Shape::Rounded {
                center: (cx, cy),
                half: (hx, hy),
                radius: r,
            } => {
                let (qx, qy) = ((x - cx).abs() - hx + r, (y - cy).abs() - hy + r);
                (
                    qx.max(0.).hypot(qy.max(0.)) + qx.max(qy).min(0.) - r,
                    (x - cx) / hx,
                    (y - cy) / hy,
                )
            }
            Shape::Triangle(_) => {
                let distance = self
                    .edges
                    .iter()
                    .map(|e| e[0] * x + e[1] * y + e[2])
                    .fold(f32::NEG_INFINITY, f32::max);
                (distance, 0., 0.)
            }
        };
        let mut coverage = if pixel == 0. {
            if distance <= 0. {
                1.
            } else {
                0.
            }
        } else {
            (0.5 - distance / pixel).clamp(0., 1.)
        };
        let mut light = self.light;
        if self.shading == 1 {
            let z = (1. - 0.65 * nx * nx - 0.65 * ny * ny).max(0.).sqrt();
            let diffuse = (-0.48 * nx - 0.4 * ny + 0.68 * z).max(0.);
            let spec = (-((nx + 0.36).powi(2) + (ny + 0.42).powi(2)) / 0.09).exp();
            light *= 0.18 + 0.66 * diffuse + 0.35 * spec;
        } else if self.shading == 2 {
            coverage *= (-(nx * nx + ny * ny) * 3.5).exp();
        }
        Sample {
            material: self.material,
            light: light.clamp(0., 1.),
            coverage: coverage * self.opacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn straight_alpha_is_normalized_once_and_transparent_ink_is_empty() {
        let palette = [[0xff000080; 256], [0x00ff0080; 256], [0x0000ff00; 256]];
        let a = Color::from_sample(
            Sample {
                material: Material::Warm,
                light: 0.5,
                coverage: 1.,
            },
            &palette,
        );
        let b = Color::from_sample(
            Sample {
                material: Material::Eye,
                light: 0.5,
                coverage: 1.,
            },
            &palette,
        );
        assert_eq!(a.rgba(), [255, 0, 0, 128]);
        let over = a.over(b).rgba();
        assert!((over[0] as i32 - 170).abs() <= 1 && (over[1] as i32 - 85).abs() <= 1);
        assert!((over[3] as i32 - 192).abs() <= 1);
        let light = a.light(b).rgba();
        assert_eq!(light, [128, 128, 0, 255]);
        assert_eq!(
            Color::from_sample(
                Sample {
                    material: Material::Dim,
                    light: 1.,
                    coverage: 1.
                },
                &palette
            )
            .rgba(),
            [0; 4]
        );
    }
    #[test]
    fn rgba_accepts_analytic_and_bounded_pixel_footprints_only() {
        let palette = [[0xffffffff; 256]; 3];
        let frame = crate::Frame::new(
            crate::AvatarStyle::WatchHover,
            crate::AvatarPose::Ready,
            0.,
            1.,
            crate::Gaze::default(),
        );
        for invalid in [f32::NAN, f32::INFINITY, -0.001, 0.126] {
            assert_eq!(frame.rgba(0., 0., invalid, &palette), [0; 4]);
        }
        for pixel in [0., 0.001, 0.03125, 0.125] {
            assert!(frame.rgba(0., 0., pixel, &palette)[3] > 200);
        }
        assert_eq!(frame.rgba(f32::NAN, 0., 0., &palette), [0; 4]);
        assert_eq!(frame.rgba(f32::MAX, 0., 0., &palette), [0; 4]);
    }
}
