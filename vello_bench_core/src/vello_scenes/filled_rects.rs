//! A simple filled-rectangles scene with no images.

use super::{VelloScene, VelloSceneInfo};
use crate::renderer::Renderer;
use vello_common::kurbo::Rect;
use vello_common::peniko::color::palette;

/// A simple scene that fills a grid of coloured rectangles.
pub struct FilledRects;

impl VelloScene for FilledRects {
    type State = ();

    fn info() -> VelloSceneInfo {
        VelloSceneInfo {
            name: "filled_rects",
            width: 1024,
            height: 768,
        }
    }

    fn setup<R: Renderer>(_r: &mut R) -> Self::State {}

    fn draw<R: Renderer>(_state: &Self::State, r: &mut R) {
        let colors = [
            palette::css::RED,
            palette::css::GREEN,
            palette::css::BLUE,
            palette::css::YELLOW,
            palette::css::CYAN,
            palette::css::MAGENTA,
        ];

        let cols = 16u16;
        let rows = 12u16;
        let cell_w = f64::from(r.width()) / f64::from(cols);
        let cell_h = f64::from(r.height()) / f64::from(rows);

        for row in 0..rows {
            for col in 0..cols {
                let idx = ((row * cols + col) as usize) % colors.len();
                r.set_paint(colors[idx]);
                r.fill_rect(&Rect::new(
                    f64::from(col) * cell_w,
                    f64::from(row) * cell_h,
                    f64::from(col + 1) * cell_w,
                    f64::from(row + 1) * cell_h,
                ));
            }
        }
    }
}
