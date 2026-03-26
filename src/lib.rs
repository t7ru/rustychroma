#![allow(non_snake_case)]

//! Package rustychroma provides high-performance chroma key
//! background removal and edge erosion for images.

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// chromaCb and chromaCr compute BT.601 chroma components (range ~16-240).
// Used for luminance independent keying so dark pixels always have neutral
// chroma and are never falsely pulled into the removal range.
#[inline(always)]
fn chromaCb(r: i32, g: i32, b: i32) -> i32 {
    ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128
}

#[inline(always)]
fn chromaCr(r: i32, g: i32, b: i32) -> i32 {
    ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128
}

/// **remove** modifies an RGBA buffer where pixels whose chroma is within
/// the given threshold of the key color are made fully transparent.
///
/// threshold is the squared Euclidean distance of BT.601 chroma components (dCb^2 + dCr^2).
pub fn remove(pixels: &mut [u8], kr: u8, kg: u8, kb: u8, threshold: f64) {
    let thresh = threshold as i32;
    let keyCb = chromaCb(kr as i32, kg as i32, kb as i32);
    let keyCr = chromaCr(kr as i32, kg as i32, kb as i32);

    #[cfg(feature = "parallel")]
    let iter = pixels.par_chunks_exact_mut(4);
    #[cfg(not(feature = "parallel"))]
    let iter = pixels.chunks_exact_mut(4);

    iter.for_each(|px| {
        if px[3] == 0 {
            return;
        }
        let dcb = chromaCb(px[0] as i32, px[1] as i32, px[2] as i32) - keyCb;
        let dcr = chromaCr(px[0] as i32, px[1] as i32, px[2] as i32) - keyCr;
        if dcb * dcb + dcr * dcr < thresh {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
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
    let minThresh = min_threshold as i32;
    let maxThresh = max_threshold as i32;
    let threshDiff = (maxThresh - minThresh).max(1);

    let recip: u64 = (1u64 << 32) / threshDiff as u64;
    let invThreshDiffF = 1.0_f32 / threshDiff as f32;

    let keyCb = chromaCb(kr as i32, kg as i32, kb as i32);
    let keyCr = chromaCr(kr as i32, kg as i32, kb as i32);
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
        let r = px[0];
        let g = px[1];
        let b = px[2];
        let a = px[3];

        let dcb = chromaCb(r as i32, g as i32, b as i32) - keyCb;
        let dcr = chromaCr(r as i32, g as i32, b as i32) - keyCr;
        let dist = dcb * dcb + dcr * dcr;

        if dist <= minThresh {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
        } else if dist < maxThresh {
            let ratioNum = (dist - minThresh) as u64;
            let newA = ((a as u64 * ratioNum * recip) >> 32) as u8;
            let spill = 1.0_f32 - (dist - minThresh) as f32 * invThreshDiffF;
            px[0] = (r as f32 - spill * krf) as u8;
            px[1] = (g as f32 - spill * kgf) as u8;
            px[2] = (b as f32 - spill * kbf) as u8;
            px[3] = newA;
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
    let rowIter = dst.par_chunks_exact_mut(stride).enumerate();
    #[cfg(not(feature = "parallel"))]
    let rowIter = dst.chunks_exact_mut(stride).enumerate();

    rowIter.for_each(|(y, rowDst)| {
        let off = y * stride;

        for x in (0..stride).step_by(4) {
            if src[off + x + 3] == 0 {
                continue;
            }

            let isEdge = (x >= 4 && src[off + x - 1] == 0)
                || (x + 4 < stride && src[off + x + 7] == 0)
                || (y > 0 && src[off - stride + x + 3] == 0)
                || (y + 1 < height && src[off + stride + x + 3] == 0);

            if !isEdge {
                rowDst[x] = src[off + x];
                rowDst[x + 1] = src[off + x + 1];
                rowDst[x + 2] = src[off + x + 2];
                rowDst[x + 3] = src[off + x + 3];
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
