// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use vello_common::coarse::WideTile;
use vello_common::encode::EncodeExt;
use vello_common::kurbo::{Affine, Point};
use vello_common::paint::{Image, ImageSource};
use vello_common::peniko::{BlendMode, Compose, Extend, ImageQuality, ImageSampler, Mix};
use vello_common::pixmap::Pixmap;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, U8Kernel};

const NAMES: &[&str] = &[
    "no_transform",
    "scale",
    "rotate",
    "quality_medium",
    "quality_high",
    "extend_repeat",
];
const CATEGORY: &str = "fine/image";

static COLR_DATA: &[u8] = include_bytes!("../../../assets/big_colr.png");
static SMALL_DATA: &[u8] = include_bytes!("../../../assets/rgb_image_2x2.png");

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_names(CATEGORY, NAMES)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

    let blend = BlendMode::new(Mix::Normal, Compose::SrcOver);

    let quality = match name {
        "quality_medium" => ImageQuality::Medium,
        "quality_high" => ImageQuality::High,
        _ => ImageQuality::Low,
    };

    let extend = match name {
        "extend_repeat" => Extend::Repeat,
        _ => Extend::Pad,
    };

    let (data, transform): (&[u8], Affine) = match name {
        "extend_repeat" => (SMALL_DATA, Affine::translate((WideTile::WIDTH as f64 / 2.0, 0.0))),
        "scale" | "quality_medium" | "quality_high" => (COLR_DATA, Affine::scale(3.0)),
        "rotate" => (
            COLR_DATA,
            Affine::rotate_about(
                1.0,
                Point::new(WideTile::WIDTH as f64 / 2.0, Tile::HEIGHT as f64 / 2.0),
            ),
        ),
        _ => (COLR_DATA, Affine::IDENTITY),
    };

    let pixmap = Pixmap::from_png(data).unwrap();
    let image = Image {
        image: ImageSource::Pixmap(Arc::new(pixmap)),
        sampler: ImageSampler { x_extend: extend, y_extend: extend, quality, alpha: 1.0 },
    };

    let mut paints = vec![];
    let paint = image.encode_into(&mut paints, transform);

    let simd_variant = level_suffix(level);

    Some(dispatch!(level, simd => {
        let mut fine = Fine::<_, U8Kernel>::new(simd);

        runner.run(
            &format!("{CATEGORY}/{name}"),
            CATEGORY,
            name,
            simd_variant,
            || {
                fine.fill(0, WideTile::WIDTH as usize, &paint, blend, &paints, None, None);
                std::hint::black_box(&fine);
            },
        )
    }))
}
