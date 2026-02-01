// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use vello_bench_macros::vello_bench;
use vello_common::coarse::WideTile;
use vello_common::encode::EncodeExt;
use vello_common::fearless_simd::Simd;
use vello_common::kurbo::{Affine, Point};
use vello_common::paint::{Image, ImageSource};
use vello_common::peniko;
use vello_common::peniko::{BlendMode, ImageQuality, ImageSampler};
use vello_common::pixmap::Pixmap;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, FineKernel};

fn get_colr_image(extend: peniko::Extend, quality: ImageQuality) -> Image {
    let data = include_bytes!("../../assets/big_colr.png");

    let pixmap = Pixmap::from_png(&data[..]).unwrap();
    Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler {
            x_extend: extend,
            y_extend: extend,
            quality,
            alpha: 1.0,
        },
    }
}

fn get_small_image(extend: peniko::Extend, quality: ImageQuality) -> Image {
    let data = include_bytes!("../../assets/rgb_image_2x2.png");

    let pixmap = Pixmap::from_png(&data[..]).unwrap();
    Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler {
            x_extend: extend,
            y_extend: extend,
            quality,
            alpha: 1.0,
        },
    }
}

#[vello_bench]
fn image_no_transform<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_colr_image(peniko::Extend::Pad, ImageQuality::Low);
    let mut paints = vec![];
    let paint = im.encode_into(&mut paints, Affine::IDENTITY);

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn image_scale<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_colr_image(peniko::Extend::Pad, ImageQuality::Low);
    let mut paints = vec![];
    let paint = im.encode_into(&mut paints, Affine::scale(3.0));

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn image_rotate<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_colr_image(peniko::Extend::Pad, ImageQuality::Low);
    let mut paints = vec![];
    let paint = im.encode_into(
        &mut paints,
        Affine::rotate_about(
            1.0,
            Point::new(WideTile::WIDTH as f64 / 2.0, Tile::HEIGHT as f64 / 2.0),
        ),
    );

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn image_quality_medium<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_colr_image(peniko::Extend::Pad, ImageQuality::Medium);
    let mut paints = vec![];
    let paint = im.encode_into(&mut paints, Affine::scale(3.0));

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn image_quality_high<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_colr_image(peniko::Extend::Pad, ImageQuality::High);
    let mut paints = vec![];
    let paint = im.encode_into(&mut paints, Affine::scale(3.0));

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

#[vello_bench]
fn image_extend_repeat<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
    let im = get_small_image(peniko::Extend::Repeat, ImageQuality::Low);
    let mut paints = vec![];
    let paint = im.encode_into(
        &mut paints,
        Affine::translate((WideTile::WIDTH as f64 / 2.0, 0.0)),
    );

    fine.fill(
        0,
        WideTile::WIDTH as usize,
        &paint,
        BlendMode::default(),
        &paints,
        None,
        None,
    );
    std::hint::black_box(&fine);
}

pub fn run_benchmarks() {
    image_no_transform();
    image_scale();
    image_rotate();
    image_quality_medium();
    image_quality_high();
    image_extend_repeat();
}
