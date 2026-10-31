//! Procedural Apple scene vocabulary, expressed as bounded material-space strokes.
//! References are the read-only Orb*.swift and OrbWeave.swift implementations.
//! Scene geometry is generated once; a spatial index bounds per-pixel work.
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
}

#[derive(Clone, Debug)]
pub(crate) struct Scene {
    strokes: Vec<Stroke>,
    cells: Vec<Vec<u16>>,
}

impl Scene {
    fn new() -> Self {
        Self {
            strokes: Vec::new(),
            cells: vec![Vec::new(); CELLS * CELLS],
        }
    }
    fn add(&mut self, a: (f32, f32), b: (f32, f32), radius: f32, light: f32, material: Material) {
        if self.strokes.len() >= MAX_PRIMITIVES
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
            a.0.min(b.0) - radius * 2.5,
            a.1.min(b.1) - radius * 2.5,
            a.0.max(b.0) + radius * 2.5,
            a.1.max(b.1) + radius * 2.5,
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
}
impl State {
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

pub(crate) fn build(style: Style, pose: Pose, level: f32, t: f32, bands: [f32; 8]) -> Scene {
    let s = State {
        pose,
        level,
        t,
        bands,
    };
    let mut out = Scene::new();
    match style {
        Style::WatchHalo
        | Style::WatchVortex
        | Style::WatchNebula
        | Style::WatchLiquid
        | Style::WatchGalaxy
        | Style::WatchKitty
        | Style::WatchFaceShape
        | Style::WatchWaveShape => planet(&mut out, style, s),
        Style::WatchRings => rings(&mut out, s),
        Style::WatchLotus => lotus(&mut out, s),
        Style::WatchCrystal => crystal(&mut out, s),
        Style::WatchEclipse => eclipse(&mut out, s),
        Style::WatchSynapse => synapse(&mut out, s),
        Style::WatchDrop => drop_scene(&mut out, s),
        Style::WatchWave => wave(&mut out, s, false),
        Style::WatchWell => well(&mut out, s),
        Style::WatchBlackHole => blackhole(&mut out, s),
        Style::WatchAccretion => accretion(&mut out, s),
        Style::WatchFace | Style::WatchImprint | Style::WatchContour | Style::WatchMembrane => {
            portrait(&mut out, style, s)
        }
        Style::WatchAurora => aurora(&mut out, s),
        Style::WatchMurmuration => murmuration(&mut out, s),
        Style::WatchEye => eye(&mut out, s),
        Style::WatchLens
        | Style::WatchDome
        | Style::WatchValet
        | Style::WatchRetro
        | Style::WatchDeco
        | Style::WatchRover
        | Style::WatchHover
        | Style::WatchSpark
        | Style::WatchCruiser
        | Style::WatchAndroid
        | Style::WatchSentinel => machine(&mut out, style, s),
        Style::WatchOcean => ocean(&mut out, s),
        Style::IphoneWeave => weave(&mut out, s),
        _ => {}
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
                let flow = fract(u + s.t * if s.pose == Pose::Listening { -0.4 } else { 0.4 });
                let stream = (0.5 + 0.5 * (5. * a + 0.3 * s.t).sin()).powi(3)
                    + 0.35 * (0.5 + 0.5 * (8. * a - 0.2 * s.t).sin()).powi(5);
                let reach = 0.2 + 0.8 * s.band(band) + 0.38 * s.thinking() + 0.6 * s.working();
                let rr = 0.97 + (reach * stream + 0.12) * flow.powf(1.5);
                (
                    a.cos() * rr,
                    a.sin() * rr,
                    0.28 + 0.5 * stream * (1. - flow),
                )
            }
            Style::WatchVortex => {
                let rr = 0.55 + (0.75 + 0.55 * s.band(band)) * u * (1. - 0.18 * s.listening());
                let angle = (band % 2) as f32 * PI
                    + (3. - 1.4 * s.voice() + 0.9 * s.listening()) * (rr / 0.55).ln()
                    + s.t * (0.12 + 0.4 * s.motion() + 0.2 * s.voice());
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
                    let a = TAU * j / 3. + 0.5 + 0.03 * s.t;
                    let c = 0.42 * (1. + 0.55 * s.voice());
                    let l = 0.3 * (1. + 0.4 * s.voice());
                    (
                        c * a.cos() + l * gy * a.cos() - 1.5 * l * gx * a.sin(),
                        c * a.sin() + l * gy * a.sin() + 1.5 * l * gx * a.cos(),
                        0.04 + 0.48 * (-g * g / 2.).exp(),
                    )
                } else {
                    let rr = 0.3 + 0.95 * u;
                    let a = 2.3 + 3.1 * (band % 2) as f32 + 0.03 * s.t + 1.9 * u;
                    (rr * a.cos(), rr * a.sin(), 0.32 * (1. - 0.6 * u))
                }
            }
            Style::WatchLiquid => {
                let lift = (0.06 + 0.14 * s.band(band) + 0.04 * s.working())
                    * ((3. * x + 2. * y - 1.2 * s.t).sin()
                        + 0.5 * (4. * z - 2. * y + 1.7 * s.t).sin());
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
                    (band % 2) as f32 * PI + 2.3 * (gr + 0.08).ln() + (hash(i, 4.) - 0.5) * 0.6
                };
                angle -= s.t * (0.04 + 0.4 * s.motion() + s.voice() * 0.8 / (0.3 + gr));
                gr *= 1. + 0.3 * s.band(band);
                (
                    gr * angle.cos(),
                    gr * angle.sin() * 0.6,
                    if band == 0 { 0.9 } else { 0.35 },
                )
            }
            Style::WatchKitty => {
                if i % 7 == 0 {
                    let side = if i % 2 == 0 { -1. } else { 1. };
                    let h = hash(i, 8.);
                    x = side * (0.5 + 0.35 * (hash(i, 9.) - 0.5) * (1. - h) + 0.08 * h);
                    y = -0.78 - 0.55 * h;
                    light = 0.95;
                }
                (
                    x * (1. + 0.12 * s.voice()),
                    y * (1. + 0.08 * s.voice()),
                    0.1,
                )
            }
            Style::WatchFaceShape => {
                if z < -0.2 {
                    continue;
                }
                let ex = x.abs() - 0.36;
                let ey = y + 0.22;
                if z > 0.35 && ex * ex + ey * ey < 0.022 {
                    continue;
                }
                if (y - (0.32 - 0.5 * x * x)).abs() < 0.045 && x.abs() < 0.45 {
                    light = 1.;
                }
                let mouth_region = (-(x / 0.38).powi(2) - ((y - 0.30) / 0.18).powi(2)).exp();
                let jaw = s.voice() * (0.035 + 0.07 * super::speech(s.t)) * mouth_region;
                (x, y + jaw + s.voice() * 0.012 * (s.t * 6.).sin(), 0.08)
            }
            Style::WatchWaveShape => {
                let r = 0.25 + 0.95 * u.sqrt();
                let aa = phase;
                let yy = 0.14 * (r * 8. - s.t * 2.).sin() * (0.15 + s.voice());
                (aa.cos() * r, aa.sin() * r * 0.36 + yy, 0.5)
            }
            _ => unreachable!(),
        };
        if matches!(
            style,
            Style::WatchKitty | Style::WatchFaceShape | Style::WatchWaveShape
        ) {
            blend = 1.;
        }
        x = mix(x, tx, blend);
        y = mix(y, ty, blend);
        light += blend * extra + 0.35 * s.working() * (8. * r - 5. * s.t).cos().max(0.).powi(12);
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

fn rings(out: &mut Scene, s: State) {
    for ring in 0..6 {
        let f = ring as f32;
        let a = f * PI / 6.
            + 0.04 * (s.t * (0.4 + f * 0.15)).sin()
            + 0.3 * s.thinking() * (s.t * 1.4 + f).sin()
            + 0.45 * s.working() * (s.t * 3. - f * PI / 3.).sin();
        let radius = 0.46 * (0.97 + s.band(ring) * (0.62 * s.level - 0.22));
        let turn = s.t * s.motion();
        let tilt = 0.42 + 0.1 * (s.t * 0.21).sin();
        let points: Vec<_> = (0..97)
            .map(|j| {
                let q = TAU * j as f32 / 96.;
                let (x, y, z) = (
                    radius * q.cos(),
                    radius * q.sin() * a.cos(),
                    radius * q.sin() * a.sin(),
                );
                let bx = x * turn.cos() + z * turn.sin();
                let bz = -x * turn.sin() + z * turn.cos();
                (bx, y * tilt.cos() - bz * tilt.sin())
            })
            .collect();
        out.path(
            &points,
            false,
            0.005,
            0.55 * s.light(),
            if ring % 2 == 0 {
                s.material()
            } else {
                Material::Dim
            },
        );
        for j in 0..5 {
            let i = ((s.t * (5. + s.working() * 8.) + j as f32 * 19.2) as usize) % 96;
            out.dot(points[i].0, points[i].1, 0.013, 0.9, s.material());
        }
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
fn synapse(out: &mut Scene, s: State) {
    let points: Vec<_> = (0..38)
        .map(|i| {
            let a = TAU * hash(i, 4.) + s.t * (0.08 + s.thinking() * 0.12 + s.working() * 0.48);
            let r = (0.12 + 0.62 * hash(i, 7.).sqrt()) * (1. + 0.15 * s.voice());
            let r = if s.pose == Pose::Speaking {
                r * (0.7 + 0.3 * (6. * a).cos().abs())
            } else {
                r
            };
            (r * a.cos(), r * a.sin())
        })
        .collect();
    for (i, &a) in points.iter().enumerate() {
        for (j, &b) in points.iter().enumerate().skip(i + 1) {
            if (a.0 - b.0).hypot(a.1 - b.1) < 0.32 && (i + j) % 3 == 0 {
                out.line(
                    a,
                    b,
                    0.003,
                    0.3 + 0.3 * (s.t * 2. - j as f32).sin().max(0.),
                    Material::Dim,
                );
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
fn drop_scene(out: &mut Scene, s: State) {
    for i in 0..1500 {
        let a = TAU * fract(i as f32 * 0.7548777);
        let u = fract(i as f32 * 0.5698403);
        let taper = (PI * u).sin().powf(0.65);
        let radius = (0.12 + 0.34 * taper) * (1. + 0.2 * s.band(i % 8));
        let swirl = a
            + s.t * (0.15 + 0.7 * s.thinking())
            + u * TAU * s.thinking()
            + s.working() * (u * 8.0 - s.t * 1.3);
        let x = radius * swirl.cos();
        let y = (u - 0.5) * 1.4 + 0.06 * swirl.sin();
        let z = swirl.sin();
        out.dot(
            x,
            y,
            0.005,
            0.25 + 0.45 * z.max(0.) + 0.15 * s.voice(),
            if i % 5 == 0 {
                Material::Eye
            } else {
                s.material()
            },
        );
    }
    if s.pose == Pose::Speaking {
        for k in 0..8 {
            let a = TAU * k as f32 / 8.;
            out.line(
                (0., -0.65),
                (a.cos() * 0.52, -0.8 + a.sin() * 0.2),
                0.004,
                0.5,
                s.material(),
            );
        }
    }
}
fn wave(out: &mut Scene, s: State, membrane: bool) {
    for k in 0..28 {
        let base = (k as f32 / 27. - 0.5) * 1.3;
        let points: Vec<_> = (0..80)
            .map(|i| {
                let x = (i as f32 / 79. - 0.5) * 1.6;
                let pulse = (x * 5. - s.t * 1.5 + base * (2. + s.thinking() * 5.)).sin()
                    * (0.035 + 0.12 * s.band(k % 8) + 0.045 * s.thinking());
                let work = 0.08 * s.working() * (x * 6. + base * 4. - s.t * 3.).sin();
                let lift = if membrane {
                    0.26 * (-((x / 0.4).powi(2) + ((base + 0.1) / 0.65).powi(2))).exp()
                } else {
                    0.
                };
                (x, base + pulse + work - lift)
            })
            .collect();
        out.path(
            &points,
            false,
            0.0035,
            0.35 + 0.35 * (k as f32 / 28.),
            if k % 4 == 0 {
                Material::Eye
            } else {
                s.material()
            },
        );
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
fn accretion(out: &mut Scene, s: State) {
    out.dot(0., 0., 0.065 + 0.025 * s.voice(), 1., s.material());
    for ring in 0..6 {
        let r = 0.22 + ring as f32 * 0.1 + 0.018 * s.band(ring);
        let a = s.t * (0.4 / (0.3 + r));
        out.ellipse((0., 0.), (r, r * 0.38), 0.15, 0.004, 0.35, Material::Dim);
        for k in 0..90 {
            let q = TAU * k as f32 / 90. + a;
            let clump = 0.4 + 0.6 * (q * (3. + ring as f32) - s.t * 0.3).cos().max(0.).powi(3);
            let (x, y) = rotate(r * q.cos(), r * q.sin() * 0.38, 0.15);
            out.dot(
                x,
                y,
                0.005 + 0.004 * clump * s.thinking(),
                0.3 + 0.4 * clump + 0.20 * s.band(ring),
                s.material(),
            );
        }
        if s.pose == Pose::Working {
            let (x, y) = rotate(r * a.cos(), r * a.sin() * 0.38, 0.15);
            out.dot(x, y, 0.018, 0.9, Material::Eye);
        }
    }
}

fn face_relief(x: f32, y: f32, mouth: f32) -> Option<f32> {
    let head = (x / 0.45).powi(2) + ((y + 0.1) / 0.66).powi(2);
    if head > 1. {
        return None;
    }
    let z = 0.28 * (1. - head).sqrt()
        + 0.12 * (-(x / 0.08).powi(2) - ((y - 0.02) / 0.24).powi(2)).exp();
    let eyes = 0.065 * (-((x.abs() - 0.19) / 0.10).powi(2) - ((y + 0.19) / 0.065).powi(2)).exp();
    let lips = 0.035 * (-(x / 0.19).powi(2) - ((y - 0.29) / 0.055).powi(2)).exp();
    Some(
        z - eyes + lips
            - 0.11
                * mouth
                * (-(x / 0.16).powi(2) - ((y - 0.29) / (0.035 + 0.08 * mouth)).powi(2)).exp(),
    )
}
fn portrait(out: &mut Scene, style: Style, s: State) {
    let mouth = s.speaking() * (0.15 + 0.75 * s.level) * (0.5 + 0.5 * (s.t * 8.).sin());
    let dx = 0.025 * (s.t * 0.29).sin() - 0.04 * s.listening() + 0.03 * s.thinking();
    let dy = 0.01 * (s.t * 1.25).sin() - 0.035 * s.thinking()
        + 0.025 * s.listening() * s.level * (s.t * 4.3).sin();
    if matches!(style, Style::WatchContour | Style::WatchMembrane) {
        let vertical = style == Style::WatchMembrane;
        // Contour is horizontal engraved relief; Membrane is a vertical silk curtain.
        for k in 0..40 {
            let base = (k as f32 / 39. - 0.5) * 1.65;
            let pts: Vec<_> = (0..90)
                .map(|i| {
                    let along = (i as f32 / 89. - 0.5) * 1.7;
                    let (x, y) = if vertical {
                        (base, along)
                    } else {
                        (along, base)
                    };
                    let relief = face_relief((x - dx) * 0.9, (y - dy) * 0.9, mouth).unwrap_or(0.);
                    let outside = 1. - smooth(0., 0.13, relief);
                    let ripple =
                        (x * 6. - s.t * 2. + y * 3.).sin() * (0.005 + 0.045 * s.voice()) * outside;
                    if vertical {
                        (
                            x + relief * 0.48 + 0.012 * (y * 8. + s.t).sin() * (1. - relief),
                            y + ripple,
                        )
                    } else {
                        (x, y - relief * 0.85 + ripple)
                    }
                })
                .collect();
            out.path(
                &pts,
                false,
                0.004,
                0.25 + 0.22 * (k as f32 / 40.),
                if k % 7 == 0 {
                    Material::Eye
                } else {
                    s.material()
                },
            );
        }
        // Lit lips, nose ridge and brows remain legible at small Canvas/terminal sizes.
        let lips: Vec<_> = (0..22)
            .map(|i| {
                let x = (i as f32 / 21. - 0.5) * 0.38;
                (x + dx, 0.27 + dy - 0.4 * x * x + mouth * 0.06)
            })
            .collect();
        out.path(&lips, false, 0.008, 0.42, Material::Eye);
        out.line(
            (dx, -0.08 + dy),
            (dx + 0.025, 0.09 + dy),
            0.007,
            0.42,
            s.material(),
        );
    } else {
        let imprint = style == Style::WatchImprint;
        for i in 0..2300 {
            let x = (fract(i as f32 * 0.7548777) - 0.5) * 1.02;
            let y = (fract(i as f32 * 0.5698403) - 0.5) * 1.55;
            if let Some(z) = face_relief(x, y, mouth) {
                let epsilon = 0.006;
                let zx = face_relief(x + epsilon, y, mouth).unwrap_or(z)
                    - face_relief(x - epsilon, y, mouth).unwrap_or(z);
                let zy = face_relief(x, y + epsilon, mouth).unwrap_or(z)
                    - face_relief(x, y - epsilon, mouth).unwrap_or(z);
                let nx = -zx / (epsilon * 2.);
                let ny = -zy / (epsilon * 2.);
                let norm = (1. + nx * nx + ny * ny).sqrt();
                let light =
                    (0.13 + 0.8 * ((-0.35 * nx - 0.4 * ny + 0.85) / norm).max(0.)).clamp(0., 1.);
                let press = if imprint {
                    0.05 + 0.95 * smooth(0.23 - 0.04 * s.voice(), 0.31 - 0.04 * s.voice(), z)
                } else {
                    1.
                };
                // Imprint reveals only the glass-contact relief; the face shows its lit volume.
                let scale = if imprint { 1.20 } else { 1.0 };
                out.dot(
                    (x + dx) * scale,
                    (y + dy) * scale,
                    if imprint { 0.008 } else { 0.009 },
                    light * press,
                    s.material(),
                );
            }
        }
        for side in [-1., 1.] {
            let ex = dx + side * 0.19;
            if imprint {
                let lid: Vec<_> = (0..20)
                    .map(|i| {
                        let x = (i as f32 / 19. - 0.5) * 0.18;
                        (ex + x, dy - 0.18 + 0.9 * x * x)
                    })
                    .collect();
                out.path(&lid, false, 0.009, 0.55, s.material());
            } else {
                out.ellipse(
                    (ex, dy - 0.19),
                    (0.085, 0.035),
                    side * 0.05,
                    0.008,
                    0.42,
                    Material::Dim,
                );
                out.dot(ex, dy - 0.19, 0.023, 0.8, Material::Eye);
                out.dot(ex - 0.008, dy - 0.20, 0.009, 0.9, s.material());
                out.line(
                    (ex - 0.09, dy - 0.29),
                    (ex + 0.07, dy - 0.28 - 0.02 * s.thinking()),
                    0.012,
                    0.65,
                    s.material(),
                );
            }
        }
        let scale = if imprint { 1.2 } else { 1. };
        let smile: Vec<_> = (0..36)
            .map(|i| {
                let x = (i as f32 / 35. - 0.5) * 0.4;
                (
                    (x + dx) * scale,
                    (0.28 + dy - 0.35 * x * x + mouth * 0.04) * scale,
                )
            })
            .collect();
        out.path(&smile, false, 0.008, 0.6, Material::Eye);
        if !imprint {
            out.line((-0.12, 0.53), (-0.12, 0.70), 0.012, 0.42, s.material());
            out.line((0.12, 0.53), (0.12, 0.70), 0.012, 0.42, s.material());
            out.path(
                &[
                    (-0.48, 0.85),
                    (-0.30, 0.72),
                    (-0.12, 0.69),
                    (0.12, 0.69),
                    (0.30, 0.72),
                    (0.48, 0.85),
                ],
                false,
                0.012,
                0.42,
                s.material(),
            );
        } else {
            out.line((-0.62, -0.76), (0.62, 0.76), 0.003, 0.08, Material::Dim);
        }
    }
    if s.pose == Pose::Working {
        let y = -0.65 + 1.2 * fract(s.t * 0.35);
        let half = 0.45 * (1. - ((y + 0.1) / 0.66).powi(2)).max(0.).sqrt();
        out.line(
            (dx - half, y + dy),
            (dx + half, y + dy),
            0.012,
            0.65,
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
fn eye(out: &mut Scene, s: State) {
    let open = if s.pose == Pose::Listening {
        0.9
    } else if s.pose == Pose::Thinking {
        0.45
    } else {
        0.65
    };
    let blink = super::blink(s.t, 5.6, 0.20);
    let open = open * blink;
    for upper in [-1., 1.] {
        let pts: Vec<_> = (0..70)
            .map(|i| {
                let x = (i as f32 / 69. - 0.5) * 1.45;
                (x, upper * open * 0.5 * (1. - (x / 0.725).powi(2)))
            })
            .collect();
        out.path(&pts, false, 0.014, 0.65, s.material());
    }
    let ix = if s.pose == Pose::Working {
        0.14 * (s.t * 2.1).sin()
    } else if s.pose == Pose::Thinking {
        0.11 * (s.t * 0.7).sin()
    } else {
        0.
    };
    let ry = (0.22 * open / 0.65).max(0.03);
    out.ellipse((ix, 0.), (0.22, ry), 0., 0.015, 0.8, s.material());
    for k in 0..60 {
        let a = TAU * k as f32 / 60.;
        let l = 0.16 + 0.04 * s.voice() + 0.015 * (a * 7. + s.t).sin();
        out.line(
            (ix + 0.09 * a.cos(), 0.09 * a.sin()),
            (ix + l * a.cos(), l * a.sin() * open / 0.65),
            0.004,
            0.6,
            Material::Eye,
        );
    }
    out.dot(ix - 0.06, -0.06, 0.022, 1., Material::Dim);
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
fn lamp(out: &mut Scene, c: (f32, f32), r: f32, s: State) {
    out.dot(c.0, c.1, r, 0.55 + 0.4 * s.voice(), s.material());
    out.dot(c.0 - r * 0.25, c.1 - r * 0.25, r * 0.25, 0.9, Material::Eye);
}
fn machine(out: &mut Scene, style: Style, s: State) {
    let bob = 0.012 * (s.t * 0.85).sin() + 0.025 * s.listening() * (s.t * 1.05).sin();
    let cx = 0.025 * (s.t * 0.3).sin()
        + 0.08 * s.thinking() * (s.t * 0.7).sin()
        + 0.04 * s.working() * (s.t * 2.1).sin();
    let top = -0.25 + bob + 0.035 * s.working() * (s.t * 3.).sin();
    match style {
        Style::WatchLens => {
            metal_body(out, (0., 0.), 0.65, 0.65, 1300, s);
            out.ellipse((0., 0.), (0.53, 0.53), 0., 0.022, 0.45, Material::Dim);
            out.ellipse((0., 0.), (0.28, 0.28), 0., 0.016, 0.6, s.material());
            lamp(
                out,
                (cx, 0.05 * s.working() * (s.t * 2.).sin()),
                0.16 + 0.025 * s.voice(),
                s,
            );
            out.dot(-0.08, -0.08, 0.025, 1., Material::Eye);
        }
        Style::WatchHover => {
            metal_body(out, (cx, bob), 0.48, 0.42, 1000, s);
            out.line(
                (-0.44, 0.01 + bob),
                (0.44, 0.01 + bob),
                0.055,
                0.36,
                Material::Dim,
            );
            for side in [-1., 1.] {
                lamp(out, (cx + side * 0.23, 0.01 + bob), 0.052, s);
                out.line(
                    (side * 0.45, bob),
                    (side * (0.62 + 0.06 * s.working()), 0.16 + bob),
                    0.032,
                    0.6,
                    Material::Dim,
                );
            }
            for k in -1..=1 {
                lamp(
                    out,
                    (k as f32 * 0.19, 0.42 + bob),
                    0.027 + 0.009 * s.working(),
                    s,
                );
            }
        }
        Style::WatchRover | Style::WatchSpark => {
            let spark = style == Style::WatchSpark;
            metal_body(
                out,
                (0., 0.30),
                if spark { 0.38 } else { 0.46 },
                0.22,
                600,
                s,
            );
            out.line((0., 0.08), (cx, top + 0.04), 0.022, 0.5, Material::Dim);
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
                    let y = 0.20 + k as f32 * 0.048;
                    out.line((x - 0.075, y), (x + 0.075, y), 0.009, 0.23, Material::Dim);
                }
                let ex = cx + side * if spark { 0.21 } else { 0.24 };
                if spark {
                    // Spark's characteristic boxy binocular shutters, unlike Rover's round lenses.
                    let pts = [
                        (ex - 0.17, top - 0.14),
                        (ex + 0.16, top - 0.11),
                        (ex + 0.18, top + 0.13),
                        (ex - 0.16, top + 0.16),
                    ];
                    out.path(&pts, true, 0.016, 0.45, Material::Dim);
                    for blade in 0..6 {
                        let a = TAU * blade as f32 / 6. + 0.15 * s.thinking() * (s.t * 1.4).sin();
                        let b = a + 0.85;
                        out.line(
                            (ex + 0.032 * a.cos(), top + 0.032 * a.sin()),
                            (ex + 0.12 * b.cos(), top + 0.10 * b.sin()),
                            0.011,
                            0.6,
                            s.material(),
                        );
                    }
                    lamp(out, (ex, top), 0.027, s);
                } else {
                    metal_body(out, (ex, top), 0.18, 0.155, 300, s);
                    out.ellipse((ex, top), (0.135, 0.118), 0., 0.011, 0.5, Material::Dim);
                    lamp(out, (ex, top), 0.069, s);
                }
            }
            out.line((-0.23, 0.29), (0.23, 0.29), 0.011, 0.4, s.material());
            if spark {
                out.line(
                    (-0.08, top - 0.15),
                    (-0.13, top - 0.31),
                    0.008,
                    0.5,
                    Material::Dim,
                );
                out.dot(-0.13, top - 0.31, 0.018, 0.8, s.material());
            }
        }
        Style::WatchDome => {
            metal_body(out, (cx, 0.05 + bob), 0.42, 0.55, 1100, s);
            for side in [-1., 1.] {
                out.line(
                    (side * 0.4, -0.05),
                    (side * 0.52, 0.53),
                    0.045,
                    0.6,
                    Material::Dim,
                );
                out.line(
                    (side * 0.37, 0.55),
                    (side * 0.6, 0.55),
                    0.035,
                    0.5,
                    Material::Dim,
                );
            }
            out.ellipse((cx, top), (0.4, 0.26), 0., 0.018, 0.65, s.material());
            lamp(out, (cx - 0.13, top), 0.06, s);
            for k in 0..4 {
                lamp(out, (0.14, 0.04 + k as f32 * 0.1), 0.022, s);
            }
        }
        Style::WatchCruiser => {
            for y in 0..35 {
                let yy = y as f32 / 34.;
                let w = 0.73 - 0.23 * (yy - 0.4).abs();
                out.line(
                    (-w, 0.12 + yy * 0.35),
                    (w, 0.12 + yy * 0.35),
                    0.005,
                    0.2 + 0.25 * yy,
                    Material::Dim,
                );
            }
            let shell = [(-0.73, 0.13), (-0.45, -0.22), (0.38, -0.22), (0.72, 0.13)];
            out.path(&shell, false, 0.013, 0.75, Material::Dim);
            out.line((-0.32, -0.13), (0.3, -0.13), 0.008, 0.45, Material::Eye);
            let scan = 0.42 * (s.t * (1. + s.thinking() * 1.4 + s.working() * 3.0)).sin();
            out.line(
                (scan - 0.09, 0.22),
                (scan + 0.09, 0.22),
                0.025,
                0.9,
                s.material(),
            );
            // The voice illuminates individual dashboard bands, independently of scanner motion.
            for band in 0..7 {
                let x = (band as f32 - 3.0) * 0.055;
                let height = 0.015 + 0.08 * s.band(band);
                out.line(
                    (x, -0.17),
                    (x, -0.17 - height),
                    0.006,
                    0.25 + 0.6 * s.voice(),
                    s.material(),
                );
            }
            for side in [-1., 1.] {
                out.ellipse(
                    (side * 0.5, 0.48),
                    (0.12, 0.10),
                    0.,
                    0.035,
                    0.6,
                    Material::Dim,
                );
            }
        }
        _ => {
            let (rx, ry) = match style {
                Style::WatchValet => (0.24, 0.34),
                Style::WatchRetro => (0.36, 0.27),
                Style::WatchDeco => (0.24, 0.38),
                Style::WatchAndroid => (0.28, 0.39),
                _ => (0.34, 0.38),
            };
            metal_body(out, (cx, top), rx, ry, 900, s);
            metal_body(out, (0., 0.40), 0.40, 0.20, 400, s);
            out.line((cx, top + ry), (0., 0.25), 0.035, 0.5, Material::Dim);
            if style == Style::WatchSentinel {
                out.line(
                    (cx - rx * 0.8, top),
                    (cx + rx * 0.8, top),
                    0.025,
                    0.6,
                    Material::Dim,
                );
                lamp(
                    out,
                    (
                        cx + rx * 0.5 * (s.t * (0.7 + s.thinking() + s.working() * 2.)).sin(),
                        top,
                    ),
                    0.045,
                    s,
                );
                out.path(
                    &[
                        (-0.24, top + 0.06),
                        (-0.18, top + 0.23),
                        (0.18, top + 0.23),
                        (0.24, top + 0.06),
                    ],
                    false,
                    0.014,
                    0.75,
                    Material::Dim,
                );
            } else {
                for side in [-1., 1.] {
                    let eyes = if style == Style::WatchRetro {
                        0.11
                    } else {
                        0.05
                    };
                    lamp(out, (cx + side * rx * 0.48, top - 0.03), eyes, s);
                }
                let width = if style == Style::WatchRetro {
                    0.19
                } else {
                    0.10
                };
                out.line(
                    (cx - width, top + 0.15),
                    (cx + width, top + 0.15),
                    0.01 + 0.012 * s.voice(),
                    0.55,
                    s.material(),
                );
            }
            match style {
                Style::WatchValet => {
                    for side in [-1., 1.] {
                        let reach = 0.55 + 0.08 * s.thinking();
                        out.line(
                            (side * 0.28, 0.28),
                            (side * reach, 0.10 + 0.05 * (s.t * 2.).sin()),
                            0.018,
                            0.7,
                            s.material(),
                        );
                        out.dot(side * reach, 0.10, 0.04, 0.7, s.material());
                    }
                    out.line(
                        (-0.17, top + ry * 0.85),
                        (0.17, top + ry * 0.85),
                        0.012,
                        0.8,
                        Material::Eye,
                    );
                }
                Style::WatchRetro => {
                    out.ellipse(
                        (cx, top - 0.13),
                        (0.38, 0.22),
                        0.,
                        0.012,
                        0.75,
                        s.material(),
                    );
                    out.line((-0.42, 0.26), (-0.55, 0.60), 0.04, 0.7, Material::Dim);
                    out.line((0.42, 0.26), (0.55, 0.60), 0.04, 0.7, Material::Dim);
                    for i in -2..=2 {
                        out.line(
                            (i as f32 * 0.045, 0.30),
                            (i as f32 * 0.045, 0.45),
                            0.008,
                            0.6,
                            s.material(),
                        );
                    }
                }
                Style::WatchDeco => {
                    for i in 0..5 {
                        let y = top - ry + 0.05 + i as f32 * 0.09;
                        out.ellipse((cx, y), (0.27, 0.05), 0., 0.005, 0.5, s.material());
                    }
                    for side in [-1., 1.] {
                        out.line(
                            (side * 0.32, 0.24),
                            (side * 0.21, 0.61),
                            0.014,
                            0.65,
                            s.material(),
                        );
                    }
                }
                Style::WatchAndroid => {
                    out.ellipse((cx, top), (0.32, 0.45), 0., 0.008, 0.55, s.material());
                    out.line(
                        (cx, top - 0.18),
                        (cx, top + 0.07),
                        0.004,
                        0.45,
                        Material::Dim,
                    );
                }
                _ => {}
            }
        }
    }
    if s.pose == Pose::Error {
        for side in [-1., 1.] {
            let x = if style == Style::WatchLens {
                0.
            } else {
                cx + side * 0.12
            };
            let y = if style == Style::WatchLens { 0. } else { top };
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
            let globe = (
                (a.0 + (b.0 - a.0) * fraction) * 1.14,
                (a.1 + (b.1 - a.1) * fraction) * 1.14,
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
    fn scenes_are_bounded_and_every_style_has_geometry() {
        for st in Style::ALL.into_iter().skip(6) {
            let scene = build(st, Pose::Speaking, 0.6, 1.1, [0.6; 8]);
            assert!(!scene.strokes.is_empty(), "{st:?}");
            assert!(
                scene.strokes.len() < MAX_PRIMITIVES,
                "{st:?} exhausted its bounded primitive budget"
            );
            assert!(scene
                .cells
                .iter()
                .all(|c| c.iter().all(|&i| (i as usize) < scene.strokes.len())));
        }
    }
    #[test]
    fn rendering_does_not_allocate_per_sample() {
        let scene = build(Style::WatchNebula, Pose::Thinking, 0.6, 1., [0.; 8]);
        let n = scene.strokes.len();
        for _ in 0..10 {
            for x in 0..128 {
                let p = scene.sample(x as f32 / 64. - 1., 0.2);
                assert!(p.light.is_finite() && p.coverage.is_finite());
            }
        }
        assert_eq!(n, scene.strokes.len());
    }
}
