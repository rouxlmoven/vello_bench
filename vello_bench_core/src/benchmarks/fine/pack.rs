use crate::registry::BenchmarkInfo;
use crate::result::BenchmarkResult;
use crate::runner::BenchRunner;
use crate::simd::level_suffix;
use fearless_simd::{Level, dispatch};
use vello_common::coarse::WideTile;
use vello_common::tile::Tile;
use vello_cpu::fine::{Fine, U8Kernel, SCRATCH_BUF_SIZE};
use vello_cpu::region::Regions;

const NAMES: &[&str] = &["block", "regular"];
const CATEGORY: &str = "fine/pack";

pub fn list() -> Vec<BenchmarkInfo> {
    BenchmarkInfo::from_names(CATEGORY, NAMES)
}

pub fn run(name: &str, runner: &BenchRunner, level: Level) -> Option<BenchmarkResult> {
    if !NAMES.contains(&name) {
        return None;
    }

    let width = match name {
        "block" => WideTile::WIDTH,
        "regular" => WideTile::WIDTH - 1,
        _ => panic!("unknown fine/pack benchmark: {name}"),
    };

    let simd_variant = level_suffix(level);

    Some(dispatch!(level, simd => {
        let fine = Fine::<_, U8Kernel>::new(simd);
        let mut buf = vec![0; SCRATCH_BUF_SIZE];

        runner.run(
            &format!("{CATEGORY}/{name}"),
            CATEGORY,
            name,
            simd_variant,
            #[inline(always)]
            || {
                let mut regions = Regions::new(width, Tile::HEIGHT, &mut buf);
                regions.update_regions(|region| {
                    fine.pack(region);
                });
                std::hint::black_box(&regions);
            },
        )
    }))
}
