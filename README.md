# vello_bench

A benchmarking tool for [Vello](https://github.com/linebender/vello)'s sparse strips CPU renderer. It can run benchmarks both natively and in WASM, making it easy to compare performance across platforms and browsers.

## Project Structure

- **`vello_bench_core`** — Core benchmarking library shared by all targets.
- **`vello_bench_wasm`** — WASM bindings for running benchmarks in the browser.
- **`vello_bench_tauri`** — Tauri app that can run benchmarks both natively and in WASM side by side.
- **`ui/`** — Web frontend used by both the standalone server and the Tauri app.

## Prerequisites

- [Rust](https://rustup.rs/) (1.85+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Tauri CLI](https://tauri.app/start/) (optional, for the desktop app)

## Setup

### 1. Point Vello dependencies to your local checkout

`vello_bench_core` expects a local Vello repository at `../../vello` relative to itself. Adjust the paths in `vello_bench_core/Cargo.toml` if your Vello checkout is in a different location:

```toml
vello_common = { path = "../../vello/sparse_strips/vello_common" }
vello_cpu = { path = "../../vello/sparse_strips/vello_cpu" }
```

### 2. Build the WASM blobs

```sh
./build.sh
```

This builds both a scalar and a SIMD128 WASM package into `ui/pkg/` and `ui/pkg-simd/`.

### 3. Run

**Option A: Browser only**

```sh
./serve.sh
```

This starts a local HTTP server at `http://localhost:8080`. Open it in any browser to run the WASM benchmarks. Use `--port` to change the port.

**Option B: Tauri app (native + WASM)**

```sh
./tauri.sh
```

This launches the Tauri desktop app, which can run benchmarks both natively and in WASM, allowing direct comparison between the two.

> **Note:** `./tauri.sh` automatically rebuilds the WASM blobs before launching. If you run `cargo tauri dev` directly instead, you need to manually re-run `./build.sh` after changes to `vello_bench_core` or `vello_bench_wasm` to see updated WASM results.

## Benchmark Stability

Some benchmarks may produce unstable results between runs (in my case the tile benchmark sometimes was very random). 
So make sure to experiment by running your target benchmark multiple times, and you can also increase the calibration 
and measurement times in the UI to (hopefully) improve stability, at the cost of longer waiting times.
