// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::run_bench;
use vello_common::coarse::WideTile;
use vello_common::color::palette::css::ROYAL_BLUE;
use vello_common::paint::{Paint, PremulColor};
use vello_common::peniko::{BlendMode, Compose, Mix};
use vello_cpu::Level;
use vello_cpu::fine::{Fine, U8Kernel};

pub fn run_benchmarks() {
    let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE));
    let width = WideTile::WIDTH as usize;

    // Get the best available SIMD level
    let level = Level::new();

    // Mix modes
    let mix_modes = [
        ("normal", Mix::Normal),
        ("multiply", Mix::Multiply),
        ("screen", Mix::Screen),
        ("overlay", Mix::Overlay),
        ("darken", Mix::Darken),
        ("lighten", Mix::Lighten),
        ("color_dodge", Mix::ColorDodge),
        ("color_burn", Mix::ColorBurn),
        ("hard_light", Mix::HardLight),
        ("soft_light", Mix::SoftLight),
        ("difference", Mix::Difference),
        ("exclusion", Mix::Exclusion),
        ("hue", Mix::Hue),
        ("saturation", Mix::Saturation),
        ("color", Mix::Color),
        ("luminosity", Mix::Luminosity),
    ];

    #[cfg(target_arch = "aarch64")]
    if let Some(neon) = level.as_neon() {
        let mut fine = Fine::<_, U8Kernel>::new(neon);
        for (name, mix) in mix_modes {
            let blend_mode = BlendMode::new(mix, Compose::SrcOver);
            run_bench(&format!("fine/blend/{}_u8_neon", name), || {
                fine.fill(0, width, &paint, blend_mode, &[], None, None);
                std::hint::black_box(&fine);
            });
        }
    }

    #[cfg(target_arch = "x86_64")]
    if let Some(avx2) = level.as_avx2() {
        let mut fine = Fine::<_, U8Kernel>::new(avx2);
        for (name, mix) in mix_modes {
            let blend_mode = BlendMode::new(mix, Compose::SrcOver);
            run_bench(&format!("fine/blend/{}_u8_avx2", name), || {
                fine.fill(0, width, &paint, blend_mode, &[], None, None);
                std::hint::black_box(&fine);
            });
        }
    } else if let Some(sse42) = level.as_sse42() {
        let mut fine = Fine::<_, U8Kernel>::new(sse42);
        for (name, mix) in mix_modes {
            let blend_mode = BlendMode::new(mix, Compose::SrcOver);
            run_bench(&format!("fine/blend/{}_u8_sse42", name), || {
                fine.fill(0, width, &paint, blend_mode, &[], None, None);
                std::hint::black_box(&fine);
            });
        }
    }

    // Compose modes (just run a few key ones to keep benchmark time reasonable)
    let compose_modes = [
        ("src_over", Compose::SrcOver),
        ("src_in", Compose::SrcIn),
        ("dest_over", Compose::DestOver),
        ("xor", Compose::Xor),
    ];

    #[cfg(target_arch = "aarch64")]
    if let Some(neon) = level.as_neon() {
        let mut fine = Fine::<_, U8Kernel>::new(neon);
        for (name, compose) in compose_modes {
            let blend_mode = BlendMode::new(Mix::Normal, compose);
            run_bench(&format!("fine/blend/{}_u8_neon", name), || {
                fine.fill(0, width, &paint, blend_mode, &[], None, None);
                std::hint::black_box(&fine);
            });
        }
    }

    #[cfg(target_arch = "x86_64")]
    if let Some(avx2) = level.as_avx2() {
        let mut fine = Fine::<_, U8Kernel>::new(avx2);
        for (name, compose) in compose_modes {
            let blend_mode = BlendMode::new(Mix::Normal, compose);
            run_bench(&format!("fine/blend/{}_u8_avx2", name), || {
                fine.fill(0, width, &paint, blend_mode, &[], None, None);
                std::hint::black_box(&fine);
            });
        }
    }
}
