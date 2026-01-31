// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::benchmarks::run_bench;
use crate::data::get_data_items;
use vello_common::flatten;
use vello_common::flatten::FlattenCtx;
use vello_common::kurbo::Stroke;
use vello_common::kurbo::StrokeCtx;
use vello_cpu::Level;
use vello_cpu::kurbo::Affine;

pub fn register() {
    // Registration would go here for the registry-based approach
}

pub fn run_benchmarks() {
    // Flatten benchmarks
    for item in get_data_items() {
        let expanded_strokes = item.expanded_strokes();
        let name = format!("flatten/{}", item.name);

        run_bench(&name, || {
            let mut line_buf: Vec<flatten::Line> = vec![];
            let mut temp_buf: Vec<flatten::Line> = vec![];
            let mut flatten_ctx = FlattenCtx::default();

            line_buf.clear();

            for path in &item.fills {
                flatten::fill(
                    Level::new(),
                    &path.path,
                    path.transform,
                    &mut temp_buf,
                    &mut flatten_ctx,
                );
                line_buf.extend(&temp_buf);
            }

            for stroke in &expanded_strokes {
                flatten::fill(
                    Level::new(),
                    stroke,
                    Affine::IDENTITY,
                    &mut temp_buf,
                    &mut flatten_ctx,
                );
                line_buf.extend(&temp_buf);
            }

            std::hint::black_box(&line_buf);
        });
    }

    // Stroke expansion benchmarks
    for item in get_data_items() {
        let name = format!("strokes/{}", item.name);

        run_bench(&name, || {
            let mut stroke_ctx = StrokeCtx::default();
            let mut paths = vec![];

            for path in &item.strokes {
                let stroke = Stroke {
                    width: path.stroke_width as f64,
                    ..Default::default()
                };
                flatten::expand_stroke(path.path.iter(), &stroke, 0.25, &mut stroke_ctx);
                paths.push(stroke_ctx.output().clone());
            }

            std::hint::black_box(&paths);
        });
    }
}
