use divan::Bencher;
use rustychroma::{erode, remove, remove_range};

fn main() {
    divan::main();
}

const W_1080P: usize = 1920;
const H_1080P: usize = 1080;
const W_1K: usize = 1024;
const H_1K: usize = 1024;

fn green_1080p() -> Vec<u8> {
    let mut buf = vec![0u8; W_1080P * H_1080P * 4];
    for px in buf.chunks_exact_mut(4) {
        px[1] = 255;
        px[3] = 255;
    }
    buf
}

fn make_test_rgba(w: usize, h: usize, seed: u64) -> Vec<u8> {
    let mut buf = vec![0u8; w * h * 4];
    let mut s = seed;
    for px in buf.chunks_exact_mut(4) {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if s % 10 == 0 {
            px.copy_from_slice(&[0xDF, 0x03, 0xDF, 0xFF]);
        } else {
            px[0] = (s >> 16) as u8;
            px[1] = (s >> 24) as u8;
            px[2] = (s >> 32) as u8;
            px[3] = 0xFF;
        }
    }
    buf
}

#[divan::bench]
fn remove_1080p(bencher: Bencher) {
    let src = green_1080p();
    bencher.with_inputs(|| src.clone()).bench_values(|mut buf| {
        remove(&mut buf, 0, 255, 0, 7000.0);
        buf
    });
}

#[divan::bench]
fn remove_1024x1024(bencher: Bencher) {
    let src = make_test_rgba(W_1K, H_1K, 12345);
    bencher.with_inputs(|| src.clone()).bench_values(|mut buf| {
        remove(&mut buf, 0xDF, 0x03, 0xDF, 7000.0);
        buf
    });
}

#[divan::bench]
fn remove_range_1080p(bencher: Bencher) {
    let src = green_1080p();
    bencher.with_inputs(|| src.clone()).bench_values(|mut buf| {
        remove_range(&mut buf, 0, 255, 0, 1000.0, 7000.0);
        buf
    });
}

#[divan::bench]
fn remove_range_1024x1024(bencher: Bencher) {
    let src = make_test_rgba(W_1K, H_1K, 12345);
    bencher.with_inputs(|| src.clone()).bench_values(|mut buf| {
        remove_range(&mut buf, 0xDF, 0x03, 0xDF, 1000.0, 7000.0);
        buf
    });
}

#[divan::bench]
fn erode_1080p(bencher: Bencher) {
    let src = vec![255u8; W_1080P * H_1080P * 4];
    bencher
        .with_inputs(|| vec![0u8; src.len()])
        .bench_values(|mut dst| {
            erode(&src, &mut dst, W_1080P, H_1080P);
            dst
        });
}

#[divan::bench]
fn erode_1024x1024(bencher: Bencher) {
    let mut src = make_test_rgba(W_1K, H_1K, 42);
    for y in 0..256usize {
        for x in 0..256usize {
            src[(y * W_1K + x) * 4 + 3] = 0;
        }
    }
    bencher
        .with_inputs(|| vec![0u8; src.len()])
        .bench_values(|mut dst| {
            erode(&src, &mut dst, W_1K, H_1K);
            dst
        });
}
