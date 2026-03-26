//! Package rustychroma provides high-performance chroma key
//! background removal and edge erosion for images.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

// chroma_cb and chroma_cr compute BT.601 chroma components (range ~16-240).
// Used for luminance independent keying so dark pixels always have neutral
// chroma and are never falsely pulled into the removal range.
#[inline(always)]
fn chroma(r: i32, g: i32, b: i32) -> (i32, i32) {
    let cb = ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128;
    let cr = ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128;
    (cb, cr)
}

/// **remove** modifies an RGBA buffer where pixels whose chroma is within
/// the given threshold of the key color are made fully transparent.
///
/// threshold is the squared Euclidean distance of BT.601 chroma components (dCb^2 + dCr^2).
pub fn remove(pixels: &mut [u8], kr: u8, kg: u8, kb: u8, threshold: f64) {
    let thresh = threshold as i32;
    let (key_cb, key_cr) = chroma(kr as i32, kg as i32, kb as i32);

    #[cfg(feature = "parallel")]
    let iter = pixels.par_chunks_exact_mut(4);
    #[cfg(not(feature = "parallel"))]
    let iter = pixels.chunks_exact_mut(4);

    iter.for_each(|px| {
        if px[3] == 0 {
            return;
        }
        let (cb, cr) = chroma(px[0] as i32, px[1] as i32, px[2] as i32);
        let dcb = cb - key_cb;
        let dcr = cr - key_cr;
        if dcb * dcb + dcr * dcr < thresh {
            px.fill(0);
        }
    });
}

/// **remove_range** modifies an RGBA buffer applying a soft chroma key.
/// Pixels within min_threshold of the key color become fully transparent,
/// and those beyond max_threshold remain unchanged.
/// Intermediate pixels will receive proportional
/// transparency and color spill suppression.
///
/// thresholds are the squared Euclidean distance of BT.601 chroma components (dCb^2 + dCr^2).
pub fn remove_range(
    pixels: &mut [u8],
    kr: u8,
    kg: u8,
    kb: u8,
    min_threshold: f64,
    max_threshold: f64,
) {
    let min_thresh = min_threshold as i32;
    let max_thresh = max_threshold as i32;
    let thresh_diff = (max_thresh - min_thresh).max(1);

    let recip: u64 = (1u64 << 32) / thresh_diff as u64;
    let inv_thresh_diff_f = 1.0_f32 / thresh_diff as f32;

    let (key_cb, key_cr) = chroma(kr as i32, kg as i32, kb as i32);
    let krf = kr as f32;
    let kgf = kg as f32;
    let kbf = kb as f32;

    #[cfg(feature = "parallel")]
    let iter = pixels.par_chunks_exact_mut(4);
    #[cfg(not(feature = "parallel"))]
    let iter = pixels.chunks_exact_mut(4);

    iter.for_each(|px| {
        if px[3] == 0 {
            return;
        }

        let (cb, cr) = chroma(px[0] as i32, px[1] as i32, px[2] as i32);
        let dcb = cb - key_cb;
        let dcr = cr - key_cr;
        let dist = dcb * dcb + dcr * dcr;

        if dist <= min_thresh {
            px.fill(0);
        } else if dist < max_thresh {
            let above_min = dist - min_thresh;
            let new_a = ((px[3] as u64 * above_min as u64 * recip) >> 32) as u8;
            let spill = 1.0_f32 - above_min as f32 * inv_thresh_diff_f;
            px[0] = (px[0] as f32 - spill * krf) as u8;
            px[1] = (px[1] as f32 - spill * kgf) as u8;
            px[2] = (px[2] as f32 - spill * kbf) as u8;
            px[3] = new_a;
        }
    });
}

/// **erode** removes 1 pixel of alpha by clearing any
/// opaque pixel that touches a fully transparent pixel.
pub fn erode(src: &[u8], dst: &mut [u8], width: usize, height: usize) {
    let stride = width * 4;
    debug_assert_eq!(src.len(), stride * height);
    debug_assert_eq!(dst.len(), stride * height);

    dst.fill(0);

    #[cfg(feature = "parallel")]
    let row_iter = dst.par_chunks_exact_mut(stride).enumerate();
    #[cfg(not(feature = "parallel"))]
    let row_iter = dst.chunks_exact_mut(stride).enumerate();

    row_iter.for_each(|(y, row_dst)| {
        let row = &src[y * stride..(y + 1) * stride];
        let prev = (y > 0).then(|| &src[(y - 1) * stride..y * stride]);
        let next = (y + 1 < height).then(|| &src[(y + 1) * stride..(y + 2) * stride]);

        for x in (0..stride).step_by(4) {
            if row[x + 3] == 0 {
                continue;
            }
            let is_edge = (x >= 4 && row[x - 1] == 0)
                || (x + 4 < stride && row[x + 7] == 0)
                || prev.is_some_and(|r| r[x + 3] == 0)
                || next.is_some_and(|r| r[x + 3] == 0);
            if !is_edge {
                row_dst[x..x + 4].copy_from_slice(&row[x..x + 4]);
            }
        }
    });
}

#[cfg(feature = "c-api")]
pub mod ffi {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn chromakey_remove(
        pixels: *mut u8,
        len: usize,
        kr: u8,
        kg: u8,
        kb: u8,
        threshold: f64,
    ) {
        let buf = unsafe { core::slice::from_raw_parts_mut(pixels, len) };
        crate::remove(buf, kr, kg, kb, threshold);
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn chromakey_remove_range(
        pixels: *mut u8,
        len: usize,
        kr: u8,
        kg: u8,
        kb: u8,
        min_threshold: f64,
        max_threshold: f64,
    ) {
        let buf = unsafe { core::slice::from_raw_parts_mut(pixels, len) };
        crate::remove_range(buf, kr, kg, kb, min_threshold, max_threshold);
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn chromakey_erode(
        src: *const u8,
        dst: *mut u8,
        width: usize,
        height: usize,
    ) {
        let len = width * height * 4;
        let src_buf = unsafe { core::slice::from_raw_parts(src, len) };
        let dst_buf = unsafe { core::slice::from_raw_parts_mut(dst, len) };
        crate::erode(src_buf, dst_buf, width, height);
    }
}

#[cfg(feature = "wasm")]
mod wasm_api {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn remove(pixels: &mut [u8], kr: u8, kg: u8, kb: u8, threshold: f64) {
        crate::remove(pixels, kr, kg, kb, threshold);
    }

    #[wasm_bindgen]
    pub fn remove_range(
        pixels: &mut [u8],
        kr: u8,
        kg: u8,
        kb: u8,
        min_threshold: f64,
        max_threshold: f64,
    ) {
        crate::remove_range(pixels, kr, kg, kb, min_threshold, max_threshold);
    }

    #[wasm_bindgen]
    pub fn erode(src: &[u8], dst: &mut [u8], width: usize, height: usize) {
        crate::erode(src, dst, width, height);
    }
}
