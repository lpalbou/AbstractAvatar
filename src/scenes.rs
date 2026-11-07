//! Procedural Apple scene vocabulary, expressed as bounded material-space strokes.
//! References are the read-only Orb*.swift and OrbWeave.swift implementations.
//! Scene geometry is generated once; a spatial index bounds per-pixel work.
use super::raster::{Color, Shape, Surface};
use super::{AvatarPose as Pose, AvatarStyle as Style, Material, Sample};
use std::f32::consts::{PI, TAU};

const CELLS: usize = 48;
const EXTENT: f32 = 1.5;
const MAX_PRIMITIVES: usize = 4096;

#[derive(Clone, Copy, Debug)]
struct Stroke {
    a: (f32, f32),
    b: (f32, f32),
    radius: f32,
    light: f32,
    material: Material,
    delta: (f32, f32),
    inverse_length2: f32,
    radius2: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct Scene {
    strokes: Vec<Stroke>,
    cells: Vec<Vec<u16>>,
    surfaces: Vec<Surface>,
    surface_cells: Vec<Vec<u16>>,
}

impl Scene {
    fn new() -> Self {
        Self {
            strokes: Vec::new(),
            cells: vec![Vec::new(); CELLS * CELLS],
            surfaces: Vec::new(),
            surface_cells: vec![Vec::new(); CELLS * CELLS],
        }
    }
    fn add(&mut self, a: (f32, f32), b: (f32, f32), radius: f32, light: f32, material: Material) {
        if self.strokes.len() + self.surfaces.len() >= MAX_PRIMITIVES
            || ![a.0, a.1, b.0, b.1, radius, light]
                .iter()
                .all(|v| v.is_finite())
            || radius <= 0.
            || light <= 0.
        {
            return;
        }
        let radius = radius.clamp(0.002, 0.15);
        let bounds = (
            a.0.min(b.0) - radius * 2.5 - 0.032,
            a.1.min(b.1) - radius * 2.5 - 0.032,
            a.0.max(b.0) + radius * 2.5 + 0.032,
            a.1.max(b.1) + radius * 2.5 + 0.032,
        );
        if bounds.0 > EXTENT || bounds.1 > EXTENT || bounds.2 < -EXTENT || bounds.3 < -EXTENT {
            return;
        }
        let i = self.strokes.len() as u16;
        self.strokes.push(Stroke {
            a,
            b,
            radius,
            light: light.clamp(0., 1.),
            material,
            delta: (b.0 - a.0, b.1 - a.1),
            inverse_length2: {
                let len = (b.0 - a.0).powi(2) + (b.1 - a.1).powi(2);
                if len > 1e-9 {
                    1. / len
                } else {
                    0.
                }
            },
            radius2: radius * radius,
        });
        for y in cell(bounds.1)..=cell(bounds.3) {
            for x in cell(bounds.0)..=cell(bounds.2) {
                self.cells[y * CELLS + x].push(i);
            }
        }
    }
    fn dot(&mut self, x: f32, y: f32, r: f32, light: f32, m: Material) {
        self.add((x, y), (x, y), r, light, m);
    }
    fn surface(
        &mut self,
        shape: Shape,
        material: Material,
        light: f32,
        opacity: f32,
        shading: u8,
        additive: bool,
    ) {
        if self.strokes.len() + self.surfaces.len() >= MAX_PRIMITIVES {
            return;
        }
        if let Shape::Triangle(p) = shape {
            let area =
                (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[1].1 - p[0].1) * (p[2].0 - p[0].0);
            if !area.is_finite() || area.abs() < 1e-10 {
                return;
            }
        }
        let surface = Surface::new(shape, material, light, opacity, shading, additive);
        let b = surface.bounds();
        if ![b.0, b.1, b.2, b.3, light, opacity]
            .iter()
            .all(|x| x.is_finite())
            || b.0 >= b.2
            || b.1 >= b.3
        {
            return;
        }
        let i = self.surfaces.len() as u16;
        self.surfaces.push(surface);
        for y in cell(b.1 - 0.016)..=cell(b.3 + 0.016) {
            for x in cell(b.0 - 0.016)..=cell(b.2 + 0.016) {
                self.surface_cells[y * CELLS + x].push(i);
            }
        }
    }
    fn glow_line(&mut self, a: (f32, f32), b: (f32, f32), r: f32, light: f32, m: Material) {
        self.line(a, b, r * 3.2, light * 0.09, m);
        self.line(a, b, r, light, m);
    }
    pub(crate) fn rgba(&self, x: f32, y: f32, pixel: f32, palette: &super::Palette) -> [u8; 4] {
        self.color(x, y, pixel, palette).rgba()
    }
    fn color(&self, x: f32, y: f32, pixel: f32, palette: &super::Palette) -> Color {
        // At most two subdivisions (16 samples), only for tiny thumbnails.
        // The common 64px+ path uses one spatial lookup per output pixel.
        if pixel > 0.03125 {
            let mut sum = Color::default();
            for dy in [-0.25, 0.25] {
                for dx in [-0.25, 0.25] {
                    let c = self.color(x + dx * pixel, y + dy * pixel, pixel * 0.5, palette);
                    for i in 0..3 {
                        sum.rgb[i] += c.rgb[i] * 0.25;
                    }
                    sum.alpha += c.alpha * 0.25;
                }
            }
            return sum;
        }
        if x.abs() > EXTENT || y.abs() > EXTENT {
            return Color::default();
        }
        let index = cell(y) * CELLS + cell(x);
        let mut color = Color::default();
        for &i in &self.surface_cells[index] {
            let surface = &self.surfaces[i as usize];
            let layer = Color::from_sample(surface.sample(x, y, pixel), palette);
            color = if surface.additive {
                layer.light(color)
            } else {
                layer.over(color)
            };
        }
        let mut levels = [0.0f32; 3];
        for &i in &self.cells[index] {
            let s = self.strokes[i as usize];
            let (dx, dy) = s.delta;
            let u = (((x - s.a.0) * dx + (y - s.a.1) * dy) * s.inverse_length2).clamp(0., 1.);
            let d2 = (x - s.a.0 - u * dx).powi(2) + (y - s.a.1 - u * dy).powi(2);
            // Integrating a Gaussian over the pixel footprint avoids flickering
            // subpixel points and retains their energy rather than fattening them.
            let radius2 = s.radius2;
            let variance = radius2 + pixel * pixel / 6.;
            if d2 > 6.25 * variance {
                continue;
            }
            let energy = if s.inverse_length2 > 0. {
                (radius2 / variance).sqrt()
            } else {
                radius2 / variance
            };
            levels[s.material as usize] += (-d2 / variance).exp() * s.light * energy;
        }
        let mut lights = Color::default();
        for (i, value) in levels.into_iter().enumerate() {
            if value < 0.001 {
                continue;
            }
            lights = Color::from_sample(
                Sample {
                    material: [Material::Warm, Material::Eye, Material::Dim][i],
                    light: (0.38 + value * 0.62).min(1.),
                    coverage: (value * 1.6).sqrt().min(1.),
                },
                palette,
            )
            .light(lights);
        }
        lights.over(color)
    }
    fn line(&mut self, a: (f32, f32), b: (f32, f32), r: f32, light: f32, m: Material) {
        self.add(a, b, r, light, m);
    }
    fn path(&mut self, points: &[(f32, f32)], closed: bool, r: f32, light: f32, m: Material) {
        for v in points.windows(2) {
            self.line(v[0], v[1], r, light, m);
        }
        if closed && points.len() > 2 {
            self.line(*points.last().unwrap(), points[0], r, light, m);
        }
    }
    fn ellipse(
        &mut self,
        c: (f32, f32),
        radii: (f32, f32),
        angle: f32,
        r: f32,
        light: f32,
        m: Material,
    ) {
        let (rx, ry) = radii;
        let points: Vec<_> = (0..64)
            .map(|i| {
                let a = TAU * i as f32 / 64.;
                let (x, y) = (rx * a.cos(), ry * a.sin());
                (
                    c.0 + x * angle.cos() - y * angle.sin(),
                    c.1 + x * angle.sin() + y * angle.cos(),
                )
            })
            .collect();
        self.path(&points, true, r, light, m);
    }
    pub(crate) fn sample(&self, x: f32, y: f32) -> Sample {
        if !x.is_finite() || !y.is_finite() || x.abs() > EXTENT || y.abs() > EXTENT {
            return Sample::EMPTY;
        }
        let mut levels = [0.0f32; 3];
        for &i in &self.surface_cells[cell(y) * CELLS + cell(x)] {
            let p = self.surfaces[i as usize].sample(x, y, 0.);
            levels[p.material as usize] += p.coverage * p.light;
        }
        for &i in &self.cells[cell(y) * CELLS + cell(x)] {
            let s = self.strokes[i as usize];
            let (dx, dy) = (s.b.0 - s.a.0, s.b.1 - s.a.1);
            let len = dx * dx + dy * dy;
            let u = if len > 1e-9 {
                (((x - s.a.0) * dx + (y - s.a.1) * dy) / len).clamp(0., 1.)
            } else {
                0.
            };
            let d2 = (x - s.a.0 - u * dx).powi(2) + (y - s.a.1 - u * dy).powi(2);
            if d2 > 6.25 * s.radius * s.radius {
                continue;
            }
            levels[s.material as usize] += (-d2 / (s.radius * s.radius)).exp() * s.light;
        }
        let (index, value) = levels
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        if value < 0.001 {
            return Sample::EMPTY;
        }
        Sample {
            material: [Material::Warm, Material::Eye, Material::Dim][index],
            light: (0.38 + value * 0.62).clamp(0., 1.),
            coverage: (value * 1.6).sqrt().clamp(0., 1.),
        }
    }
}
fn cell(x: f32) -> usize {
    (((x + EXTENT) / (EXTENT * 2.) * CELLS as f32).floor() as i32).clamp(0, CELLS as i32 - 1)
        as usize
}
fn fract(x: f32) -> f32 {
    x - x.floor()
}
fn hash(i: usize, salt: f32) -> f32 {
    // Integer avalanche avoids platform-libm differences being amplified by fract(sin()*N).
    // Exactly representable 24-bit output gives native/Wasm identical point placement.
    let mut x = (i as u32)
        .wrapping_add(salt.to_bits().rotate_left(13))
        .wrapping_add(0x9e3779b9);
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    (x >> 8) as f32 * (1.0 / 16777216.0)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let u = ((x - a) / (b - a)).clamp(0., 1.);
    u * u * (3. - 2. * u)
}
fn rotate(x: f32, y: f32, a: f32) -> (f32, f32) {
    (x * a.cos() - y * a.sin(), x * a.sin() + y * a.cos())
}
fn mix(a: f32, b: f32, w: f32) -> f32 {
    a + (b - a) * w
}

#[derive(Clone, Copy)]
struct State {
    pose: Pose,
    level: f32,
    t: f32,
    bands: [f32; 8],
    /// Whether the host measured `bands`; otherwise every band equals `level`.
    measured: bool,
    /// Bounded eye shift; zero unless the design follows gaze. Only eyes may use it.
    gaze: (f32, f32),
}
impl State {
    fn look(self, (x, y): (f32, f32)) -> (f32, f32) {
        (x + self.gaze.0, y + self.gaze.1)
    }
    fn listening(self) -> f32 {
        if self.pose == Pose::Listening {
            1.
        } else {
            0.
        }
    }
    fn speaking(self) -> f32 {
        if self.pose == Pose::Speaking {
            1.
        } else {
            0.
        }
    }
    fn thinking(self) -> f32 {
        if self.pose == Pose::Thinking {
            1.
        } else {
            0.
        }
    }
    fn working(self) -> f32 {
        if self.pose == Pose::Working {
            1.
        } else {
            0.
        }
    }
    fn voice(self) -> f32 {
        self.level * (self.listening() + self.speaking())
    }
    fn motion(self) -> f32 {
        0.12 + 0.65 * self.thinking() + 1.35 * self.working()
    }
    fn material(self) -> Material {
        if self.pose == Pose::Error {
            Material::Eye
        } else {
            Material::Warm
        }
    }
    fn light(self) -> f32 {
        if self.pose == Pose::Waiting {
            0.65
        } else {
            1.
        }
    }
    fn band(self, i: usize) -> f32 {
        self.bands[i % 8] * (self.listening() + self.speaking())
    }
}

pub(crate) fn build(
    style: Style,
    pose: Pose,
    level: f32,
    t: f32,
    bands: Option<[f32; 8]>,
    gaze: (f32, f32),
) -> Scene {
    let s = State {
        pose,
        level,
        t,
        bands: bands.unwrap_or([level; 8]),
        measured: bands.is_some(),
        gaze,
    };
    let mut out = Scene::new();
    match style {
        Style::WatchHalo
        | Style::WatchVortex
        | Style::WatchNebula
        | Style::WatchLiquid
        | Style::WatchGalaxy => planet(&mut out, style, s),
        Style::WatchRings => rings(&mut out, s),
        Style::WatchLotus => lotus(&mut out, s),
        Style::WatchCrystal => crystal(&mut out, s),
        Style::WatchEclipse => eclipse(&mut out, s),
        Style::WatchSynapse => synapse(&mut out, s),
        Style::WatchWave => wave(&mut out, s),
        Style::WatchWell => well(&mut out, s),
        Style::WatchBlackHole => blackhole(&mut out, s),
        Style::WatchAccretion => accretion(&mut out, s),
        Style::WatchAurora => aurora(&mut out, s),
        Style::WatchMurmuration => murmuration(&mut out, s),
        Style::WatchEye => eye(&mut out, s),
        Style::WatchRover | Style::WatchSpark => machine(&mut out, style, s),
        Style::WatchHover => hover_machine(&mut out, s),
        Style::WatchOcean => ocean(&mut out, s),
        Style::IphoneWeave => weave(&mut out, s),
        // Sampled analytically in lib.rs; Frame never builds a scene for them.
        Style::Halo | Style::Hover | Style::Vortex | Style::Eclipse | Style::Rover => {}
    }
    // Waiting and Error extend the five original Apple phases with readable,
    // non-audio status cues; the avatar identity remains intact underneath.
    if pose == Pose::Waiting {
        for stroke in &mut out.strokes {
            stroke.light *= 0.58;
        }
        out.ellipse((0., 0.), (0.86, 0.86), 0., 0.004, 0.28, Material::Warm);
        for i in 0..3 {
            let a = t * 0.65 + i as f32 * 0.20;
            out.dot(0.86 * a.cos(), 0.86 * a.sin(), 0.012, 0.65, Material::Warm);
        }
    }
    if pose == Pose::Error {
        out.ellipse((0.62, 0.62), (0.12, 0.12), 0., 0.009, 0.65, Material::Eye);
        out.line((0.57, 0.57), (0.67, 0.67), 0.014, 0.85, Material::Eye);
        out.line((0.57, 0.67), (0.67, 0.57), 0.014, 0.85, Material::Eye);
    }
    out
}

// Orb.swift and apple_avatar_engine.ts: a tilted, slowly turning point planet.
// Voice/activity reveals the design; anchored points preserve its identity.
fn planet(out: &mut Scene, style: Style, s: State) {
    let yaw = s.t * 0.08 + s.t * s.motion() * 0.36;
    let tilt = 0.41f32;
    for i in 0..1600 {
        let u = fract(0.5 + i as f32 * 0.7548777);
        let v = fract(0.5 + i as f32 * 0.5698403);
        let sy = 1. - 2. * u;
        let rr = (1. - sy * sy).max(0.).sqrt();
        let a = TAU * v;
        let (sx, sz) = (rr * a.cos(), rr * a.sin());
        let xx = sx * yaw.cos() + sz * yaw.sin();
        let zz = -sx * yaw.sin() + sz * yaw.cos();
        let mut x = xx;
        let mut y = sy * tilt.cos() - zz * tilt.sin();
        let z = sy * tilt.sin() + zz * tilt.cos();
        let lit = (-0.55 * x - 0.35 * y + 0.76 * z).max(0.);
        let land = 0.5 + 0.5 * (1.9 * sx + (1.4 * sz).sin()).sin() * (2.2 * sy - 0.8 * sz).cos();
        let mut light = (0.22 + 0.68 * lit)
            * (0.45 + 0.55 * smooth(0.25, 0.7, land))
            * (0.22 + 0.78 * smooth(-0.25, 0.15, z));
        let u = hash(i, 11.);
        let phase = TAU * hash(i, 12.);
        let band = (hash(i, 13.) * 9.) as usize;
        let r = x.hypot(y).max(0.001);
        let a = y.atan2(x);
        let engagement =
            (s.listening() + s.speaking() + 0.6 * s.thinking() + 0.9 * s.working()).min(1.);
        let signal = (s.listening() + s.speaking()) * (0.12 + 0.88 * s.level);
        let mut blend = engagement * smooth(0.5 - signal, 1. - signal, 0.72 * r + 0.28 * u);
        blend = blend.max(s.thinking() * 0.6 + s.working() * 0.8);
        if i % 6 == 0 {
            blend *= 0.15;
        }
        let (tx, ty, extra) = match style {
            Style::WatchHalo => {
                // Five broad and three finer streamers, broad at the foot and
                // narrowing outward. Voice flows in to listen, out to speak;
                // thought is a turning crown and work a fixed flowing crown.
                // Thought draws the streamers slowly in; work throws them out fast.
                let flow = if s.thinking() > 0. {
                    fract(u - 0.16 * s.t)
                } else {
                    fract(
                        u + s.t
                            * if s.listening() > 0. {
                                -0.4
                            } else if s.working() > 0. {
                                0.95
                            } else {
                                0.4
                            },
                    )
                };
                let crown =
                    a - 0.25 * s.t * s.thinking() - 0.03 * s.t * (s.listening() + s.speaking());
                let stream =
                    0.5 + 0.5 * (0.62 * (5. * crown).cos() + 0.38 * (8. * crown + 1.7).cos());
                let shape = 1. + 0.3 * (3. * a + 1.3).sin() + 0.15 * (5. * a - 0.4).sin();
                let light = (0.5 + 0.5 * (a - 1.57 * s.t).cos()).powi(3);
                let reach = shape
                    * ((s.listening() + s.speaking()) * (0.35 + 1.1 * s.band(band))
                        + 0.38 * s.thinking() * (0.75 + 0.25 * light)
                        + 1.05 * s.working());
                let length = reach * (0.2 + 0.8 * stream.powf(0.6 + 2.2 * flow));
                let rr = 0.97 + length * flow.powf(1.4);
                (
                    a.cos() * rr,
                    a.sin() * rr,
                    (0.35 + 0.55 * s.band(band) + 0.8 * s.thinking() * light + 0.4 * s.working())
                        * (0.5 + 0.5 * stream)
                        * (1. - flow).powi(2)
                        + 0.25 * (1. - flow).powi(6),
                )
            }
            Style::WatchVortex => {
                let rr = 0.55 + (0.75 + 0.55 * s.band(band)) * u * (1. - 0.18 * s.listening());
                // Winding c and turn w set where a crest travels: w/c > 0 carries it
                // inward. Thought winds the arms tight and draws them slowly in; work
                // loosens them and reverses the turn, so the arms stream outward.
                let angle = (band % 2) as f32 * PI
                    + (3. - 1.4 * s.voice() + 0.9 * s.listening() + 1.2 * s.thinking()
                        - 1.1 * s.working())
                        * (rr / 0.55).ln()
                    + s.t * (0.17 + 0.2 * s.voice() + 0.25 * s.thinking() - 1.05 * s.working());
                (rr * angle.cos(), rr * angle.sin(), 0.45 * (1. - 0.5 * u))
            }
            Style::WatchNebula => {
                let g = (-2. * (1. - 0.84 * u).ln()).sqrt();
                let (gx, gy) = (g * phase.cos(), g * phase.sin());
                if band <= 1 {
                    (
                        0.24 * (1. + 0.3 * s.voice()) * gx,
                        0.24 * (1. + 0.3 * s.voice()) * gy,
                        0.15 + 0.7 * (-g * g / 2.).exp(),
                    )
                } else if band <= 6 {
                    let j = (band % 3) as f32;
                    // The lobes fold in and turn slowly to think, push out and wheel to work.
                    let a =
                        TAU * j / 3. + 0.5 + s.t * (0.03 + 0.1 * s.thinking() + 0.55 * s.working());
                    let c = 0.42
                        * (1. + 0.55 * s.voice())
                        * (1. - 0.38 * s.thinking() + 0.22 * s.working());
                    let l = 0.3 * (1. + 0.4 * s.voice());
                    (
                        c * a.cos() + l * gy * a.cos() - 1.5 * l * gx * a.sin(),
                        c * a.sin() + l * gy * a.sin() + 1.5 * l * gx * a.cos(),
                        0.04 + 0.48 * (-g * g / 2.).exp(),
                    )
                } else {
                    // Outer dust streams away from the core while working.
                    let rr = 0.3 + 0.95 * fract(u + 0.22 * s.t * s.working());
                    let a = 2.3 + 3.1 * (band % 2) as f32 + 0.03 * s.t + 1.9 * u;
                    (rr * a.cos(), rr * a.sin(), 0.32 * (1. - 0.6 * u))
                }
            }
            Style::WatchLiquid => {
                // Calm crossing swells; thought sends slow ripples to the centre,
                // work sends quick ripples out from it.
                let calm = (0.06 + 0.14 * s.band(band))
                    * ((3. * x + 2. * y - 1.2 * s.t).sin()
                        + 0.5 * (4. * z - 2. * y + 1.7 * s.t).sin());
                let rr = x.hypot(y);
                let lift = calm * (1. - s.thinking() - s.working())
                    + s.thinking() * 0.06 * (11. * rr + 1.4 * s.t).sin()
                    + s.working() * 0.11 * (11. * rr - 4.6 * s.t).sin();
                (
                    x * (1.12 + 0.22 * s.voice() + lift),
                    y * (1.12 + 0.22 * s.voice() + lift),
                    0.3 + 2. * lift.max(0.),
                )
            }
            Style::WatchGalaxy => {
                let mut gr = if band == 0 {
                    0.3 * u.powf(0.6)
                } else {
                    (-0.55 * (1. - 0.96 * u).ln()).min(1.7) + 0.08
                };
                let mut angle = if band <= 2 {
                    phase
                } else {
                    (band % 2) as f32 * PI
                        + (2.3 + 1.1 * s.thinking() - 0.8 * s.working()) * (gr + 0.08).ln()
                        + (hash(i, 4.) - 0.5) * 0.6
                };
                // Thought winds the arms in around a brighter core and turns slowly;
                // work flings them loose and wide and turns fast.
                angle -= s.t
                    * (0.09
                        + 0.1 * s.thinking()
                        + 0.75 * s.working()
                        + s.voice() * 0.8 / (0.3 + gr));
                gr *= (1. + 0.3 * s.band(band)) * (1. - 0.16 * s.thinking() + 0.12 * s.working());
                (
                    gr * angle.cos(),
                    gr * angle.sin() * 0.6,
                    if band == 0 {
                        0.9 + 0.5 * s.thinking()
                    } else {
                        0.35
                    },
                )
            }
            _ => unreachable!(),
        };
        x = mix(x, tx, blend);
        y = mix(y, ty, blend);
        // Light rings travel slowly inward to think and quickly outward to work.
        light += blend * extra
            + 0.35 * s.working() * (8. * r - 5. * s.t).cos().max(0.).powi(12)
            + 0.3 * s.thinking() * (9. * r + 2.2 * s.t).cos().max(0.).powi(12);
        let material = if s.pose == Pose::Error {
            Material::Eye
        } else if band.is_multiple_of(3) {
            Material::Dim
        } else if band % 3 == 1 {
            Material::Eye
        } else {
            Material::Warm
        };
        out.dot(x * 0.46, y * 0.46, 0.0055, light * s.light(), material);
        if style == Style::WatchHalo && i % 5 == 0 {
            out.dot(
                x * 0.46,
                y * 0.46,
                0.018,
                0.12 * light * s.light(),
                material,
            );
        }
    }
    if s.pose == Pose::Waiting {
        out.ellipse(
            (0., 0.),
            (0.78, 0.78),
            0.,
            0.006,
            0.45 + 0.15 * (s.t * 2.4).sin(),
            Material::Warm,
        );
    }
}

/// How strongly each of the six rings answers the voice, 0..1, low frequencies first.
///
/// With measured bands, ring `i` follows the mean of the bands it spans (eight bands over
/// six rings, so neighbours share one band). A host that sends only a level gives no
/// spectrum, so the spread is DERIVED, not measured: each ring follows the level through
/// its own syllable-rate envelope (distinct rates and phases, a slight low-frequency tilt),
/// so the rings still move one by one rather than in unison. Outside listening and
/// speaking every ring is still.
fn ring_drive(s: State) -> [f32; 6] {
    let voiced = s.listening() + s.speaking();
    std::array::from_fn(|i| {
        if s.measured {
            let (lo, hi) = (i * 8 / 6, (i * 8 + 13) / 6);
            let span = &s.bands[lo..hi.min(8)];
            voiced * span.iter().sum::<f32>() / span.len() as f32
        } else {
            let f = i as f32;
            let syllable = 0.5 + 0.5 * (s.t * (4.0 + 1.9 * f) + 1.7 * f).sin();
            let phrase = 0.6 + 0.4 * (s.t * (1.3 + 0.7 * f) + 0.6 * f).sin();
            voiced * s.level * (1. - 0.08 * f) * (0.25 + 0.75 * syllable * phrase)
        }
    })
}
// OrbScenes.RingGeometry: great circles normal to the six opposite-corner
// axes of an icosahedron. Each ring tilts, swells and brightens with its own part of
// the voice: it pushes out while speaking and draws in while listening. Thinking turns
// inward (the cage turns slowly, contracts, and a ring of light closes in on the core);
// working turns outward (the cage spins, the rings swing wide and beads race).
fn rings(out: &mut Scene, s: State) {
    type V = (f32, f32, f32);
    fn cross(a: V, b: V) -> V {
        (
            a.1 * b.2 - a.2 * b.1,
            a.2 * b.0 - a.0 * b.2,
            a.0 * b.1 - a.1 * b.0,
        )
    }
    fn norm(a: V) -> V {
        let l = (a.0 * a.0 + a.1 * a.1 + a.2 * a.2).sqrt();
        (a.0 / l, a.1 / l, a.2 / l)
    }
    let phi = (1. + 5f32.sqrt()) / 2.;
    let axes = [
        (0., 1., phi),
        (0., 1., -phi),
        (1., phi, 0.),
        (-1., phi, 0.),
        (phi, 0., 1.),
        (phi, 0., -1.),
    ];
    let drive = ring_drive(s);
    let turn = s.t * (0.12 + 0.16 * s.thinking() + 0.95 * s.working());
    let tilt = 0.42 + 0.1 * (s.t * 0.21).sin();
    let scale = 1. - 0.1 * s.thinking() + 0.03 * s.working();
    for (i, axis) in axes.into_iter().enumerate() {
        let f = i as f32;
        let n = norm(axis);
        let u = norm(cross(
            n,
            if n.1.abs() < 0.9 {
                (0., 1., 0.)
            } else {
                (1., 0., 0.)
            },
        ));
        let v = cross(n, u);
        let d = drive[i];
        let side = if i % 2 == 0 { 1. } else { -1. };
        // The band sets the ring's tilt and size directly, never its speed: a changing
        // level must not jump a ring's phase.
        let theta = 0.04 * (s.t * (0.4 + 0.15 * f)).sin()
            + side * 0.34 * d
            + 0.16 * s.thinking() * (s.t * 0.6 + f).sin()
            + 0.45 * s.working() * (s.t * 3. - f * PI / 3.).sin();
        let v = (
            v.0 * theta.cos() + n.0 * theta.sin(),
            v.1 * theta.cos() + n.1 * theta.sin(),
            v.2 * theta.cos() + n.2 * theta.sin(),
        );
        let radius = 0.46 * scale * (0.97 + d * (0.2 * s.speaking() - 0.13 * s.listening()));
        let spin = (s.t * (0.1 + 0.05 * f + 0.25 * s.thinking()) + 0.9 * d) * side;
        let mut previous = None;
        for j in 0..=96 {
            let angle = TAU * j as f32 / 96.;
            let (x, y, z) = (
                radius * (angle.cos() * u.0 + angle.sin() * v.0),
                radius * (angle.cos() * u.1 + angle.sin() * v.1),
                radius * (angle.cos() * u.2 + angle.sin() * v.2),
            );
            let (bx, bz) = (
                x * turn.cos() + z * turn.sin(),
                -x * turn.sin() + z * turn.cos(),
            );
            let point = (bx, y * tilt.cos() - bz * tilt.sin());
            let depth = smooth(-0.5, 0.5, y * tilt.sin() + bz * tilt.cos());
            let light = ((0.2 + 0.8 * depth) * (0.75 + 0.6 * d) + 0.08 * s.thinking()) * s.light();
            let m = if s.pose == Pose::Error {
                Material::Eye
            } else if (angle / TAU + f / 6.).fract() < 0.33 {
                Material::Dim
            } else if (angle / TAU + f / 6.).fract() < 0.66 {
                Material::Eye
            } else {
                Material::Warm
            };
            if let Some(p) = previous {
                out.glow_line(p, point, 0.0045 * (1. + 0.5 * d), light, m);
                let calm = (5. * (angle - spin)).cos().max(0.).powi(12);
                let race = (5. * (angle - spin - 3. * s.t)).cos().max(0.).powi(12);
                let bead = calm * (1. - s.working()) + race * s.working();
                if bead > 0.4 {
                    out.line(p, point, 0.009, bead * (0.35 + 0.65 * depth), Material::Dim);
                }
            }
            previous = Some(point);
        }
    }
    if s.thinking() > 0. {
        let u = fract(s.t * 0.3);
        let r = 0.44 * (1. - u) + 0.05;
        out.ellipse(
            (0., 0.),
            (r, r),
            0.,
            0.004,
            0.5 * (PI * u).sin(),
            Material::Dim,
        );
    }
}

fn lotus(out: &mut Scene, s: State) {
    for layer in 0..2 {
        for petal in 0..8 {
            let a =
                TAU * (petal as f32 + layer as f32 * 0.5) / 8. + s.t * (0.05 + 0.09 * s.thinking());
            let open = 0.38
                + 0.3 * s.listening()
                + 0.48 * s.speaking()
                + 0.15 * s.thinking()
                + 0.22 * s.voice();
            let r = (0.48 + layer as f32 * 0.16) * (1.0 + 0.12 * s.band(petal));
            for i in 0..70 {
                let u = i as f32 / 69.;
                let spread = (PI * u).sin() * open;
                let spine = r * u;
                let arc = a + (u * 2. - 1.) * spread;
                let x = spine * arc.cos();
                let y = 0.2 - spine * arc.sin() - 0.2 * u * u;
                out.dot(x, y, 0.006, 0.3 + 0.55 * u, s.material());
                if i % 3 == 0 {
                    let inner = spine * (1. - 0.25 * (PI * u).sin());
                    out.dot(
                        inner * arc.cos(),
                        0.2 - inner * arc.sin() - 0.2 * u * u,
                        0.005,
                        0.28,
                        Material::Dim,
                    );
                }
            }
            if s.pose == Pose::Working {
                let tip = (r * a.cos(), 0.2 - r * a.sin() - 0.2);
                out.line((0., 0.18), tip, 0.008, 0.65, s.material());
                out.dot(tip.0, tip.1, 0.025, 0.9, s.material());
            }
        }
    }
}
fn crystal(out: &mut Scene, s: State) {
    let turn = s.t * (0.08 + 0.15 * s.thinking());
    let spread = 0.2 + 0.14 * s.voice() + 0.2 * s.speaking();
    for k in 0..7 {
        let a = TAU * k as f32 / 7. + turn;
        let r = if k == 0 {
            0.62
        } else {
            0.34 + 0.09 * (k % 3) as f32
        };
        let c = (a.cos() * spread, a.sin() * spread * 0.5);
        let pts = [
            (c.0, c.1 - r),
            (c.0 + 0.12, c.1 - 0.38 * r),
            (c.0 + 0.14, c.1 + 0.5 * r),
            (c.0, c.1 + r),
            (c.0 - 0.14, c.1 + 0.5 * r),
            (c.0 - 0.12, c.1 - 0.38 * r),
        ];
        out.path(&pts, true, 0.006, 0.75, s.material());
        out.line(pts[0], pts[3], 0.005, 0.4, Material::Dim);
        out.line(pts[1], pts[5], 0.004, 0.45, Material::Eye);
        if s.pose == Pose::Working {
            out.ellipse(c, (0.28, 0.13), turn + k as f32, 0.005, 0.5, s.material());
        }
    }
}
fn eclipse(out: &mut Scene, s: State) {
    for i in 0..96 {
        let a = TAU * i as f32 / 96.;
        let r = 0.43 + 0.04 * s.voice();
        let reach = 0.06 + 0.15 * (0.5 + 0.5 * (a * 7. + s.t * 0.3).sin()) + 0.14 * s.band(i % 8);
        out.line(
            (r * a.cos(), r * a.sin()),
            ((r + reach) * a.cos(), (r + reach) * a.sin()),
            0.005,
            0.55,
            s.material(),
        );
    }
    out.ellipse((0., 0.), (0.43, 0.43), 0., 0.011, 0.85, s.material());
    if s.pose == Pose::Working {
        for k in 0..7 {
            let a = s.t + TAU * k as f32 / 7.;
            out.line(
                (0.18 * a.cos(), 0.18 * a.sin()),
                (0.41 * (a + 0.55).cos(), 0.41 * (a + 0.55).sin()),
                0.008,
                0.65,
                Material::Eye,
            );
        }
    }
    if s.pose == Pose::Thinking {
        out.ellipse(
            (0., 0.),
            (0.72, 0.23),
            s.t * 0.5,
            0.006,
            0.65,
            Material::Dim,
        );
    }
}
// Thought gathers the nodes inward into a denser tangle, turning slowly, with signals
// running along the links toward the centre. Work spreads the network, orbits it fast,
// and fires signals outward.
fn synapse(out: &mut Scene, s: State) {
    let gather = 1. - 0.2 * s.thinking() + 0.08 * s.working();
    let points: Vec<_> = (0..38)
        .map(|i| {
            let a = TAU * hash(i, 4.) + s.t * (0.08 + s.thinking() * 0.04 + s.working() * 0.48);
            let r = (0.12 + 0.62 * hash(i, 7.).sqrt()) * (1. + 0.15 * s.voice()) * gather;
            let r = if s.pose == Pose::Speaking {
                r * (0.7 + 0.3 * (6. * a).cos().abs())
            } else {
                r
            };
            (r * a.cos(), r * a.sin())
        })
        .collect();
    let reach = 0.32 * gather;
    for (i, &a) in points.iter().enumerate() {
        for (j, &b) in points.iter().enumerate().skip(i + 1) {
            if (a.0 - b.0).hypot(a.1 - b.1) < reach && (i + j) % 3 == 0 {
                out.line(
                    a,
                    b,
                    0.003,
                    0.3 + 0.3 * (s.t * 2. - j as f32).sin().max(0.),
                    Material::Dim,
                );
                let busy = s.thinking() + s.working();
                if busy > 0. {
                    let (inner, outer) = if a.0.hypot(a.1) < b.0.hypot(b.1) {
                        (a, b)
                    } else {
                        (b, a)
                    };
                    let (from, to) = if s.working() > 0. {
                        (inner, outer)
                    } else {
                        (outer, inner)
                    };
                    let u = fract(s.t * (0.35 + 1.25 * s.working()) + hash(i * 38 + j, 9.));
                    out.dot(
                        mix(from.0, to.0, u),
                        mix(from.1, to.1, u),
                        0.008,
                        0.75 * (PI * u).sin(),
                        Material::Eye,
                    );
                }
            }
        }
        out.dot(
            a.0,
            a.1,
            0.012 + 0.006 * s.band(i),
            0.6 + 0.3 * (s.t * 1.2 + i as f32).sin().max(0.),
            s.material(),
        );
    }
}
// OrbConcepts.drawWave: five translucent ribbons, outgoing/incoming voice
// wavefronts, and five spiral arms while tools work. OrbShape.wave is separate.
fn wave(out: &mut Scene, s: State) {
    let voiced = s.listening() + s.speaking();
    for k in 0..5 {
        let fk = k as f32;
        let band = s.band(k * 7 / 4);
        let rest = 0.1 * (1. - 0.12 * fk) * (0.8 + 0.2 * (s.t * 0.7 + fk).sin());
        let amp = rest * (1. - voiced - s.thinking())
            + voiced * (0.12 + 1.1 * band + 0.25 * s.voice())
            + s.thinking() * (0.32 + 0.08 * (s.t * 0.5 + fk * 1.7).sin());
        let freq = 1.3 + 0.55 * fk;
        let pace = 0.6 + 0.25 * fk;
        let counter = 0.4 * s.t * if k % 2 == 0 { 1. } else { -1. };
        let mut prev = None;
        for i in 0..=72 {
            let u = i as f32 / 72.;
            let x = 2. * u - 1.;
            let env = (1. - x * x).powi(2) * (0.35 + 0.65 * (-x * x / 0.22).exp());
            let calm = (freq * PI * x - pace * s.t + fk * 1.3).sin();
            let thought = (freq * PI * x - 0.5 * pace * s.t + fk * 1.3 + counter).sin();
            let outgoing = (freq * PI * x.abs() - 3.2 * pace * s.t + fk * 1.3).sin();
            let incoming = (-freq * PI * x.abs() - 3.2 * pace * s.t + fk * 1.3).sin();
            let y = amp
                * env
                * ((1. - voiced - s.thinking()) * calm
                    + s.thinking() * thought
                    + s.speaking() * outgoing
                    + s.listening() * incoming);
            let angle = TAU * 1.5 * u + s.t * 1.6 + fk * TAU / 5.;
            let radius = 0.8 * (0.12 + 0.88 * u) * (1. + 0.06 * (u * 12. - s.t * 4.).sin());
            let outer = (
                mix(x * 0.94, radius * angle.cos(), s.working()),
                mix(-y * 0.6, radius * angle.sin() * 0.92, s.working()),
            );
            let inner = (
                mix(x * 0.94, radius * 0.8 * (angle - 0.12).cos(), s.working()),
                mix(
                    -y * 0.6 * 0.12,
                    radius * 0.8 * (angle - 0.12).sin() * 0.92,
                    s.working(),
                ),
            );
            let m = if s.pose == Pose::Error {
                Material::Eye
            } else if u < 0.33 {
                Material::Dim
            } else if u < 0.67 {
                Material::Eye
            } else {
                Material::Warm
            };
            if let Some((po, pi)) = prev {
                out.surface(Shape::Triangle([po, outer, inner]), m, 0.75, 0.20, 0, true);
                out.surface(Shape::Triangle([po, inner, pi]), m, 0.75, 0.20, 0, true);
                out.glow_line(po, outer, 0.006, 0.78 * s.light(), m);
            }
            prev = Some((outer, inner));
        }
    }
    if s.working() < 0.5 {
        out.line((-0.94, 0.), (0.94, 0.), 0.002, 0.14, s.material());
    }
}

fn well(out: &mut Scene, s: State) {
    for axis in 0..2 {
        for k in 0..21 {
            let v = (k as f32 / 20. - 0.5) * 1.8;
            let pts: Vec<_> = (0..70)
                .map(|i| {
                    let u = (i as f32 / 69. - 0.5) * 1.8;
                    let (x, z) = if axis == 0 { (u, v) } else { (v, u) };
                    let r = x.hypot(z);
                    let depth =
                        -(0.16 + 0.18 * s.thinking() + 0.22 * s.working()) / (1. + r * r * 18.);
                    let wave = (10. * r + s.t * if s.pose == Pose::Listening { 3.2 } else { -3.2 })
                        .sin()
                        * (0.015 + 0.07 * s.voice())
                        * smooth(0.05, 0.3, r);
                    let (x, z) = rotate(x, z, 0.35 + 0.1 * (s.t * 0.07).sin());
                    (x, z * 0.48 - depth + wave - 0.15)
                })
                .collect();
            out.path(&pts, false, 0.003, 0.4, s.material());
        }
    }
    let d = 0.11 * s.thinking() + 0.17 * s.working();
    let a = s.t * (1. + s.working());
    for side in [-1., 1.] {
        out.dot(
            side * d * a.cos(),
            side * d * a.sin() * 0.45,
            0.025,
            1.,
            Material::Eye,
        );
    }
}
fn blackhole(out: &mut Scene, s: State) {
    for i in 0..1600 {
        let u = hash(i, 2.);
        let a = TAU * fract(i as f32 * 0.7548777) + s.t * (0.1 + 0.4 * s.thinking()) / (0.5 + u);
        let r = 0.27 + 0.35 * u + 0.035 * s.voice() * (1. - u);
        let y = r * a.sin() * 0.28;
        let bend = if y < 0. {
            -0.20 * (1. - (r * a.cos() / 0.6).powi(2)).max(0.)
        } else {
            0.
        };
        out.dot(
            r * a.cos(),
            y + bend,
            0.005,
            0.25 + 0.65 * (1. - u) + 0.22 * s.band(i % 8),
            s.material(),
        );
    }
    let ring = 0.26 + 0.018 * s.voice();
    out.ellipse(
        (0., -0.01),
        (ring, ring),
        0.,
        0.012 + 0.008 * s.voice(),
        0.9,
        Material::Eye,
    );
    if s.pose == Pose::Working {
        for side in [-1., 1.] {
            out.line(
                (0., side * 0.26),
                (0.035 * (s.t * 3.).sin(), side * 0.86),
                0.014,
                0.65,
                s.material(),
            );
        }
    }
}
// Thought gathers the dust into tight clumps that turn slowly while a ring of light
// closes in on the star. Work merges each ring into a planet, speeds the orbits and sends
// a ring of light outward.
fn accretion(out: &mut Scene, s: State) {
    out.dot(0., 0., 0.065 + 0.025 * s.voice(), 1., s.material());
    let pace = 1. - 0.45 * s.thinking() + 1.4 * s.working();
    let sharp = if s.thinking() > 0. { 7 } else { 3 };
    for ring in 0..6 {
        let r = 0.22 + ring as f32 * 0.1 + 0.018 * s.band(ring);
        let a = s.t * (0.4 / (0.3 + r)) * pace;
        out.ellipse((0., 0.), (r, r * 0.38), 0.15, 0.004, 0.35, Material::Dim);
        for k in 0..90 {
            let q = TAU * k as f32 / 90. + a;
            let clump = 0.4
                + 0.6
                    * (q * (3. + ring as f32) - s.t * 0.3)
                        .cos()
                        .max(0.)
                        .powi(sharp);
            let (x, y) = rotate(r * q.cos(), r * q.sin() * 0.38, 0.15);
            out.dot(
                x,
                y,
                0.005 + 0.004 * clump * s.thinking(),
                (0.3 + 0.4 * clump) * (1. - 0.35 * s.thinking() * (1. - clump))
                    + 0.20 * s.band(ring),
                s.material(),
            );
        }
        if s.pose == Pose::Working {
            let (x, y) = rotate(r * a.cos(), r * a.sin() * 0.38, 0.15);
            out.dot(x, y, 0.018, 0.9, Material::Eye);
        }
    }
    if s.thinking() + s.working() > 0. {
        let u = fract(s.t * (0.22 + 0.33 * s.working()));
        let r = if s.working() > 0. {
            0.12 + 0.62 * u
        } else {
            0.74 - 0.62 * u
        };
        out.ellipse(
            (0., 0.),
            (r, r * 0.38),
            0.15,
            0.006,
            0.55 * (PI * u).sin(),
            Material::Eye,
        );
    }
}
fn aurora(out: &mut Scene, s: State) {
    for layer in 0..3 {
        let f = layer as f32;
        for i in 0..70 {
            let u = i as f32 / 69.;
            let x = (u - 0.5) * 1.8;
            let base = 0.32 - 0.09 * f + 0.06 * (x * 3. + s.t * 0.25 + f * 1.9).sin();
            let height = (0.3 + f * 0.09)
                * (0.7 + 0.3 * (x * 3. + s.t * 0.5 + f).sin())
                * (1. + s.band(i % 8));
            let a = TAU * u + f * 2. + s.t * 0.25;
            let start = (
                mix(x, 0.02 * a.cos(), s.thinking()),
                mix(base, -0.3 + 0.02 * a.sin(), s.thinking()),
            );
            let end = (
                mix(
                    x + 0.025 * (x * 2. + s.t * 0.3).sin(),
                    (0.3 + f * 0.05) * a.cos(),
                    s.thinking(),
                ),
                mix(
                    base - height,
                    -0.3 + (0.3 + f * 0.05) * a.sin(),
                    s.thinking(),
                ),
            );
            let bright =
                0.25 + 0.3 * (x * 7. - s.t * 0.9 + f * 3.).sin().max(0.) + 0.2 * s.working();
            out.line(
                start,
                end,
                0.006,
                bright,
                if layer == 1 {
                    Material::Eye
                } else {
                    s.material()
                },
            );
            out.line(
                (start.0, 0.68 - start.1),
                (end.0, 0.68 - end.1),
                0.004,
                bright * 0.18,
                Material::Dim,
            );
        }
    }
    let ridge: Vec<_> = (0..70)
        .map(|i| {
            let x = i as f32 / 69. * 1.9 - 0.95;
            (
                x,
                0.37 - 0.08 * (x * 6.).sin().abs() - 0.035 * (x * 13.).cos(),
            )
        })
        .collect();
    out.path(&ridge, false, 0.008, 0.28, Material::Dim);
}
fn murmuration(out: &mut Scene, s: State) {
    for i in 0..500 {
        let a = TAU * hash(i, 2.) + s.t * 0.2;
        let u = hash(i, 4.);
        let radius = 0.3 + 0.2 * u + 0.08 * (a * 3. + s.t * 0.4).sin();
        let x = radius * a.cos() * 1.35;
        let y = radius * a.sin() * 0.7;
        let twist = s.thinking() * a * 0.3;
        let mut p = rotate(x, y, twist + s.t * 0.07);
        p.0 += 0.12 * s.working() * (a + s.t).sin();
        p.1 += 0.12 * s.voice() * (a * 3. - s.t * 4.).sin();
        let wing = 0.008 + 0.009 * (s.t * 12. + i as f32).sin().abs();
        out.line(
            (p.0 - wing, p.1 - wing * 0.5),
            (p.0, p.1),
            0.003,
            0.55,
            s.material(),
        );
        out.line(
            (p.0, p.1),
            (p.0 + wing, p.1 - wing * 0.5),
            0.003,
            0.55,
            s.material(),
        );
    }
}
// OrbAvatars.drawEye: asymmetrical lids, iris fibers, dark pupil,
// fixed catchlight, crease, brow and lashes, clipped to the opening.
fn eye(out: &mut Scene, s: State) {
    let blink = super::blink(s.t, 5.3, 0.20);
    let open = (0.8 + 0.18 * s.listening() - 0.2 * s.thinking()) * blink;
    let smile = 0.06 + 0.12 * s.speaking() + 0.06 * s.listening();
    let upper =
        |x: f32| -0.62 * open * (1. - (x / 1.25).powi(2)).max(0.).powf(0.75) + 0.04 * x / 1.25;
    let lower = |x: f32| {
        ((0.42 - smile) * open * (1. - (x / 1.25).powi(2)).max(0.).powf(0.9)
            - 0.1 * smile * (1. - (x / 1.25).powi(2)).max(0.))
            + 0.04 * x / 1.25
    };
    let inside = |x: f32, y: f32| x.abs() < 1.25 && y > upper(x) && y < lower(x);
    let at = |x: f32, y: f32| (x * 0.65, (y + 0.08) * 0.65);
    let fix = (s.t * 2.2) as usize;
    let gx = (0.1 * (s.t * 0.4).sin() + 0.05 * (s.t * 0.13).sin())
        * (1. - s.listening())
        * (1. - s.thinking())
        * (1. - s.working())
        + s.thinking() * (-0.26 + 0.07 * (s.t * 0.7).sin())
        + s.working() * (hash(fix, 81.) - 0.5) * 0.5;
    let gy = 0.03 * (s.t * 0.33).sin() * (1. - s.thinking()) * (1. - s.working())
        - 0.17 * s.thinking()
        + s.working() * (hash(fix, 82.) - 0.5) * 0.18
        + 0.03
        + s.gaze.1 / 0.65;
    let gx = gx + s.gaze.0 / 0.65;
    let pupil = 0.16 + 0.07 * s.listening()
        - 0.05 * s.speaking() * (1.5 * s.level).min(1.)
        - 0.02 * s.thinking()
        - 0.04 * s.working();
    // The sclera's soft reflective field. Its triangles share the exact lid curve.
    for i in 0..64 {
        let x0 = -1.25 + 2.5 * i as f32 / 64.;
        let x1 = -1.25 + 2.5 * (i + 1) as f32 / 64.;
        let a = at(x0, upper(x0));
        let b = at(x1, upper(x1));
        let c = at(x1, lower(x1));
        let d = at(x0, lower(x0));
        out.surface(
            Shape::Triangle([a, b, c]),
            Material::Dim,
            0.55,
            0.14,
            0,
            false,
        );
        out.surface(
            Shape::Triangle([a, c, d]),
            Material::Dim,
            0.55,
            0.14,
            0,
            false,
        );
    }
    // Fibers follow a winding path; the rim and inner collarette remain visible.
    for k in 0..96 {
        let angle = TAU * k as f32 / 96. + s.t * 0.0144;
        let mut prev = None;
        for j in 0..=9 {
            let u = j as f32 / 9.;
            let a = angle + 0.22 * (3. * u + angle * 2.).sin() * (1. - u);
            let r = pupil + (0.46 - pupil) * u;
            let x = gx + r * a.cos();
            let y = gy + r * a.sin();
            if !inside(x, y) {
                prev = None;
                continue;
            }
            let p = at(x, y);
            let lum = 0.30
                + 0.3 * (-((u - 0.32) / 0.08).powi(2)).exp()
                + 0.4 * smooth(0.88, 1., u)
                + 0.55 * s.band(k * 8 / 96) * (0.4 + 0.6 * u);
            if let Some(previous) = prev {
                out.line(
                    previous,
                    p,
                    0.0024,
                    lum,
                    if k % 3 == 0 {
                        Material::Eye
                    } else {
                        s.material()
                    },
                );
            }
            prev = Some(p);
        }
    }
    for k in 0..96 {
        let a = TAU * k as f32 / 96.;
        let p = (gx + pupil * a.cos(), gy + pupil * a.sin());
        if inside(p.0, p.1) {
            let p = at(p.0, p.1);
            out.dot(p.0, p.1, 0.0035, 0.65, s.material());
        }
    }
    // Lids and crease respect the source's asymmetry and smile.
    let up: Vec<_> = (0..71)
        .map(|i| {
            let x = -1.25 + 2.5 * i as f32 / 70.;
            at(x, upper(x))
        })
        .collect();
    let low: Vec<_> = (0..71)
        .map(|i| {
            let x = -1.25 + 2.5 * i as f32 / 70.;
            at(x, lower(x))
        })
        .collect();
    for p in up.windows(2) {
        out.glow_line(p[0], p[1], 0.006, 0.70, s.material());
    }
    out.path(&low, false, 0.004, 0.42, s.material());
    let crease: Vec<_> = (0..57)
        .map(|i| {
            let x = -1.04 + 2.08 * i as f32 / 56.;
            at(
                x,
                upper(x) * 1.05 - 0.13 - 0.05 * open - 0.03 * s.listening(),
            )
        })
        .collect();
    out.path(&crease, false, 0.002, 0.14, Material::Dim);
    let brow: Vec<_> = (0..49)
        .map(|i| {
            let x = -1.1 + 2.25 * i as f32 / 48.;
            at(
                x,
                -0.98 - 0.1 * s.listening() - 0.04 * s.thinking()
                    + 0.2 * (x / 1.15).powi(2)
                    + 0.04 * (1. - open)
                    + 0.03 * x / 1.15,
            )
        })
        .collect();
    out.path(&brow, false, 0.008, 0.35, s.material());
    for k in 0..9 {
        let f = k as f32 / 9.;
        let x = 1.25 * (-0.4 + 1.32 * f);
        if x.abs() > 1.21 {
            continue;
        }
        let len = (0.07 + 0.1 * f * f) * (0.55 + 0.45 * open);
        out.line(
            at(x, upper(x)),
            at(x + (0.2 + 0.8 * f) * len * 0.7, upper(x) - len * 1.05),
            0.003,
            0.60,
            s.material(),
        );
    }
    let catch = at(gx * 0.5 - 0.14, gy * 0.5 - 0.17);
    if inside(gx * 0.5 - 0.14, gy * 0.5 - 0.17) {
        out.dot(catch.0, catch.1, 0.016, 1., Material::Dim);
        out.dot(catch.0, catch.1, 0.045, 0.06, Material::Dim);
    }
}

fn metal_body(out: &mut Scene, c: (f32, f32), rx: f32, ry: f32, n: usize, s: State) {
    for i in 0..n {
        let a = TAU * fract(i as f32 * 0.7548777);
        let r = fract(i as f32 * 0.5698403).sqrt();
        let (x, y) = (r * a.cos(), r * a.sin());
        let z = (1. - r * r).sqrt();
        let light = 0.16
            + 0.58 * (-0.48 * x - 0.55 * y + 0.68 * z).max(0.)
            + 0.15 * (-((x + 0.3).powi(2) + (y + 0.3).powi(2)) / 0.06).exp();
        out.dot(
            c.0 + x * rx,
            c.1 + y * ry,
            0.007,
            light * s.light(),
            Material::Dim,
        );
    }
}
fn lamp(out: &mut Scene, c: (f32, f32), r: f32, light: f32, s: State) {
    out.dot(c.0, c.1, r, (0.55 + 0.4 * s.voice()) * light, s.material());
    out.dot(
        c.0 - r * 0.25,
        c.1 - r * 0.25,
        r * 0.25,
        0.9 * light,
        Material::Eye,
    );
}
// OrbMachines.drawHover: opaque domed barrel, dark visor, articulated
// shoulders and three soft engine plumes, suspended above a floor-light pool.
fn hover_machine(out: &mut Scene, s: State) {
    let near = 1. + 0.12 * s.listening() - 0.05 * s.thinking();
    let bob = 0.025 * (s.t * 1.05).sin() + 0.008 * (s.t * 2.3 + 1.).sin();
    let oy = -0.1 + bob + 0.08 * s.listening() - 0.08 * s.thinking()
        + 0.045 * s.speaking() * s.level
        - 0.02 * s.working();
    let yaw = 0.28 * (s.t * 0.29).sin() * (1. - s.thinking())
        + s.thinking() * (0.5 + 0.05 * (s.t * 0.4).sin());
    let cx = 0.02 * (s.t * 0.71).sin();
    let body = |x: f32, y: f32| (cx + x * near, oy + y * near);
    let rx = 0.47 * near;
    // The pool remains separate from the robot: a visible hover gap.
    out.surface(
        Shape::Ellipse {
            center: (0., 0.79),
            radius: (0.50, 0.075),
        },
        s.material(),
        0.72,
        0.16 + 0.12 * s.working(),
        2,
        true,
    );
    for k in 0..3 {
        let x = (k as f32 - 1.) * 0.24;
        let nozzle = body(x, 0.38);
        let len = 0.11 + 0.11 * s.working() + 0.012 * (s.t * 13. + k as f32).sin();
        out.surface(
            Shape::Ellipse {
                center: (nozzle.0, nozzle.1 + len * 0.65),
                radius: (0.052, len),
            },
            s.material(),
            0.9,
            0.60 + 0.25 * s.working(),
            2,
            true,
        );
        out.surface(
            Shape::Ellipse {
                center: (nozzle.0, nozzle.1),
                radius: (0.033, 0.012),
            },
            Material::Dim,
            1.,
            0.95,
            0,
            true,
        );
    }
    // Rounded arms unfold forward while tools run. Metallic shoulder and wrist.
    for side in [-1., 1.] {
        let a = body(side * 0.46, 0.04);
        let b = body(
            side * (0.50 + 0.18 * s.working()),
            0.23 - 0.06 * s.working(),
        );
        out.glow_line(a, b, 0.045, 0.35, Material::Dim);
        out.surface(
            Shape::Ellipse {
                center: b,
                radius: (0.047, 0.05),
            },
            Material::Dim,
            0.85,
            1.,
            1,
            false,
        );
    }
    out.surface(
        Shape::Ellipse {
            center: body(0., 0.29),
            radius: (rx, 0.13 * near),
        },
        Material::Dim,
        0.68,
        1.,
        1,
        false,
    );
    out.surface(
        Shape::Rounded {
            center: body(0., 0.065),
            half: (rx, 0.25 * near),
            radius: 0.045,
        },
        Material::Dim,
        0.92,
        1.,
        1,
        false,
    );
    out.surface(
        Shape::Ellipse {
            center: body(0., -0.19),
            radius: (rx, 0.25 * near),
        },
        Material::Dim,
        1.,
        1.,
        1,
        false,
    );
    out.surface(
        Shape::Ellipse {
            center: body(-0.025, -0.425),
            radius: (0.105, 0.026),
        },
        Material::Dim,
        0.74,
        1.,
        1,
        false,
    );
    let visor_y = -0.065;
    out.surface(
        Shape::Rounded {
            center: body(0., visor_y),
            half: (rx * 1.002, 0.102 * near),
            radius: 0.034,
        },
        Material::Eye,
        0.025,
        1.,
        0,
        false,
    );
    out.surface(
        Shape::Ellipse {
            center: body(yaw * 0.10, visor_y),
            radius: (0.35, 0.13),
        },
        s.material(),
        0.45,
        0.12 + 0.2 * s.voice(),
        2,
        true,
    );
    for y in [-0.17, 0.04, 0.29] {
        let points: Vec<_> = (0..33)
            .map(|i| {
                let x = (i as f32 / 32. - 0.5) * 0.9;
                body(x, y + 0.025 * (1. - (x / 0.45).powi(2)))
            })
            .collect();
        out.path(
            &points,
            false,
            0.003,
            if y < 0. { 0.60 } else { 0.32 },
            Material::Dim,
        );
    }
    let blink = super::blink(s.t + 1.9, 5.7, 0.20) * (1. - 0.48 * s.thinking());
    for side in [-1., 1.] {
        let eye = s.look(body(
            side * 0.18 * yaw.cos() + yaw.sin() * 0.19,
            visor_y + 0.015,
        ));
        out.surface(
            Shape::Ellipse {
                center: eye,
                radius: (0.086, 0.086),
            },
            Material::Dim,
            0.50,
            1.,
            1,
            false,
        );
        out.surface(
            Shape::Ellipse {
                center: eye,
                radius: (0.066, 0.064 * blink.max(0.14)),
            },
            s.material(),
            0.75 + 0.25 * s.voice(),
            1.,
            0,
            false,
        );
        out.surface(
            Shape::Ellipse {
                center: eye,
                radius: (0.13, 0.10 * blink.max(0.25)),
            },
            s.material(),
            0.70,
            0.20 + 0.30 * s.voice(),
            2,
            true,
        );
        out.dot(
            eye.0 - 0.015,
            eye.1 - 0.018 * blink,
            0.012,
            0.85,
            Material::Dim,
        );
    }
    if s.thinking() > 0. {
        let x = 0.36 * (s.t * 0.7).sin();
        out.dot(body(x, 0.).0, body(0., 0.).1, 0.025, 0.7, s.material());
    }
    if s.working() > 0. {
        let tool = body(0.68, 0.15);
        for k in 0..3 {
            let angle = s.t * 7. + k as f32 * TAU / 3.;
            out.line(
                tool,
                (tool.0 + 0.07 * angle.cos(), tool.1 + 0.07 * angle.sin()),
                0.005,
                0.65,
                s.material(),
            );
        }
    }
}
// OrbRobots Rover and Spark: a tracked body, a neck and binocular eyes.
// Thinking turns inward: the head sways slowly, the eyes look up and dim, and a focus ring
// closes in on each eye. Working turns outward: the treads roll, the head bobs, and light
// runs out from the middle of the chest bar to both ends.
fn machine(out: &mut Scene, style: Style, s: State) {
    let spark = style == Style::WatchSpark;
    let bob = 0.012 * (s.t * 0.85).sin() + 0.025 * s.listening() * (s.t * 1.05).sin();
    let cx = 0.025 * (s.t * 0.3).sin()
        + 0.08 * s.thinking() * (s.t * 0.7).sin()
        + 0.04 * s.working() * (s.t * 2.1).sin();
    let top = -0.25 + bob + 0.05 * s.working() * (s.t * 3.).sin() + 0.04 * s.thinking();
    // Pensive eyes look up and to the side; they are pose-driven, not gaze.
    let ponder = (
        s.thinking() * 0.025 * (s.t * 0.45).sin(),
        -s.thinking() * if spark { 0.03 } else { 0.04 },
    );
    let dim = 1. - s.thinking() * (0.5 - 0.12 * (s.t * 1.1).sin());
    metal_body(
        out,
        (0., 0.30),
        if spark { 0.38 } else { 0.46 },
        0.22,
        600,
        s,
    );
    out.line((0., 0.08), (cx, top + 0.04), 0.022, 0.5, Material::Dim);
    let roll = fract(s.t * 1.8) * s.working();
    for side in [-1., 1.] {
        let x = side * 0.48;
        out.path(
            &[
                (x - 0.09, 0.17),
                (x + 0.09, 0.17),
                (x + 0.11, 0.63),
                (x - 0.11, 0.63),
            ],
            true,
            0.019,
            0.18,
            Material::Dim,
        );
        for k in 0..9 {
            let y = 0.20 + (k as f32 + roll) * 0.048;
            out.line((x - 0.075, y), (x + 0.075, y), 0.009, 0.23, Material::Dim);
        }
        // A pondering head turns a little aside, so its eyes draw together.
        let spacing = if spark { 0.21 } else { 0.24 };
        let ex = cx + side * spacing * (1. - 0.2 * s.thinking());
        let eye = s.look((ex + ponder.0, top + ponder.1));
        if spark {
            // Spark's characteristic boxy binocular shutters, unlike Rover's round lenses.
            let pts = [
                (ex - 0.17, top - 0.14),
                (ex + 0.16, top - 0.11),
                (ex + 0.18, top + 0.13),
                (ex - 0.16, top + 0.16),
            ];
            out.path(&pts, true, 0.016, 0.45, Material::Dim);
            // The shutter blades close a little while thinking.
            let inner = 0.032 + 0.03 * s.thinking();
            for blade in 0..6 {
                let a = TAU * blade as f32 / 6. + 0.15 * s.thinking() * (s.t * 1.4).sin();
                let b = a + 0.85;
                out.line(
                    (ex + inner * a.cos(), top + inner * a.sin()),
                    (ex + 0.12 * b.cos(), top + 0.10 * b.sin()),
                    0.011,
                    0.6,
                    s.material(),
                );
            }
            lamp(out, eye, 0.027, dim, s);
        } else {
            metal_body(out, (ex, top), 0.18, 0.155, 300, s);
            out.ellipse((ex, top), (0.135, 0.118), 0., 0.011, 0.5, Material::Dim);
            lamp(out, eye, 0.069, dim, s);
        }
        if s.thinking() > 0. {
            let u = fract(s.t * 0.42 + side * 0.25);
            let r = 0.17 - 0.10 * u;
            out.ellipse(
                (ex, top),
                (r, r * 0.88),
                0.,
                0.006,
                0.8 * (PI * u).sin(),
                s.material(),
            );
        }
    }
    out.line((-0.23, 0.29), (0.23, 0.29), 0.011, 0.4, s.material());
    if s.working() > 0. {
        for k in 0..2 {
            let u = fract(s.t * 1.6 + k as f32 * 0.5);
            for side in [-1., 1.] {
                out.dot(side * 0.23 * u, 0.29, 0.02, 0.9 * (1. - u), Material::Eye);
            }
        }
    }
    if spark {
        out.line(
            (-0.08, top - 0.15),
            (-0.13, top - 0.31),
            0.008,
            0.5,
            Material::Dim,
        );
        let beacon = if s.working() > 0. {
            0.35 + 0.6 * ((s.t * 5.).floor() as i32 % 2) as f32
        } else {
            0.8
        };
        out.dot(-0.13, top - 0.31, 0.018, beacon, s.material());
    }
    if s.pose == Pose::Error {
        for side in [-1., 1.] {
            let (x, y) = (cx + side * 0.12, top);
            out.line(
                (x - 0.05, y - 0.05),
                (x + 0.05, y + 0.05),
                0.012,
                1.,
                Material::Eye,
            );
            out.line(
                (x - 0.05, y + 0.05),
                (x + 0.05, y - 0.05),
                0.012,
                1.,
                Material::Eye,
            );
        }
    }
}
fn ocean(out: &mut Scene, s: State) {
    for row in 0..27 {
        let z = row as f32 / 26.;
        let pts: Vec<_> = (0..75)
            .map(|i| {
                let x = (i as f32 / 74. - 0.5) * 1.9;
                let amp = (0.015 + 0.04 * z + 0.045 * s.band(row % 8) + 0.025 * s.working())
                    * (1. - 0.4 * s.thinking());
                let y = -0.07
                    + z * 0.85
                    + amp
                        * ((x * 6. + z * 8. - s.t * (1.1 + s.working() * 1.3)).sin()
                            + 0.4 * (x * 13. - z * 7. + s.t * 0.7).sin());
                (x, y)
            })
            .collect();
        out.path(
            &pts,
            false,
            0.003 + 0.003 * z,
            0.23 + 0.43 * z,
            if row % 5 == 0 {
                Material::Eye
            } else {
                Material::Dim
            },
        );
    }
    out.dot(0.22, -0.29, 0.07, 0.65, s.material());
    out.line((-0.87, -0.07), (0.87, -0.07), 0.005, 0.5, s.material());
}

fn weave_hand() -> &'static [(f32, f32)] {
    static SMOOTH: std::sync::LazyLock<Vec<(f32, f32)>> = std::sync::LazyLock::new(|| {
        const HAND: [(f32, f32); 34] = [
            (0.38, 0.90),
            (0.31, 0.78),
            (0.22, 0.67),
            (0.13, 0.56),
            (0.13, 0.51),
            (0.17, 0.48),
            (0.21, 0.50),
            (0.29, 0.60),
            (0.29, 0.28),
            (0.31, 0.22),
            (0.35, 0.22),
            (0.38, 0.25),
            (0.38, 0.48),
            (0.40, 0.49),
            (0.41, 0.16),
            (0.43, 0.11),
            (0.47, 0.11),
            (0.50, 0.15),
            (0.50, 0.46),
            (0.52, 0.47),
            (0.53, 0.22),
            (0.55, 0.17),
            (0.59, 0.17),
            (0.62, 0.21),
            (0.62, 0.51),
            (0.64, 0.52),
            (0.65, 0.34),
            (0.67, 0.29),
            (0.71, 0.30),
            (0.74, 0.34),
            (0.74, 0.65),
            (0.72, 0.75),
            (0.65, 0.83),
            (0.63, 0.90),
        ];

        let mut points = HAND.to_vec();
        for _ in 0..2 {
            let n = points.len();
            let mut next = Vec::with_capacity(n * 2);
            for i in 0..n {
                let a = points[i];
                let b = points[(i + 1) % n];
                next.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
                next.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
            }
            points = next;
        }
        let mut distances = vec![0.];
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            distances.push(distances.last().unwrap() + (b.0 - a.0).hypot(b.1 - a.1));
        }
        let total = *distances.last().unwrap();
        let mut sampled = Vec::with_capacity(240);
        for i in 0..240 {
            let target = total * i as f32 / 240.;
            let upper = distances.partition_point(|d| *d <= target);
            let index = upper.saturating_sub(1).min(points.len() - 1);
            let part =
                (target - distances[index]) / (distances[index + 1] - distances[index]).max(1e-6);
            let a = points[index];
            let b = points[(index + 1) % points.len()];
            sampled.push((mix(a.0, b.0, part), mix(a.1, b.1, part)));
        }
        sampled
    });
    &SMOOTH
}
fn hand_point(u: f32, cord: usize, t: f32) -> (f32, f32) {
    let hand = weave_hand();
    let fraction = fract(u);
    let position = fraction * hand.len() as f32;
    let i = position.floor() as usize;
    let f = position - i as f32;
    let a = hand[i];
    let b = hand[(i + 1) % hand.len()];
    let scale = 0.94 + cord as f32 * 0.025;
    let wobble = (fraction * TAU * 4. + t * 3.8 + cord as f32).sin() * 0.003;
    (
        (mix(a.0, b.0, f) - 0.435) * scale * 1.8 + wobble,
        (mix(a.1, b.1, f) - 0.5) * scale * 1.8 + wobble * 0.6,
    )
}

#[derive(Clone, Copy)]
struct Vertex(f32, f32, f32);
impl Vertex {
    fn normalized(self) -> Self {
        let n = (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt();
        Self(self.0 / n, self.1 / n, self.2 / n)
    }
    fn project(self, t: f32) -> Self {
        let yaw = t * 1.9;
        let pitch = t * 0.83 + 0.4;
        let a = self.0 * yaw.cos() + self.2 * yaw.sin();
        let b = -self.0 * yaw.sin() + self.2 * yaw.cos();
        let ry = self.1 * pitch.cos() - b * pitch.sin();
        let rz = self.1 * pitch.sin() + b * pitch.cos();
        let k = 1. / (1.4 - rz * 0.22);
        Self(a * k, ry * k, rz)
    }
}
type Mesh = (Vec<Vertex>, Vec<(usize, usize)>);
fn weave_mesh() -> &'static Mesh {
    static MESH: std::sync::LazyLock<Mesh> = std::sync::LazyLock::new(|| {
        let phi = (1. + 5.0f32.sqrt()) / 2.;
        let mut vertices = Vec::new();
        for a in [-1., 1.] {
            for b in [-phi, phi] {
                vertices.extend([
                    Vertex(0., a, b).normalized(),
                    Vertex(a, b, 0.).normalized(),
                    Vertex(b, 0., a).normalized(),
                ]);
            }
        }
        let near = |a: usize, b: usize| {
            let u = vertices[a];
            let v = vertices[b];
            (u.0 - v.0).powi(2) + (u.1 - v.1).powi(2) + (u.2 - v.2).powi(2) < 1.11
        };
        let mut faces = Vec::new();
        for a in 0..12 {
            for b in a + 1..12 {
                for c in b + 1..12 {
                    if near(a, b) && near(a, c) && near(b, c) {
                        faces.push((a, b, c));
                    }
                }
            }
        }
        let mut midpoints = std::collections::BTreeMap::new();
        let mut edge_set = std::collections::BTreeSet::new();
        let mut edges = Vec::new();
        for (a, b, c) in faces {
            let mut midpoint = |a: usize, b: usize| {
                let key = (a.min(b), a.max(b));
                if let Some(&i) = midpoints.get(&key) {
                    return i;
                }
                let u = vertices[a];
                let v = vertices[b];
                let i = vertices.len();
                vertices.push(Vertex(u.0 + v.0, u.1 + v.1, u.2 + v.2).normalized());
                midpoints.insert(key, i);
                i
            };
            let ab = midpoint(a, b);
            let bc = midpoint(b, c);
            let ca = midpoint(c, a);
            for (u, v) in [
                (a, ab),
                (ab, b),
                (b, bc),
                (bc, c),
                (c, ca),
                (ca, a),
                (ab, bc),
                (bc, ca),
                (ca, ab),
            ] {
                let key = (u.min(v), u.max(v));
                if edge_set.insert(key) {
                    edges.push((u, v));
                }
            }
        }
        (vertices, edges)
    });
    &MESH
}
fn weave(out: &mut Scene, s: State) {
    let geo = s.listening() + s.thinking();
    let hand = if s.pose == Pose::Waiting { 1. } else { 0. };
    let mesh = weave_mesh();
    let projected: Vec<_> = mesh.0.iter().map(|p| p.project(s.t)).collect();
    let per_cord = mesh.1.len() as f32 / 5.;
    for (edge_index, &(a, b)) in mesh.1.iter().enumerate() {
        let cord = edge_index % 5;
        let section = edge_index / 5;
        let a = projected[a];
        let b = projected[b];
        let mut points = Vec::new();
        for i in 0..=8 {
            let fraction = i as f32 / 8.;
            let along = (section as f32 + fraction) / per_cord;
            let angle = TAU * along;
            let energy = s.voice();
            let stop = s.working();
            let exponent = 2. / (2. + 5. * stop);
            let ripple = energy
                * (0.19 * (angle * 3. + s.t * 3. + cord as f32 * 1.7).sin()
                    + 0.075 * (angle * 7. - s.t * 3.).sin());
            let (x, y) = (angle.cos(), angle.sin());
            let px = x.signum()
                * x.abs().powf(exponent)
                * (0.53 + cord as f32 * 0.055 + energy * 0.16)
                * (1. + ripple);
            let py = y.signum()
                * y.abs().powf(exponent)
                * (0.39 + energy * 0.33 + stop * 0.16)
                * (1. + ripple);
            let orbit = rotate(
                px,
                py,
                (cord as f32 * 34. + s.t * 9. + energy * (cord as f32 - 2.) * 28.) * PI / 180.,
            );
            // Listening selects the globe completely, so orbit deformation
            // cannot carry its microphone energy. Breathe the connected mesh
            // itself with real input, preserving the quiet/thinking geometry.
            let listening_scale = 1.14 * (1. + 0.18 * s.level * s.listening());
            let globe = (
                (a.0 + (b.0 - a.0) * fraction) * listening_scale,
                (a.1 + (b.1 - a.1) * fraction) * listening_scale,
            );
            let h = hand_point(along, cord, s.t);
            points.push((
                orbit.0 * (1. - geo - hand) + globe.0 * geo + h.0 * hand,
                orbit.1 * (1. - geo - hand) + globe.1 * geo + h.1 * hand,
            ));
        }
        let depth = (a.2 + b.2 + 2.) / 4.;
        let flow = 0.5 + 0.5 * (s.t * 5. - edge_index as f32 * 0.29).sin();
        let light =
            (0.35 + depth * 0.55) * (1. - hand) + (0.48 + flow * 0.48) * hand + 0.12 * s.band(cord);
        out.path(
            &points,
            false,
            0.0045 * (0.75 + depth * 0.5 + hand * 0.3),
            light,
            if s.pose == Pose::Error {
                Material::Eye
            } else if cord.is_multiple_of(2) {
                Material::Warm
            } else {
                Material::Dim
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn listening_weave_geometry_moves_with_microphone_energy() {
        let quiet = build(
            Style::IphoneWeave,
            Pose::Listening,
            0.,
            1.,
            Some([0.; 8]),
            (0., 0.),
        );
        let loud = build(
            Style::IphoneWeave,
            Pose::Listening,
            1.,
            1.,
            Some([1.; 8]),
            (0., 0.),
        );
        let radius = |scene: &Scene| {
            scene
                .strokes
                .iter()
                .flat_map(|line| [line.a, line.b])
                .map(|(x, y)| (x * x + y * y).sqrt())
                .fold(0_f32, f32::max)
        };
        assert!(radius(&loud) > radius(&quiet) * 1.15);
        assert!(radius(&loud) < 1.15);
        for pose in [Pose::Ready, Pose::Thinking] {
            let a = build(Style::IphoneWeave, pose, 0., 1., Some([0.; 8]), (0., 0.));
            let b = build(Style::IphoneWeave, pose, 1., 1., Some([1.; 8]), (0., 0.));
            assert_eq!(radius(&a), radius(&b));
        }
    }

    #[test]
    fn each_ring_follows_its_own_band_or_a_derived_spread() {
        let state = |pose, level, t, bands: Option<[f32; 8]>| State {
            pose,
            level,
            t,
            bands: bands.unwrap_or([level; 8]),
            measured: bands.is_some(),
            gaze: (0., 0.),
        };
        // Measured: the low bands drive only the first rings, the high bands only the last.
        let low = ring_drive(state(
            Pose::Speaking,
            0.6,
            2.,
            Some([1., 1., 0., 0., 0., 0., 0., 0.]),
        ));
        assert_eq!(low, [1., 0.5, 0., 0., 0., 0.]);
        let high = ring_drive(state(
            Pose::Listening,
            0.6,
            2.,
            Some([0., 0., 0., 0., 0., 0., 1., 1.]),
        ));
        assert_eq!(high, [0., 0., 0., 0., 0.5, 1.]);
        // Level only: a derived spread, bounded by the level, so the rings move one by one
        // (on most frames they differ; a shared pause now and then is speech-like).
        let spread = (0..200)
            .filter(|k| {
                let d = ring_drive(state(Pose::Speaking, 0.8, *k as f32 * 0.37, None));
                assert!(d.iter().all(|v| (0.0..=0.8).contains(v)), "{d:?}");
                d.iter().fold(0f32, |m, v| m.max(*v)) - d.iter().fold(1f32, |m, v| m.min(*v)) > 0.1
            })
            .count();
        assert!(
            spread > 160,
            "rings move together on {} of 200 frames",
            200 - spread
        );
        assert_ne!(
            ring_drive(state(Pose::Speaking, 0.8, 1., None)),
            ring_drive(state(Pose::Speaking, 0.8, 1.2, None))
        );
        assert_eq!(ring_drive(state(Pose::Speaking, 0., 1., None)), [0.; 6]);
        for pose in [Pose::Ready, Pose::Thinking, Pose::Working, Pose::Waiting] {
            assert_eq!(ring_drive(state(pose, 1., 1., Some([1.; 8]))), [0.; 6]);
        }
        // And the geometry answers: one band moves the picture differently from another.
        let ends = |bands| {
            build(
                Style::WatchRings,
                Pose::Speaking,
                0.6,
                2.,
                Some(bands),
                (0., 0.),
            )
            .strokes
            .iter()
            .map(|s| s.a)
            .collect::<Vec<_>>()
        };
        let mut only = [0.; 8];
        only[0] = 1.;
        let mut other = [0.; 8];
        other[7] = 1.;
        assert_ne!(ends(only), ends(other));
    }

    #[test]
    fn scenes_are_bounded_and_every_style_has_geometry() {
        for st in Style::ALL.into_iter().filter(|s| !s.analytic()) {
            let scene = build(st, Pose::Speaking, 0.6, 1.1, Some([0.6; 8]), (0., 0.));
            assert!(!scene.strokes.is_empty(), "{st:?}");
            assert!(
                scene.strokes.len() + scene.surfaces.len() < MAX_PRIMITIVES,
                "{st:?} exhausted its bounded primitive budget"
            );
            assert!(scene
                .cells
                .iter()
                .all(|c| c.iter().all(|&i| (i as usize) < scene.strokes.len())));
            assert!(scene
                .surface_cells
                .iter()
                .all(|c| c.iter().all(|&i| (i as usize) < scene.surfaces.len())));
        }
    }
    #[test]
    fn rendering_does_not_allocate_per_sample() {
        let scene = build(
            Style::WatchNebula,
            Pose::Thinking,
            0.6,
            1.,
            Some([0.; 8]),
            (0., 0.),
        );
        let n = scene.strokes.len();
        for _ in 0..10 {
            for x in 0..128 {
                let p = scene.sample(x as f32 / 64. - 1., 0.2);
                assert!(p.light.is_finite() && p.coverage.is_finite());
            }
        }
        assert_eq!(n, scene.strokes.len());
    }
    #[test]
    fn overlapping_materials_keep_both_color_channels() {
        let mut scene = Scene::new();
        scene.dot(0., 0., 0.03, 0.4, Material::Warm);
        scene.dot(0., 0., 0.03, 0.4, Material::Eye);
        let palette = [[0xff0000ff; 256], [0x00ff00ff; 256], [0x0000ffff; 256]];
        let mixed = scene.rgba(0., 0., 0.01, &palette);
        assert!(
            mixed[0] > 150 && mixed[1] > 150 && mixed[2] == 0,
            "{mixed:?}"
        );
        assert!(scene.sample(0., 0.).material != Material::Dim);
    }
    #[test]
    fn pixel_integration_reduces_subpixel_point_flicker() {
        let palette = [[0xffffffff; 256]; 3];
        let mut point_totals = Vec::new();
        let mut pixel_totals = Vec::new();
        for step in 0..8 {
            let mut scene = Scene::new();
            scene.dot(step as f32 * 0.0025, 0., 0.002, 0.8, Material::Warm);
            let mut point = 0.;
            let mut pixel = 0.;
            for x in -4..=4 {
                for y in -4..=4 {
                    point += scene.rgba(x as f32 * 0.02, y as f32 * 0.02, 0., &palette)[3] as f32;
                    pixel += scene.rgba(x as f32 * 0.02, y as f32 * 0.02, 0.02, &palette)[3] as f32;
                }
            }
            point_totals.push(point);
            pixel_totals.push(pixel);
        }
        let spread = |xs: &[f32]| {
            xs.iter().copied().fold(0., f32::max) - xs.iter().copied().fold(f32::INFINITY, f32::min)
        };
        assert!(spread(&pixel_totals) < spread(&point_totals) * 0.4);
    }
    #[test]
    fn focused_designs_retain_source_structure_and_opaque_hover_shell() {
        let wave = build(
            Style::WatchWave,
            Pose::Speaking,
            0.6,
            2.3,
            Some([0.6; 8]),
            (0., 0.),
        );
        assert!(
            wave.surfaces.len() > 500,
            "wave ribbons have translucent bodies"
        );
        let hover = build(
            Style::WatchHover,
            Pose::Ready,
            0.,
            2.3,
            Some([0.; 8]),
            (0., 0.),
        );
        assert!(
            hover.surfaces.len() > 15,
            "shell, visor, engines and eyes are surfaces"
        );
        let palette = [[0xffffffff; 256]; 3];
        assert_eq!(hover.rgba(0., 0., 0.01, &palette)[3], 255);
        let eye = build(
            Style::WatchEye,
            Pose::Speaking,
            0.6,
            2.3,
            Some([0.6; 8]),
            (0., 0.),
        );
        assert!(
            eye.strokes.len() > 900,
            "iris fibers, lids and brow remain separate"
        );
    }
}
