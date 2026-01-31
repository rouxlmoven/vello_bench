// Web Worker for running WASM benchmarks without blocking the main thread

let wasmModule = null;
let initFn = null;

// Handle messages from main thread
self.onmessage = async function(e) {
    const { type, ...data } = e.data;

    switch (type) {
        case 'load':
            try {
                const pkgDir = data.pkgDir;
                // Import the wasm-bindgen generated module
                const module = await import(`./${pkgDir}/vello_bench_wasm.js`);
                // Initialize the WASM module
                await module.default();
                wasmModule = module;
                self.postMessage({ type: 'loaded', success: true });
            } catch (e) {
                console.error('Worker: Failed to load WASM:', e);
                self.postMessage({ type: 'loaded', success: false, error: e.message });
            }
            break;

        case 'run':
            if (!wasmModule) {
                self.postMessage({ type: 'error', id: data.id, error: 'WASM not loaded' });
                return;
            }

            try {
                const result = wasmModule.run_benchmark(
                    data.id,
                    BigInt(0),  // warmup unused
                    BigInt(data.measurementMs)
                );
                self.postMessage({ type: 'result', id: data.id, result });
            } catch (e) {
                self.postMessage({ type: 'error', id: data.id, error: e.message });
            }
            break;

        case 'list':
            if (!wasmModule) {
                self.postMessage({ type: 'benchmarks', benchmarks: [] });
                return;
            }
            try {
                const benchmarks = wasmModule.list_benchmarks();
                self.postMessage({ type: 'benchmarks', benchmarks });
            } catch (e) {
                self.postMessage({ type: 'error', error: e.message });
            }
            break;

        case 'platform':
            if (!wasmModule) {
                self.postMessage({ type: 'platformInfo', info: { arch: 'unknown', os: 'unknown', simd_features: [] } });
                return;
            }
            try {
                const info = wasmModule.get_platform_info();
                self.postMessage({ type: 'platformInfo', info });
            } catch (e) {
                self.postMessage({ type: 'error', error: e.message });
            }
            break;
    }
};
