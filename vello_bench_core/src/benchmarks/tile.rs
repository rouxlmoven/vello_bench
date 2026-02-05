use crate::data::get_data_items;
use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::Level;
use vello_common::tile::Tiles;

const CATEGORY: &str = "tile";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_data_items(CATEGORY)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    let items = get_data_items();
    let item = items.iter().find(|i| i.name == name)?;
    let lines = item.lines();
    let simd_variant = level_suffix(level);

    let mut tiles = Tiles::new(level);

    Some(runner.run(
        &format!("{CATEGORY}/{name}"),
        CATEGORY,
        name,
        simd_variant,
        #[inline(always)]
        || {
            tiles.reset();
            tiles.make_tiles_analytic_aa(&lines, item.width, item.height);
            std::hint::black_box(&tiles);
        },
    ))
}
