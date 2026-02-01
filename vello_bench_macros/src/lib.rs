// Copyright 2025 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Proc-macros for benchmarking vello sparse strip implementations.

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Create a new Vello benchmark for fine rasterization.
///
/// This macro expects a function that takes a `Fine` as input, and will generate
/// a benchmark function that creates Fine instances with the appropriate SIMD backend.
///
/// ## Example
/// ```ignore
/// #[vello_bench]
/// pub fn transparent_short<S: Simd, T: FineKernel<S>>(fine: &mut Fine<S, T>) {
///     let paint = Paint::Solid(PremulColor::from_alpha_color(ROYAL_BLUE.with_alpha(0.3)));
///     fine.fill(0, 32, &paint, BlendMode::default(), &[], None, None);
///     std::hint::black_box(&fine);
/// }
/// ```
#[proc_macro_attribute]
pub fn vello_bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    let input_fn_name = input_fn.sig.ident.clone();
    let input_fn_name_str = input_fn.sig.ident.to_string();
    let inner_fn_name = Ident::new(&format!("{input_fn_name}_inner"), input_fn_name.span());

    input_fn.sig.ident = inner_fn_name.clone();

    let expanded = quote! {
        #input_fn

        pub fn #input_fn_name() {
            use vello_cpu::fine::{Fine, U8Kernel, F32Kernel};
            use vello_common::fearless_simd::Simd;
            use vello_cpu::Level;

            fn get_bench_name(suffix1: &str, suffix2: &str) -> String {
                let module_path = module_path!();

                let module = module_path
                    .split("::")
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("/");

                format!("{}/{}_{}", module, suffix1, suffix2)
            }

            fn run_integer<S: Simd>(name: &str, simd: S) {
                let mut fine = Fine::<S, U8Kernel>::new(simd);
                crate::run_bench(name, || {
                    #inner_fn_name(&mut fine);
                });
            }

            fn run_float<S: Simd>(name: &str, simd: S) {
                let mut fine = Fine::<S, F32Kernel>::new(simd);
                crate::run_bench(name, || {
                    #inner_fn_name(&mut fine);
                });
            }

            // Run u8 SIMD benchmark
            #[cfg(target_arch = "aarch64")]
            if let Some(neon) = Level::new().as_neon() {
                run_integer(&get_bench_name(#input_fn_name_str, "u8_neon"), neon);
            }

            #[cfg(target_arch = "x86_64")]
            if let Some(avx2) = Level::new().as_avx2() {
                run_integer(&get_bench_name(#input_fn_name_str, "u8_avx2"), avx2);
            } else if let Some(sse42) = Level::new().as_sse42() {
                run_integer(&get_bench_name(#input_fn_name_str, "u8_sse42"), sse42);
            }

            // WASM SIMD is determined at compile time via target_feature
            #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
            {
                // Safety: We only reach this code when simd128 target feature is enabled
                run_integer(&get_bench_name(#input_fn_name_str, "u8_wasm_simd128"), vello_common::fearless_simd::WasmSimd128::new_unchecked());
            }

            #[cfg(all(target_arch = "wasm32", not(target_feature = "simd128")))]
            {
                run_integer(&get_bench_name(#input_fn_name_str, "u8_wasm_scalar"), vello_common::fearless_simd::Fallback::new());
            }

            // Fallback for platforms without SIMD
            #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "wasm32")))]
            {
                run_integer(&get_bench_name(#input_fn_name_str, "u8_scalar"), vello_common::fearless_simd::Fallback::new());
            }
        }
    };

    expanded.into()
}
