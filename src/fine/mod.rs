// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod blend;
pub mod fill;
pub mod gradient;
pub mod image;
pub mod pack;
pub mod rounded_blurred_rect;
pub mod strip;

use vello_common::peniko::{BlendMode, Compose, Mix};

pub(crate) fn default_blend() -> BlendMode {
    BlendMode::new(Mix::Normal, Compose::SrcOver)
}
