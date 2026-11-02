//! Optional C/WebAssembly adapter. Handles own bounded, reusable pixel buffers.
//! Borrowed pixel pointers remain valid until destroy; callers must copy before
//! another render/destroy and synchronize native cross-thread pointer reads.
use super::*;
use std::{
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};
const LIMIT: u32 = 512;
const MAX_LIVE: usize = 16;
struct Frame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    palette: [[u32; 256]; 3],
    set: [[bool; 256]; 3],
    ready: usize,
    ground: u32,
    gaze: Gaze,
    bands: Option<[f32; 8]>,
}
struct Registry {
    next: u32,
    frames: BTreeMap<u32, Frame>,
}
static FRAMES: LazyLock<Mutex<Registry>> = LazyLock::new(|| {
    Mutex::new(Registry {
        next: 1,
        frames: BTreeMap::new(),
    })
});
#[unsafe(no_mangle)]
pub extern "C" fn avatar_abi_version() -> u32 {
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_style_count() -> u32 {
    AvatarStyle::ALL.len() as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_style_field_ptr(style: u32, field: u32) -> *const u8 {
    AvatarStyle::from_code(style)
        .and_then(|s| s.field(field))
        .map_or(std::ptr::null(), |s| s.as_ptr())
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_style_field_len(style: u32, field: u32) -> usize {
    AvatarStyle::from_code(style)
        .and_then(|s| s.field(field))
        .map_or(0, |s| s.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_create(width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 || width > LIMIT || height > LIMIT {
        return 0;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return 0;
    };
    if r.frames.len() >= MAX_LIVE || r.next == u32::MAX {
        return 0;
    }
    let id = r.next;
    r.next += 1;
    r.frames.insert(
        id,
        Frame {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
            palette: [[0; 256]; 3],
            set: [[false; 256]; 3],
            ready: 0,
            ground: 0,
            gaze: Gaze::default(),
            bands: None,
        },
    );
    id
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_set_shade(handle: u32, material: u32, index: u32, rgba: u32) -> i32 {
    if material >= 3 || index >= 256 {
        return -2;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    let (m, i) = (material as usize, index as usize);
    f.palette[m][i] = rgba;
    if !f.set[m][i] {
        f.set[m][i] = true;
        f.ready += 1;
    }
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_set_ground(handle: u32, rgba: u32) -> i32 {
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    f.ground = rgba;
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_set_gaze(handle: u32, x: f32, y: f32) -> i32 {
    if !x.is_finite() || !y.is_finite() || !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&y)
    {
        return -4;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    f.gaze = Gaze::new(x, y);
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_render(
    handle: u32,
    style: u32,
    pose: u32,
    level: f32,
    time: f32,
    transparent: u32,
) -> i32 {
    let Some(style) = AvatarStyle::from_code(style) else {
        return -2;
    };
    let Some(pose) = AvatarPose::from_code(pose) else {
        return -3;
    };
    if !level.is_finite()
        || !(0.0..=1.0).contains(&level)
        || !time.is_finite()
        || !(0.0..=86400.0).contains(&time)
        || transparent > 1
    {
        return -4;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    draw(f, style, pose, level, time, transparent)
}
fn draw(
    f: &mut Frame,
    style: AvatarStyle,
    pose: AvatarPose,
    level: f32,
    time: f32,
    transparent: u32,
) -> i32 {
    if f.ready != 768 {
        return -5;
    }
    let radius = f.width.min(f.height) as f32 * 0.5;
    let ground = f.ground.to_be_bytes();
    let frame = super::Frame::with_bands(
        style,
        pose,
        level,
        time,
        f.gaze,
        f.bands.unwrap_or([level; 8]),
    );
    for y in 0..f.height {
        for x in 0..f.width {
            let ink = frame.rgba(
                (x as f32 + 0.5 - f.width as f32 * 0.5) / radius,
                (y as f32 + 0.5 - f.height as f32 * 0.5) / radius,
                (1.0 / radius).min(0.125),
                &f.palette,
            );
            let a = ink[3] as f32 / 255.;
            let offset = ((y * f.width + x) * 4) as usize;
            let dest_alpha = ground[3] as f32 / 255.0;
            let out_alpha = a + dest_alpha * (1.0 - a);
            for c in 0..3 {
                f.pixels[offset + c] = if transparent == 1 {
                    ink[c]
                } else if out_alpha > 0.0 {
                    ((ink[c] as f32 * a + ground[c] as f32 * dest_alpha * (1.0 - a)) / out_alpha)
                        .round() as u8
                } else {
                    0
                };
            }
            f.pixels[offset + 3] =
                ((if transparent == 1 { a } else { out_alpha }) * 255.0).round() as u8;
        }
    }
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_pixels(handle: u32) -> *const u8 {
    let Ok(r) = FRAMES.lock() else {
        return std::ptr::null();
    };
    r.frames
        .get(&handle)
        .map_or(std::ptr::null(), |f| f.pixels.as_ptr())
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_pixels_len(handle: u32) -> usize {
    let Ok(r) = FRAMES.lock() else {
        return 0;
    };
    r.frames.get(&handle).map_or(0, |f| f.pixels.len())
}
#[unsafe(no_mangle)]
pub extern "C" fn avatar_destroy(handle: u32) -> i32 {
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    if r.frames.remove(&handle).is_some() {
        0
    } else {
        -1
    }
}
/// Set the eight host-measured normalized audio bands; all other states ignore them.
#[unsafe(no_mangle)]
pub extern "C" fn avatar_set_bands(
    handle: u32,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    b7: f32,
) -> i32 {
    let bands = [b0, b1, b2, b3, b4, b5, b6, b7];
    if bands
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return -4;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    f.bands = Some(bands);
    0
}
/// Restore scalar-level band defaults for hosts without spectral input.
#[unsafe(no_mangle)]
pub extern "C" fn avatar_clear_bands(handle: u32) -> i32 {
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    f.bands = None;
    0
}
/// Render and copy into caller-owned memory atomically with respect to native handles.
/// # Safety
/// `output` must point to at least `capacity` writable bytes belonging to the caller.
/// The allocation must not alias a borrowed renderer buffer or another active call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn avatar_render_copy(
    handle: u32,
    style: u32,
    pose: u32,
    level: f32,
    time: f32,
    transparent: u32,
    output: *mut u8,
    capacity: usize,
) -> i32 {
    let Some(style) = AvatarStyle::from_code(style) else {
        return -2;
    };
    let Some(pose) = AvatarPose::from_code(pose) else {
        return -3;
    };
    if !level.is_finite()
        || !(0.0..=1.0).contains(&level)
        || !time.is_finite()
        || !(0.0..=86400.0).contains(&time)
        || transparent > 1
    {
        return -4;
    }
    let Ok(mut r) = FRAMES.lock() else {
        return -1;
    };
    let Some(f) = r.frames.get_mut(&handle) else {
        return -1;
    };
    if output.is_null() || capacity < f.pixels.len() {
        return -6;
    }
    let status = draw(f, style, pose, level, time, transparent);
    if status != 0 {
        return status;
    }
    // SAFETY: caller guarantees a writable, non-overlapping destination of capacity bytes;
    // the above check bounds this copy and the registry lock keeps the source alive.
    unsafe {
        std::ptr::copy_nonoverlapping(f.pixels.as_ptr(), output, f.pixels.len());
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_invalid_handles_and_disposal_are_explicit() {
        assert_eq!(avatar_create(0, 32), 0);
        assert_eq!(avatar_create(513, 32), 0);
        let h = avatar_create(32, 32);
        assert_ne!(h, 0);
        assert_eq!(avatar_pixels_len(h), 4096);
        assert_eq!(avatar_render(h, 0, 0, 0.5, 0., 1), -5);
        for m in 0..3 {
            for i in 0..256 {
                assert_eq!(
                    avatar_set_shade(h, m, i, (i << 24) | (i << 16) | (i << 8) | 255),
                    0
                );
            }
        }
        assert_eq!(avatar_render(h, 0, 0, 0.5, 0., 1), 0);
        let mut owned = vec![0u8; 4096];
        // SAFETY: null is explicitly rejected before accessing it.
        assert_eq!(
            unsafe { avatar_render_copy(h, 0, 0, 0.5, 0., 1, std::ptr::null_mut(), 4096) },
            -6
        );
        // SAFETY: the supplied allocation is valid, and the insufficient capacity is rejected.
        assert_eq!(
            unsafe { avatar_render_copy(h, 0, 0, 0.5, 0., 1, owned.as_mut_ptr(), 4095) },
            -6
        );
        // SAFETY: owned is disjoint caller memory of the required length.
        assert_eq!(
            unsafe { avatar_render_copy(h, 0, 0, 0.5, 0., 1, owned.as_mut_ptr(), owned.len()) },
            0
        );
        assert_eq!(owned, FRAMES.lock().unwrap().frames[&h].pixels);
        assert_eq!(avatar_set_gaze(h, 2., 0.), -4);
        assert_eq!(
            avatar_set_bands(h, 0., 0., 0., f32::NAN, 0., 0., 0., 0.),
            -4
        );
        assert_eq!(avatar_render(h, 0, 0, 0.5, 86401., 1), -4);
        assert_eq!(avatar_render(h, 99, 0, 0., 0., 1), -2);
        assert_eq!(avatar_render(h, 0, 99, 0., 0., 1), -3);
        assert_eq!(avatar_render(h, 0, 0, f32::NAN, 0., 1), -4);
        assert_eq!(avatar_set_ground(h, 0x123456ff), 0);
        assert_eq!(avatar_render(h, 0, 0, 0.5, 0., 0), 0);
        assert_eq!(
            &FRAMES.lock().unwrap().frames[&h].pixels[..4],
            &[0x12, 0x34, 0x56, 0xff]
        );
        assert_eq!(avatar_destroy(h), 0);
        assert_eq!(avatar_destroy(h), -1);
        assert_eq!(avatar_pixels_len(h), 0);
        assert!(avatar_pixels(h).is_null());
        // SAFETY: owned remains a valid allocation; the destroyed handle is rejected.
        assert_eq!(
            unsafe { avatar_render_copy(h, 0, 0, 0.5, 0., 1, owned.as_mut_ptr(), owned.len()) },
            -1
        );
    }
}
