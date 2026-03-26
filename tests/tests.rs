use rustychroma::{erode, remove, remove_range};
use std::fs;

const fn rgba(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

fn alpha_at(buf: &[u8], width: usize, x: usize, y: usize) -> u8 {
    buf[(y * width + x) * 4 + 3]
}

fn make_3x3_green_red() -> Vec<u8> {
    let green = rgba(0, 255, 0);
    let red = rgba(255, 0, 0);
    let mut buf = vec![0u8; 3 * 3 * 4];
    for y in 0..3usize {
        for x in 0..3usize {
            let off = (y * 3 + x) * 4;
            buf[off..off + 4].copy_from_slice(if x == 1 && y == 1 { &red } else { &green });
        }
    }
    buf
}

fn make_3x3_range() -> Vec<u8> {
    let bg = rgba(0, 255, 0);
    let edge = rgba(100, 200, 0);
    let fg = rgba(255, 0, 0);
    let mut buf = vec![0u8; 3 * 3 * 4];
    for off in (0..buf.len()).step_by(4) {
        buf[off..off + 4].copy_from_slice(&bg);
    }
    let e = (1 * 3 + 1) * 4;
    buf[e..e + 4].copy_from_slice(&edge);
    let f = (2 * 3 + 2) * 4;
    buf[f..f + 4].copy_from_slice(&fg);
    buf
}

fn make_5x5_centre_square() -> Vec<u8> {
    let mut buf = vec![0u8; 5 * 5 * 4];
    for y in 1..=3usize {
        for x in 1..=3usize {
            let off = (y * 5 + x) * 4;
            buf[off..off + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    buf
}

fn load_png_rgba(path: &str) -> (Vec<u8>, usize, usize) {
    let data = fs::read(path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let decoder = png::Decoder::new(std::io::Cursor::new(data));
    let mut reader = decoder.read_info().unwrap();
    let mut raw = vec![
        0u8;
        reader
            .output_buffer_size()
            .expect("Failed to get PNG buffer size")
    ];
    let info = reader.next_frame(&mut raw).unwrap();
    let pixels = match info.color_type {
        png::ColorType::Rgba => raw[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => raw[..info.buffer_size()]
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        ct => panic!("unsupported color type in test.png: {ct:?}"),
    };
    (pixels, info.width as usize, info.height as usize)
}

// remove
#[test]
fn test_remove_preserves_non_key_pixel() {
    let mut buf = make_3x3_green_red();
    remove(&mut buf, 0, 255, 0, 7000.0);
    assert_ne!(
        alpha_at(&buf, 3, 1, 1),
        0,
        "centre (red) pixel should remain opaque"
    );
}

#[test]
fn test_remove_erases_key_pixel() {
    let mut buf = make_3x3_green_red();
    remove(&mut buf, 0, 255, 0, 7000.0);
    assert_eq!(
        alpha_at(&buf, 3, 0, 0),
        0,
        "corner (green) pixel should be removed"
    );
}

#[test]
fn test_remove_skips_already_transparent() {
    let mut buf = vec![0u8, 255, 0, 0];
    remove(&mut buf, 0, 255, 0, 7000.0);
    assert_eq!(&buf, &[0, 255, 0, 0]);
}

#[test]
#[ignore = "requires test.png"]
fn test_remove_file() {
    let (mut pixels, w, h) = load_png_rgba("tests/test.png");
    let original_len = pixels.len();
    remove(&mut pixels, 0xDF, 0x03, 0xDF, 7000.0);
    assert_eq!(pixels.len(), original_len);
    assert_eq!(pixels.len(), w * h * 4);
}

// remove_range
#[test]
fn test_remove_range_background_transparent() {
    let mut buf = make_3x3_range();
    remove_range(&mut buf, 0, 255, 0, 1000.0, 7000.0);
    assert_eq!(
        alpha_at(&buf, 3, 0, 0),
        0,
        "background pixel should be fully transparent"
    );
}

#[test]
fn test_remove_range_foreground_opaque() {
    let mut buf = make_3x3_range();
    remove_range(&mut buf, 0, 255, 0, 1000.0, 7000.0);
    assert_eq!(
        alpha_at(&buf, 3, 2, 2),
        255,
        "foreground pixel should be fully opaque"
    );
}

#[test]
fn test_remove_range_edge_semi_transparent() {
    let mut buf = make_3x3_range();
    remove_range(&mut buf, 0, 255, 0, 1000.0, 7000.0);
    let a = alpha_at(&buf, 3, 1, 1);
    assert!(
        a > 0 && a < 255,
        "edge pixel should be semi-transparent, got alpha={a}"
    );
}

#[test]
#[ignore = "requires test.png"]
fn test_remove_range_file() {
    let (mut pixels, w, h) = load_png_rgba("tests/test.png");
    let original_len = pixels.len();
    remove_range(&mut pixels, 0xDF, 0x03, 0xDF, 1000.0, 7000.0);
    assert_eq!(pixels.len(), original_len);
    assert_eq!(pixels.len(), w * h * 4);
}

// erode
#[test]
fn test_erode_centre_survives() {
    let src = make_5x5_centre_square();
    let mut dst = vec![0u8; src.len()];
    erode(&src, &mut dst, 5, 5);
    assert_ne!(
        alpha_at(&dst, 5, 2, 2),
        0,
        "absolute centre pixel should remain opaque"
    );
}

#[test]
fn test_erode_edge_removed() {
    let src = make_5x5_centre_square();
    let mut dst = vec![0u8; src.len()];
    erode(&src, &mut dst, 5, 5);
    assert_eq!(
        alpha_at(&dst, 5, 1, 1),
        0,
        "corner of opaque square should be eroded"
    );
}

#[test]
fn test_erode_output_zeroed_for_transparent_src() {
    let src = vec![0u8; 4 * 4 * 4];
    let mut dst = vec![0xFFu8; src.len()];
    erode(&src, &mut dst, 4, 4);
    assert!(
        dst.iter().all(|&b| b == 0),
        "dst should be fully zeroed when src is transparent"
    );
}
