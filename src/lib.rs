//! Shared procedural geometry, style identities and pose vocabulary.
//! No UI toolkit, timers, network, audio, inference or preference ownership.
#![deny(unsafe_op_in_unsafe_fn)]
use std::time::Duration;
/// Observed activity, supplied by the application rather than inferred here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AvatarPose {
    #[default]
    Ready,
    Thinking,
    Working,
    Listening,
    Speaking,
    Waiting,
    Error,
}
impl AvatarPose {
    pub fn id(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Thinking => "thinking",
            Self::Working => "working",
            Self::Listening => "listening",
            Self::Speaking => "speaking",
            Self::Waiting => "waiting",
            Self::Error => "error",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Thinking => "Thinking",
            Self::Working => "Working",
            Self::Listening => "Listening",
            Self::Speaking => "Speaking",
            Self::Waiting => "Waiting",
            Self::Error => "Error",
        }
    }
    pub fn period(self) -> Option<Duration> {
        match self {
            Self::Ready => Some(Duration::from_millis(500)),
            Self::Waiting => Some(Duration::from_millis(167)),
            Self::Error => None,
            _ => Some(Duration::from_millis(84)),
        }
    }
}
mod catalog;
pub use catalog::{AvatarStyle, RetiredStyle, RETIRED};
#[cfg(feature = "json")]
pub mod descriptor;
mod raster;
mod scenes;
/// Host-owned material shade ramps, packed as `0xRRGGBBAA`.
/// Rows correspond to [`Material::Warm`], [`Material::Eye`] and [`Material::Dim`].
pub type Palette = [[u32; 256]; 3];
impl AvatarPose {
    pub const ALL: [Self; 7] = [
        Self::Ready,
        Self::Thinking,
        Self::Working,
        Self::Listening,
        Self::Speaking,
        Self::Waiting,
        Self::Error,
    ];
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL.get(code as usize).copied()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    Warm,
    Eye,
    Dim,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub material: Material,
    pub light: f32,
    pub coverage: f32,
}
impl Sample {
    pub const EMPTY: Self = Self {
        material: Material::Dim,
        light: 0.,
        coverage: 0.,
    };
}
/// Largest eye shift a full gaze produces, in centered frame units (the frame spans -1..1):
/// 2.5% of the frame width horizontally and 1.25% vertically. Gaze never moves anything else.
pub const GAZE_REACH: [f32; 2] = [0.05, 0.025];
/// Normalized look direction (-1..1 each axis). Only `gaze: "eyes"` designs in Ready answer it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gaze {
    pub x: f32,
    pub y: f32,
}
impl Gaze {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: if x.is_finite() {
                x.clamp(-1.0, 1.0)
            } else {
                0.0
            },
            y: if y.is_finite() {
                y.clamp(-1.0, 1.0)
            } else {
                0.0
            },
        }
    }
    /// The bounded eye shift for this gaze, in centered frame units.
    pub(crate) fn offset(self) -> (f32, f32) {
        (self.x * GAZE_REACH[0], self.y * GAZE_REACH[1])
    }
}
#[derive(Clone, Debug)]
pub struct Frame {
    style: AvatarStyle,
    pose: AvatarPose,
    level: f32,
    time: f32,
    gaze: Gaze,
    scene: Option<std::sync::Arc<scenes::Scene>>,
}
impl Frame {
    /// Sample a composited foreground in straight RGBA8, preserving colored light layers.
    ///
    /// `pixel_size` is a pixel's full width in centered coordinates (normally
    /// `2.0 / min(width, height)`). Zero requests an analytic point sample. Values
    /// outside `0..=0.125`, and non-finite coordinates, return transparent black.
    /// This function allocates nothing. Composite its result over the host's background.
    pub fn rgba(&self, x: f32, y: f32, pixel_size: f32, palette: &Palette) -> [u8; 4] {
        if !x.is_finite()
            || !y.is_finite()
            || !pixel_size.is_finite()
            || x.abs() > 1.5
            || y.abs() > 1.5
            || !(0.0..=0.125).contains(&pixel_size)
        {
            return [0; 4];
        }
        if let Some(scene) = &self.scene {
            scene.rgba(x, y, pixel_size, palette)
        } else {
            raster::Color::from_sample(self.sample(x, y), palette).rgba()
        }
    }
    /// A frame driven by one audio level. Band-aware designs derive a plausible spread of
    /// motion from the level and time (see [`Self::with_bands`]).
    pub fn new(style: AvatarStyle, pose: AvatarPose, level: f32, time: f32, gaze: Gaze) -> Self {
        Self::build(style, pose, level, time, gaze, None)
    }
    /// A frame driven by eight measured audio bands (low to high frequency, each 0..1) as
    /// well as the level. Designs with per-band parts, such as `watch-rings`, move each part
    /// with its own band.
    pub fn with_bands(
        style: AvatarStyle,
        pose: AvatarPose,
        level: f32,
        time: f32,
        gaze: Gaze,
        bands: [f32; 8],
    ) -> Self {
        Self::build(style, pose, level, time, gaze, Some(bands))
    }
    pub(crate) fn build(
        style: AvatarStyle,
        pose: AvatarPose,
        level: f32,
        time: f32,
        gaze: Gaze,
        bands: Option<[f32; 8]>,
    ) -> Self {
        let level = if level.is_finite() {
            level.clamp(0., 1.)
        } else {
            0.
        };
        let time = if pose == AvatarPose::Error {
            0.
        } else if time.is_finite() {
            time.rem_euclid(86400.)
        } else {
            0.
        };
        // Gaze only shifts the eyes of designs that have them, and only while Ready.
        let gaze = if pose == AvatarPose::Ready && style.follows_gaze() {
            Gaze::new(gaze.x, gaze.y)
        } else {
            Gaze::default()
        };
        let bands = bands.map(|b| b.map(|x| if x.is_finite() { x.clamp(0., 1.) } else { 0. }));
        let scene = if !style.analytic() {
            Some(std::sync::Arc::new(scenes::build(
                style,
                pose,
                level,
                time,
                bands,
                gaze.offset(),
            )))
        } else {
            None
        };
        Self {
            style,
            pose,
            level,
            time,
            gaze,
            scene,
        }
    }
    pub fn sample(&self, x: f32, y: f32) -> Sample {
        if !x.is_finite() || !y.is_finite() {
            return Sample::EMPTY;
        }
        if let Some(scene) = &self.scene {
            return scene.sample(x, y);
        }
        sample_inner(
            self.style,
            self.pose,
            self.level,
            self.time,
            x,
            y,
            &Materials {
                warm: Material::Warm,
                eye: Material::Eye,
                dim: Material::Dim,
                gaze: self.gaze,
            },
        )
    }
}
struct Materials {
    gaze: Gaze,
    warm: Material,
    eye: Material,
    dim: Material,
}
impl Materials {
    fn ground(&self) -> Sample {
        Sample::EMPTY
    }
    fn state(&self, p: AvatarPose) -> &Material {
        if p == AvatarPose::Error {
            &self.eye
        } else {
            &self.warm
        }
    }
    fn paint(&self, m: &Material, light: f32, coverage: f32) -> Sample {
        Sample {
            material: *m,
            light: light.clamp(0., 1.),
            coverage: coverage.clamp(0., 1.),
        }
    }
}
pub fn sample(
    style: AvatarStyle,
    pose: AvatarPose,
    level: f32,
    time: f32,
    x: f32,
    y: f32,
) -> Sample {
    Frame::new(style, pose, level, time, Gaze::default()).sample(x, y)
}

#[cfg(feature = "ffi")]
mod ffi;
fn soft(distance: f32, width: f32) -> f32 {
    (0.5 - distance / width.max(0.001)).clamp(0.0, 1.0)
}
fn glow(distance: f32, width: f32) -> f32 {
    (-(distance / width).powi(2)).exp()
}
fn segment(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let u = ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy);
    let u = u.clamp(0.0, 1.0);
    ((x - ax - u * dx).powi(2) + (y - ay - u * dy).powi(2)).sqrt()
}
fn sample_inner(
    style: AvatarStyle,
    pose: AvatarPose,
    level: f32,
    time: f32,
    x: f32,
    y: f32,
    m: &Materials,
) -> Sample {
    match style {
        AvatarStyle::Halo => halo(pose, level, time, x, y, m),
        AvatarStyle::Hover => hover(pose, level, time, x, y, m),
        AvatarStyle::Vortex => vortex(pose, level, time, x, y, m),
        AvatarStyle::Eclipse => eclipse(pose, level, time, x, y, m),
        AvatarStyle::Rover => rover(pose, level, time, x, y, m),
        _ => unreachable!("scene styles are dispatched before analytic sampling"),
    }
}
fn halo(p: AvatarPose, level: f32, time: f32, x: f32, y: f32, m: &Materials) -> Sample {
    let r = (x * x + y * y).sqrt();
    let a = y.atan2(x);
    let voice = if matches!(p, AvatarPose::Speaking | AvatarPose::Listening) {
        level
    } else {
        0.0
    };
    let breath = if p == AvatarPose::Ready {
        0.014 * (time * 0.9).sin()
    } else {
        0.0
    };
    let expression = if p == AvatarPose::Speaking {
        0.018 * speech(time)
    } else if p == AvatarPose::Listening {
        -0.010 * (0.5 + 0.5 * (time * 1.2).sin())
    } else {
        0.0
    };
    let radius = 0.49 + voice * 0.035 + breath + expression;
    let rim = glow(r - radius, 0.075);
    if r < radius + 0.015 {
        let nx = x / radius;
        let ny = y / radius;
        let z = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
        let diffuse = (-0.45 * nx - 0.5 * ny + 0.72 * z).max(0.0);
        let scan = if p == AvatarPose::Thinking {
            0.12 * (time * 0.8).sin()
        } else {
            0.0
        };
        let highlight = glow(
            ((nx + 0.30 - scan).powi(2) + (ny + 0.35).powi(2)).sqrt(),
            0.22,
        );
        let veins = (a * 5.0 + r * 22.0 - time * if p == AvatarPose::Thinking { 1.7 } else { 0.6 })
            .sin()
            .abs()
            .powi(9)
            * if p == AvatarPose::Thinking {
                0.08
            } else {
                0.035
            };
        let heart = glow(r, radius * 0.58) * 0.18;
        return m.paint(
            &m.warm,
            (0.18 + 0.65 * diffuse + 0.24 * highlight + veins + heart).clamp(0.0, 1.0),
            soft(r - radius, 0.04),
        );
    }
    // The watch Halo's seven broad streamers remain soft, not isolated dots.
    let turn = if p == AvatarPose::Thinking {
        time * 0.25
    } else if p == AvatarPose::Listening {
        -time * 0.24
    } else if p == AvatarPose::Speaking {
        time * 0.45
    } else {
        time * 0.07
    };
    let streamers = (0.5
        + 0.5 * (0.62 * (5.0 * (a - turn)).cos() + 0.38 * (8.0 * (a - turn) + 1.7).cos()))
    .max(0.0);
    let reach = 0.12 + voice * 0.18 + if p == AvatarPose::Thinking { 0.08 } else { 0.0 };
    let crown = glow(r - radius, 0.08 + reach * streamers) * 0.42;
    // Intake travels toward the core; spoken syllables send a wider wave out.
    // The distinction remains visible with a constant or quiet audio level.
    let flow = if matches!(p, AvatarPose::Listening | AvatarPose::Speaking) {
        let progress = (time
            * if p == AvatarPose::Listening {
                0.55
            } else {
                0.90
            })
        .fract();
        let distance = if p == AvatarPose::Listening {
            0.28 * (1.0 - progress)
        } else {
            0.32 * progress
        };
        glow(r - radius - distance, 0.028) * (std::f32::consts::PI * progress).sin().powi(2) * 0.28
    } else {
        0.0
    };
    let rotation = if p == AvatarPose::Working {
        time * 0.65
    } else {
        time * 0.35
    };
    let (c, s) = (rotation.cos(), rotation.sin());
    let (rx, ry) = (x * c - y * s, x * s + y * c);
    let orbit = ((rx / 0.79).powi(2) + (ry / 0.31).powi(2)).sqrt();
    let arc = if matches!(p, AvatarPose::Thinking | AvatarPose::Working) {
        glow(orbit - 1.0, 0.07) * (0.32 + 0.16 * (a - time).cos())
    } else {
        glow(r - 0.76, 0.065) * 0.12
    };
    let outer = if p == AvatarPose::Waiting {
        let pulse = 0.5 + 0.5 * (time * 2.4).sin();
        glow(r - (0.83 + 0.018 * (time * 2.4).sin()), 0.045) * (0.30 + 0.25 * pulse)
            + glow(r - 0.83, 0.035) * (0.5 + 0.5 * (a - time * 1.1).cos()).powi(6) * 0.30
    } else {
        0.0
    };
    let intensity = (rim * 0.38 + crown + arc + outer + flow).clamp(0.0, 0.88);
    if intensity < 0.012 {
        m.ground()
    } else {
        m.paint(m.state(p), intensity, 1.0)
    }
}
fn blink(time: f32, period: f32, duration: f32) -> f32 {
    let phase = time.rem_euclid(period);
    if phase < duration {
        1.0 - 0.88 * (std::f32::consts::PI * phase / duration).sin().powi(2)
    } else {
        1.0
    }
}
fn speech(time: f32) -> f32 {
    0.6 * (0.5 + 0.5 * (time * 8.0).sin()) + 0.4 * (0.5 + 0.5 * (time * 13.0 + 1.0).sin())
}
fn hover(p: AvatarPose, level: f32, time: f32, x: f32, y: f32, m: &Materials) -> Sample {
    let bob = if p == AvatarPose::Working {
        0.10 * (time * 1.6).sin() + 0.012 * (time * 3.1).sin()
    } else if p == AvatarPose::Ready {
        0.018 * (time * 0.85).sin()
    } else if p == AvatarPose::Thinking {
        -0.025 + 0.026 * (time * 0.9).sin()
    } else if matches!(p, AvatarPose::Listening | AvatarPose::Speaking) {
        0.025 * (time * 1.05).sin()
    } else {
        0.0
    };
    let mut x = x;
    let mut y = y - bob;
    if p == AvatarPose::Listening {
        // Cock toward the user's conversation, with a raised listening hand.
        // This is a real head/body roll, not an almost invisible shear.
        x += 0.075;
        let (s, c) = (-0.24 + 0.055 * (time * 1.1).sin()).sin_cos();
        (x, y) = (c * x + s * y, -s * x + c * y);
    } else if p == AvatarPose::Thinking {
        y += (0.065 + 0.045 * (time * 0.8).sin()) * x;
    } else if p == AvatarPose::Waiting {
        y -= 0.065;
    }
    let dome = ((x / 0.68).powi(2) + ((y + 0.11) / 0.50).powi(2)).sqrt();
    let dome_edge = (dome - 1.0) * 0.5;
    if y < -0.11 && dome_edge < 0.025 {
        let nx = x / 0.68;
        let ny = (y + 0.11) / 0.50;
        let z = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
        let light = (-0.45 * nx - 0.40 * ny + 0.73 * z).max(0.0);
        let reflection = glow(((x + 0.24).powi(2) + (y + 0.42).powi(2)).sqrt(), 0.115);
        let rim = glow(dome - 0.98, 0.04) * 0.10;
        return m.paint(
            &m.warm,
            (0.52 + light * 0.36 + reflection * 0.23 + rim).clamp(0.0, 1.0),
            soft(dome_edge, 0.05),
        );
    }
    // Curved polished rim along the lower dome, following the watch barrel.
    let top_rim = -0.10 + 0.055 * (x / 0.68).powi(2);
    if x.abs() < 0.69 && (y - top_rim).abs() < 0.022 {
        return m.paint(&m.warm, 0.90, soft(x.abs() - 0.68, 0.04));
    }
    if (-0.10..0.20).contains(&y) && x.abs() < 0.68 {
        let eye_y = m.gaze.offset().1
            + if p == AvatarPose::Waiting {
                0.07
            } else {
                0.035
            };
        let eye_x = 0.27
            + if p == AvatarPose::Thinking {
                0.025
            } else {
                0.0
            };
        let open = if p == AvatarPose::Thinking {
            0.78
        } else if p == AvatarPose::Error {
            0.40
        } else if p == AvatarPose::Waiting {
            0.55
        } else {
            1.0
        };
        let blink = if p == AvatarPose::Ready {
            blink(time, 5.0, 0.8)
        } else if p == AvatarPose::Thinking {
            blink(time + 1.0, 4.7, 0.35)
        } else if p == AvatarPose::Listening {
            blink(time + 2.0, 6.3, 0.3)
        } else {
            1.0
        };
        let open = open * blink;
        let gaze = m.gaze.offset().0
            + if p == AvatarPose::Waiting {
                0.045
            } else if p == AvatarPose::Thinking {
                0.075 * (time * 1.1).sin()
            } else {
                0.0
            };
        if p == AvatarPose::Error {
            let xx = (x - gaze).abs() - eye_x;
            let yy = y - eye_y;
            let cross = glow(
                segment(xx, yy, -0.070, -0.070, 0.070, 0.070)
                    .min(segment(xx, yy, -0.070, 0.070, 0.070, -0.070)),
                0.018,
            );
            return if cross > 0.045 {
                m.paint(&m.eye, 0.65 + 0.09 * cross, 1.0)
            } else {
                m.paint(&m.dim, 0.12, 1.0)
            };
        }
        let d = (((x - gaze).abs() - eye_x).powi(2) + ((y - eye_y) / open).powi(2)).sqrt();
        let radius = 0.088
            + if p == AvatarPose::Speaking {
                level * 0.018 + 0.008 * speech(time)
            } else if p == AvatarPose::Listening {
                level * 0.012
            } else {
                0.0
            };
        let eye = if p == AvatarPose::Waiting {
            &m.warm
        } else {
            &m.eye
        };
        if d < radius {
            let core = 1.0 - d / radius;
            let brightness = if p == AvatarPose::Waiting {
                0.42 + 0.22 * core.powf(0.35) + 0.15 * (0.5 + 0.5 * (time * 2.4).sin())
            } else {
                0.66 + 0.34 * core.powf(0.35)
            };
            return m.paint(eye, brightness, 1.0);
        }
        let shine = glow(d - radius, 0.055) * 0.54;
        return if shine > 0.03 {
            m.paint(eye, shine, 1.0)
        } else {
            m.paint(&m.dim, 0.12 + 0.08 * (1.0 - x.abs()), 1.0)
        };
    }
    let hull = ((x / 0.69).powi(2) + ((y - 0.24) / 0.34).powi(2)).sqrt();
    if (0.20..0.60).contains(&y) && hull < 1.045 {
        let nx = x / 0.69;
        let z = (1.0 - nx * nx).max(0.0).sqrt();
        let lit = (0.62 + 0.27 * (-0.4 * nx + 0.7 * z) - 0.12 * (y - 0.2)).clamp(0.0, 1.0);
        let band1 = 0.245 + 0.055 * (x / 0.69).powi(2);
        let band2 = 0.455 + 0.045 * (x / 0.69).powi(2);
        let band = glow(y - band1, 0.012).max(glow(y - band2, 0.016));
        if p == AvatarPose::Speaking
            && x.abs() < 0.23
            && (y - 0.35).abs() < 0.025 + (0.035 + level * 0.045) * speech(time)
        {
            return if (x * 55.0).sin() > 0.1 {
                m.paint(&m.eye, 0.38 + 0.35 * level + 0.17 * speech(time), 1.0)
            } else {
                m.paint(&m.dim, 0.10, 1.0)
            };
        }
        return m.paint(
            &m.warm,
            (lit + band * 0.18).clamp(0.0, 1.0),
            soft((hull - 1.0) * 0.35, 0.05),
        );
    }
    // Rounded arms, extending a little while tools are working.
    let reach = if p == AvatarPose::Working { 0.12 } else { 0.0 };
    let arm = if p == AvatarPose::Listening && x < 0.0 {
        segment(x, y, -0.72, 0.06, -0.82 - 0.025 * (time * 1.4).sin(), -0.20) - 0.065
    } else if p == AvatarPose::Waiting {
        segment(x.abs(), y, 0.72, 0.06, 0.69, 0.42) - 0.055
    } else {
        segment(x.abs(), y, 0.72, 0.06, 0.75 + reach, 0.40 - reach) - 0.055
    };
    if arm < 0.025 {
        return m.paint(
            &m.warm,
            (0.75 - x.abs() * 0.1).clamp(0.0, 1.0),
            soft(arm, 0.05),
        );
    }
    if p == AvatarPose::Waiting {
        let hot = (time * 1.5).floor() as usize % 3;
        let dots = [-0.15f32, 0.0, 0.15]
            .iter()
            .enumerate()
            .map(|(i, dx)| {
                glow(x - dx, 0.035) * glow(y - 0.76, 0.035) * if i == hot { 0.72 } else { 0.25 }
            })
            .fold(0.0f32, f32::max);
        if dots > 0.02 {
            return m.paint(&m.warm, dots, 1.0);
        }
    }
    let plume_len = 0.14
        + if p == AvatarPose::Working {
            0.14 + 0.025 * (time * 4.0).sin()
        } else {
            0.0
        };
    let plume = glow(x.abs() - 0.32, 0.045) * glow(y - 0.59, plume_len);
    let pool = glow(x, 0.48) * glow(y - 0.91, 0.04) * 0.16;
    let mut particles = 0.0f32;
    if p == AvatarPose::Working {
        for side in [-1.0f32, 1.0] {
            for i in 0..3 {
                let age = (time * 0.7 + i as f32 * 0.31).fract();
                let px = side * 0.32 + side * 0.12 * age;
                let py = 0.65 + 0.34 * age;
                particles += glow(x - px, 0.025) * glow(y - py, 0.035) * (1.0 - age) * 0.65;
            }
        }
    }
    let intensity = (plume * 0.65 + pool + particles).min(0.85);
    if intensity < 0.015 {
        m.ground()
    } else {
        m.paint(&m.eye, intensity, 1.0)
    }
}

// Watch references: Orb.swift vortex arms, OrbConcepts Eclipse, OrbRobots Rover. Adapted to a bounded analytic field for terminal and Canvas.
fn vortex(p: AvatarPose, level: f32, t: f32, x: f32, y: f32, m: &Materials) -> Sample {
    let r = x.hypot(y);
    let a = y.atan2(x);
    let speed = match p {
        AvatarPose::Thinking => 1.3,
        AvatarPose::Working => 2.0,
        AvatarPose::Listening => -0.6,
        _ => 0.25,
    };
    let reach = if p == AvatarPose::Speaking {
        0.14 * speech(t) + 0.12 * level
    } else if p == AvatarPose::Listening {
        -0.12 * level
    } else {
        0.
    };
    let radius = 0.77 + reach + 0.02 * (t * 1.8).sin();
    if r > radius + 0.12 {
        return m.ground();
    }
    let arms = if p == AvatarPose::Working { 3.0 } else { 2.0 };
    let spiral = (arms * (a - speed * t - 3.0 * (r + 0.15).ln()))
        .cos()
        .max(0.)
        .powf(9.);
    let envelope = soft(r - radius, 0.09) * (0.12 + 0.88 * glow(r - 0.39, 0.42));
    let core = glow(r, 0.19) * (0.6 + 0.15 * (t * 2.).sin());
    let wait = if p == AvatarPose::Waiting {
        0.50 + 0.28 * (t * 3.).sin()
    } else {
        1.
    };
    let light = (spiral * envelope + core).clamp(0., 1.) * wait;
    if light < 0.035 {
        m.ground()
    } else {
        m.paint(m.state(p), light, soft(r - radius, 0.07))
    }
}
fn eclipse(p: AvatarPose, level: f32, t: f32, x: f32, y: f32, m: &Materials) -> Sample {
    let r = x.hypot(y);
    let a = y.atan2(x);
    let pulse = if p == AvatarPose::Speaking {
        0.07 * speech(t) + 0.08 * level
    } else {
        0.02 * (t * 2.).sin()
    };
    let radius = 0.51 + pulse;
    if r < radius - 0.035 {
        return m.paint(&m.dim, 0.08 + 0.06 * (x + 1.), 1.);
    }
    let rim = glow(r - radius, 0.025);
    let rays = (a * 9. + (t * 0.35)).sin().abs().powf(12.);
    let work = if p == AvatarPose::Working {
        (a * 5. - t * 3.).sin().max(0.).powf(3.)
    } else {
        rays
    };
    let corona = glow(r - radius, 0.16 + 0.13 * work + level * 0.05);
    // Thought is a slow orbit around the disc; work turns the rays and sends flares out.
    let orbit = if p == AvatarPose::Thinking {
        glow((x * x + (y * 1.8).powi(2)).sqrt() - 0.78, 0.018) * (0.5 + 0.5 * (a - t * 0.9).cos())
    } else {
        0.
    };
    let flare = if p == AvatarPose::Working {
        let u = (t * 0.9).fract();
        glow(r - (radius + 0.06 + 0.4 * u), 0.03) * (1. - u) * 0.5 * (0.4 + 0.6 * work)
    } else {
        0.
    };
    let flow = if matches!(p, AvatarPose::Listening | AvatarPose::Speaking) {
        let u = (t * 0.55).fract();
        let rr = if p == AvatarPose::Listening {
            0.99 - u * 0.4
        } else {
            0.58 + u * 0.4
        };
        glow(r - rr, 0.022) * 0.45
    } else {
        0.
    };
    let light = (0.85 * rim + 0.48 * corona + orbit + flare + flow).min(1.)
        * if p == AvatarPose::Waiting {
            0.5 + 0.23 * (t * 3.).sin()
        } else {
            1.
        };
    if light < 0.045 {
        m.ground()
    } else {
        m.paint(m.state(p), light, 1.)
    }
}
fn rover(p: AvatarPose, level: f32, t: f32, x: f32, y: f32, m: &Materials) -> Sample {
    let lean = if p == AvatarPose::Listening {
        0.22 + 0.035 * (t * 1.7).sin()
    } else if p == AvatarPose::Thinking {
        0.08 * (t * 1.3).sin()
    } else {
        0.
    };
    let bob = if p == AvatarPose::Working {
        0.055 * (t * 3.).sin()
    } else {
        0.012 * (t * 1.7).sin()
    };
    let (sx, cx) = lean.sin_cos();
    let (hx, hy) = (x * cx + y * sx, -x * sx + y * cx + bob);
    let head_y = if p == AvatarPose::Waiting {
        -0.30
    } else {
        -0.43
    };
    for side in [-1.0, 1.0] {
        let dx = hx - side * 0.31;
        let dy = hy - head_y;
        let shell = (dx / 0.29).powi(2) + (dy / 0.27).powi(2);
        if shell < 1.12 {
            if p == AvatarPose::Error && dx.hypot(dy) < 0.20 {
                let cross = glow(
                    segment(dx, dy, -0.105, -0.105, 0.105, 0.105)
                        .min(segment(dx, dy, -0.105, 0.105, 0.105, -0.105)),
                    0.025,
                );
                return if cross > 0.04 {
                    m.paint(&m.eye, 0.65 + 0.09 * cross, 1.0)
                } else {
                    m.paint(&m.dim, 0.08, 1.0)
                };
            }
            // Thinking: half-lidded eyes looking up; the lid stays put while pupils drift.
            let lid = match p {
                AvatarPose::Waiting => 0.5,
                AvatarPose::Thinking => 0.62,
                _ => blink(t, 5.1, 0.16).max(0.15),
            };
            let iris = (dx / 0.19).powi(2) + (dy / (0.19 * lid)).powi(2);
            if iris < 1. {
                let gaze = m.gaze.offset().0
                    + if p == AvatarPose::Thinking {
                        0.05 * (t * 2.).sin()
                    } else {
                        0.
                    };
                let up = if p == AvatarPose::Thinking { -0.05 } else { 0. };
                if (dx - gaze).hypot(dy - m.gaze.offset().1 - up) < 0.075 {
                    return m.paint(&m.dim, 0.1, 1.);
                }
                return m.paint(
                    &m.eye,
                    0.52 + 0.28 * glow(iris, 0.5)
                        + level * 0.17
                        + if p == AvatarPose::Speaking {
                            speech(t) * 0.15
                        } else {
                            0.
                        },
                    1.,
                );
            }
            return m.paint(
                &m.warm,
                (0.24 + 0.53 * (1.0 - shell.min(1.0)).sqrt() - 0.18 * dx - 0.15 * dy
                    + 0.18 * glow((dx + 0.09).hypot(dy + 0.09), 0.07))
                .clamp(0., 1.),
                soft(shell - 1., 0.12),
            );
        }
    }
    if x.abs() < 0.09 && (-0.24..0.12).contains(&y) {
        return m.paint(&m.dim, 0.72, 1.);
    }
    let body = (x / 0.54).abs().max(((y - 0.29) / 0.22).abs());
    if body < 1.05 {
        if (y - 0.27).abs() < 0.045 && x.abs() < 0.32 {
            return m.paint(
                m.state(p),
                0.3 + 0.5
                    * match p {
                        AvatarPose::Speaking => speech(t),
                        // Work runs lights from the middle of the bar to both ends.
                        AvatarPose::Working => {
                            let u = (x.abs() / 0.32 - t * 1.6).rem_euclid(0.5) * 2.;
                            0.2 + 0.8 * (1. - u).powi(3)
                        }
                        // Thought dims the bar to a slow glow.
                        AvatarPose::Thinking => 0.25 + 0.1 * (t * 1.1).sin(),
                        _ => 0.45 + 0.2 * (t * 2.).sin(),
                    },
                1.,
            );
        }
        return m.paint(
            &m.warm,
            (0.63 - 0.2 * x - 0.16 * y
                + 0.15 * glow(body - 0.92, 0.075)
                + 0.1 * glow((x + 0.24).hypot(y - 0.16), 0.18))
            .clamp(0., 1.),
            soft(body - 1., 0.08),
        );
    }
    if (0.40..0.70).contains(&x.abs()) && (0.25..0.69).contains(&y) {
        let roll = if p == AvatarPose::Working { t * 5. } else { 0. };
        let tread = (y * 38. + roll).sin().abs();
        return m.paint(&m.dim, 0.30 + 0.4 * tread, 1.);
    }
    m.ground()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn styles_are_distinct_and_every_pose_is_bounded() {
        let mut hashes = std::collections::HashSet::new();
        for style in AvatarStyle::ALL {
            let mut hash = 0u64;
            for p in AvatarPose::ALL {
                let frame = Frame::new(style, p, 0.6, 1.1, Gaze::default());
                for y in 0..64 {
                    for x in 0..64 {
                        let s = frame.sample((x as f32 - 32.) / 32., (y as f32 - 32.) / 32.);
                        assert!(
                            s.light.is_finite()
                                && s.coverage.is_finite()
                                && (0.0..=1.0).contains(&s.coverage)
                        );
                        // The original Watch planets intentionally share their resting body;
                        // compare the complete activity repertoire rather than only Ready.
                        hash = hash.wrapping_mul(31).wrapping_add(
                            (s.light * 255.) as u64 + ((s.coverage * 255.) as u64) * 256,
                        );
                    }
                }
            }
            assert!(hashes.insert(hash), "duplicate style {style:?}");
        }
    }
    #[test]
    fn invalid_numeric_inputs_remain_bounded() {
        for style in AvatarStyle::ALL {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -2.0, 2.0] {
                let result = sample(style, AvatarPose::Speaking, value, value, 0.2, 0.1);
                assert!(result.light.is_finite() && result.coverage.is_finite());
                assert_eq!(
                    sample(style, AvatarPose::Ready, 0.0, 0.0, value, f32::NAN),
                    Sample::EMPTY
                );
            }
        }
    }
    #[test]
    fn every_voice_pose_responds_to_measured_level() {
        for style in AvatarStyle::ALL {
            for pose in [AvatarPose::Listening, AvatarPose::Speaking] {
                let quiet = Frame::new(style, pose, 0.05, 1.25, Gaze::default());
                let loud = Frame::new(style, pose, 0.9, 1.25, Gaze::default());
                assert!(
                    (0..64).any(|y| (0..64).any(|x| {
                        let (x, y) = ((x as f32 - 32.0) / 32.0, (y as f32 - 32.0) / 32.0);
                        quiet.sample(x, y) != loud.sample(x, y)
                    })),
                    "{style:?}/{pose:?} ignores voice level"
                );
            }
        }
    }
    #[test]
    fn robot_errors_are_red_crosses_not_filled_eyes() {
        for (style, x, y, diagonal, scale) in [
            (AvatarStyle::Hover, 0.27, 0.035, 0.05, 1.0),
            (AvatarStyle::Rover, 0.31, -0.43, 0.07, 1.0),
        ] {
            let center = sample(style, AvatarPose::Error, 0.0, 0.0, x, y);
            assert_eq!(center.material, Material::Eye);
            assert_eq!(
                sample(
                    style,
                    AvatarPose::Error,
                    0.0,
                    0.0,
                    x + diagonal,
                    y + diagonal * scale
                )
                .material,
                Material::Eye
            );
            assert_eq!(
                sample(style, AvatarPose::Error, 0.0, 0.0, x + diagonal, y).material,
                Material::Dim
            );
        }
    }
    #[test]
    fn gaze_moves_only_the_eyes_of_designs_that_have_them() {
        // Distinct ramps per material so moving an eye over its socket shows.
        let mut palette = [[0u32; 256]; 3];
        for (shade, i) in (0..256u32).zip(0..) {
            palette[0][i] = shade << 24 | 0x2040ff;
            palette[1][i] = 0x14 << 24 | shade << 16 | 0x50d2;
            palette[2][i] = 0x3228 << 16 | shade << 8 | 0xb4;
        }
        let draw = |style, pose, gaze| {
            let frame = Frame::new(style, pose, 0.6, 1.1, gaze);
            (0..64 * 64)
                .map(|i| {
                    let (x, y) = ((i % 64) as f32, (i / 64) as f32);
                    frame.rgba((x - 31.5) / 32., (y - 31.5) / 32., 1. / 32., &palette)
                })
                .collect::<Vec<_>>()
        };
        // Outer silhouette bounds: a moving body shifts them; moving eyes stay inside.
        let bounds = |pixels: &[[u8; 4]]| {
            let lit: Vec<_> = (0..pixels.len()).filter(|i| pixels[*i][3] > 8).collect();
            let (xs, ys) = (lit.iter().map(|i| i % 64), lit.iter().map(|i| i / 64));
            (xs.clone().min(), xs.max(), ys.clone().min(), ys.max())
        };
        for style in AvatarStyle::ALL {
            assert!(matches!(style.gaze(), "eyes" | "none"));
            let rest = draw(style, AvatarPose::Ready, Gaze::default());
            let covered = rest.iter().filter(|p| p[3] > 8).count();
            for gaze in [Gaze::new(-1.0, 0.6), Gaze::new(1.0, -1.0)] {
                let looked = draw(style, AvatarPose::Ready, gaze);
                let changed = rest.iter().zip(&looked).filter(|(a, b)| a != b).count();
                if style.gaze() == "none" {
                    assert_eq!(changed, 0, "{style:?} must ignore gaze");
                    continue;
                }
                assert!(changed > 0, "{style:?} eyes ignore gaze");
                assert!(changed * 2 < covered, "{style:?} moves more than its eyes");
                assert_eq!(bounds(&rest), bounds(&looked), "{style:?} body moved");
            }
            for pose in AvatarPose::ALL
                .into_iter()
                .filter(|p| *p != AvatarPose::Ready)
            {
                assert_eq!(
                    draw(style, pose, Gaze::default()),
                    draw(style, pose, Gaze::new(1.0, -1.0)),
                    "{style:?}/{pose:?}"
                );
            }
        }
        assert_eq!(Gaze::new(f32::NAN, 2.0), Gaze { x: 0.0, y: 1.0 });
        assert_eq!(
            Gaze::new(-5.0, 1.0).offset(),
            (-GAZE_REACH[0], GAZE_REACH[1])
        );
    }
    /// Thinking and working must read as different animations, not one motion at another
    /// moment: every thinking frame must stay measurably far from the nearest working frame
    /// at any time in an 8 s window. This checks distinctness only; the direction of each
    /// (thinking inward, working outward) is the design guide's rule, reviewed by eye.
    #[test]
    fn thinking_and_working_are_different_animations() {
        let mut palette = [[0u32; 256]; 3];
        for i in 0..256u32 {
            let ramp = |r: u32, g: u32, b: u32| {
                (r * i / 255) << 24 | (g * i / 255) << 16 | (b * i / 255) << 8 | 0xff
            };
            palette[0][i as usize] = ramp(255, 60, 0);
            palette[1][i as usize] = ramp(0, 255, 60);
            palette[2][i as usize] = ramp(60, 0, 255);
        }
        let draw = |style, pose, t: f32| {
            let frame = Frame::new(style, pose, 0.5, t, Gaze::default());
            (0..32 * 32)
                .flat_map(|i| {
                    let (x, y) = ((i % 32) as f32 + 0.5, (i / 32) as f32 + 0.5);
                    let [r, g, b, a] = frame.rgba(x / 16. - 1., y / 16. - 1., 1. / 16., &palette);
                    [r, g, b].map(|c| c as f32 * a as f32 / 255.)
                })
                .collect::<Vec<f32>>()
        };
        for style in AvatarStyle::ALL {
            let working: Vec<_> = (0..32)
                .map(|k| draw(style, AvatarPose::Working, k as f32 * 0.25))
                .collect();
            let nearest = [0.7, 2.3, 4.1]
                .map(|t| {
                    let thinking = draw(style, AvatarPose::Thinking, t);
                    working
                        .iter()
                        .map(|w| {
                            let (d, n) = thinking.iter().zip(w).fold((0., 0.), |(d, n), (a, b)| {
                                (d + (a - b).abs(), n + a.max(*b))
                            });
                            d / f32::max(n, 1.)
                        })
                        .fold(f32::INFINITY, f32::min)
                })
                .into_iter()
                .fold(f32::INFINITY, f32::min);
            assert!(
                nearest > 0.1,
                "{style:?}: thinking looks like working ({nearest:.3})"
            );
        }
    }
    #[test]
    fn old_ids_and_watch_mapping_are_stable() {
        assert_eq!(AvatarStyle::from_id("halo"), Some(AvatarStyle::Halo));
        assert_eq!(AvatarStyle::Rover.watch_code(), "Q");
        assert_eq!(AvatarStyle::from_id("future"), None);
        assert_eq!(AvatarStyle::from_id("eye"), None);
        assert_eq!(AvatarStyle::resolve("eye"), AvatarStyle::WatchEye);
    }
}
