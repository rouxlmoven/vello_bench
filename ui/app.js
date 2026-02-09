// Vello Benchmark Suite - Web UI

const DEFAULT_CALIBRATION_MS = 100;
const DEFAULT_MEASUREMENT_MS = 250;

const state = {
    benchmarks: [],
    results: new Map(),
    selectedBenchmarks: [],
    queuedBenchmarks: new Set(),
    runningBenchmark: null,
    runningPhase: null,
    activeTab: 'micro', // 'micro' or 'scene'
    currentCategory: 'all',
    expandedCategories: new Set(),
    isRunning: false,
    abortRequested: false,
    isTauri: false,
    wasmWorker: null,
    wasmSimdLevel: 'scalar',
    wasmSimd128Available: false,
    executionMode: 'native',
    pendingWasmResolve: null,
    references: [],
    loadedReference: null,
    referenceResults: new Map(),
    // Main-thread WASM module for hybrid WebGL benchmarks
    mainThreadWasm: null,
    hybridCanvas: null,
    hybridInitialized: false,
};

// Returns true if the given category belongs to the "scene" tab.
function isSceneCategory(category) {
    return category.startsWith('scene_');
}

function detectTauri() {
    return window.__TAURI__ !== undefined;
}

async function invoke(cmd, args = {}) {
    if (window.__TAURI__?.core?.invoke) {
        return await window.__TAURI__.core.invoke(cmd, args);
    } else if (window.__TAURI__?.invoke) {
        return await window.__TAURI__.invoke(cmd, args);
    }
    throw new Error('Tauri not available');
}

function createWasmWorker() {
    const worker = new Worker('worker.js', { type: 'module' });

    worker.onmessage = (e) => {
        const { type, ...data } = e.data;
        if (!state.pendingWasmResolve) return;

        switch (type) {
            case 'result':
                state.pendingWasmResolve(data.result);
                state.pendingWasmResolve = null;
                break;
            case 'error':
                console.error('Worker error:', data.error);
                state.pendingWasmResolve(null);
                state.pendingWasmResolve = null;
                break;
            case 'benchmarks':
                state.pendingWasmResolve(data.benchmarks);
                state.pendingWasmResolve = null;
                break;
        }
    };

    worker.onerror = (e) => console.error('Worker error:', e);
    state.wasmWorker = worker;
}

async function loadWasmFrom(pkgDir) {
    if (!state.wasmWorker) {
        createWasmWorker();
    }

    return new Promise((resolve) => {
        const handler = (e) => {
            if (e.data.type === 'loaded') {
                state.wasmWorker.removeEventListener('message', handler);
                resolve(e.data.success);
            }
        };
        state.wasmWorker.addEventListener('message', handler);
        state.wasmWorker.postMessage({ type: 'load', pkgDir });
    });
}

async function checkSimd128Available() {
    try {
        const response = await fetch('./pkg-simd/vello_bench_wasm.js', { method: 'HEAD' });
        return response.ok;
    } catch (e) {
        return false;
    }
}

async function loadWasm() {
    state.wasmSimd128Available = await checkSimd128Available();
    const pkgDir = state.wasmSimd128Available ? 'pkg-simd' : 'pkg';
    state.wasmSimdLevel = state.wasmSimd128Available ? 'simd128' : 'scalar';
    return await loadWasmFrom(pkgDir);
}

// Load the WASM module on the main thread for hybrid WebGL benchmarks.
// This is separate from the worker-loaded module.
async function loadMainThreadWasm() {
    try {
        const pkgDir = state.wasmSimd128Available ? 'pkg-simd' : 'pkg';
        const module = await import(`./${pkgDir}/vello_bench_wasm.js`);
        await module.default();
        state.mainThreadWasm = module;
        return true;
    } catch (e) {
        console.error('Failed to load main-thread WASM:', e);
        return false;
    }
}

// Initialize the hybrid WebGL renderer with a hidden canvas.
function initHybridRenderer() {
    if (state.hybridInitialized || !state.mainThreadWasm) return false;
    try {
        // Create a hidden canvas for WebGL rendering
        const canvas = document.createElement('canvas');
        canvas.width = 1024;
        canvas.height = 768;
        canvas.style.display = 'none';
        canvas.id = 'hybrid-bench-canvas';
        document.body.appendChild(canvas);
        state.hybridCanvas = canvas;

        const success = state.mainThreadWasm.init_hybrid(canvas);
        state.hybridInitialized = success;
        return success;
    } catch (e) {
        console.error('Failed to init hybrid renderer:', e);
        return false;
    }
}

// Check if a benchmark ID is a hybrid scene benchmark
function isHybridBenchmark(id) {
    return id.startsWith('scene_hybrid/');
}

async function switchWasmSimdLevel(level) {
    if (level === state.wasmSimdLevel) return true;

    const pkgDir = level === 'simd128' ? 'pkg-simd' : 'pkg';
    const success = await loadWasmFrom(pkgDir);
    if (success) {
        state.wasmSimdLevel = level;
        await loadBenchmarks();
    }
    return success;
}

async function init() {
    state.isTauri = detectTauri();

    document.getElementById('calibration-ms').value = DEFAULT_CALIBRATION_MS;
    document.getElementById('measurement-ms').value = DEFAULT_MEASUREMENT_MS;

    const execMode = document.getElementById('exec-mode');
    if (state.isTauri) {
        execMode.innerHTML = `
            <option value="native">Native (Tauri)</option>
            <option value="wasm">WASM (Browser)</option>
        `;
        execMode.value = 'native';
        state.executionMode = 'native';
    } else {
        execMode.innerHTML = '<option value="wasm">WASM (Browser)</option>';
        execMode.value = 'wasm';
        state.executionMode = 'wasm';
    }

    const wasmLoaded = await loadWasm();

    if (!state.isTauri && !wasmLoaded) {
        document.getElementById('benchmark-tbody').innerHTML =
            '<tr><td colspan="7" class="no-results">Failed to load WASM module. Build it with: ./scripts/build-wasm.sh</td></tr>';
        return;
    }

    // Load main-thread WASM for hybrid WebGL benchmarks.
    // This is needed in both Tauri and browser modes so that scene_hybrid
    // benchmarks can run via WebGL on the main thread when WASM mode is selected.
    const mainLoaded = await loadMainThreadWasm();
    if (mainLoaded) {
        initHybridRenderer();
    }

    await loadSimdLevels();
    await loadBenchmarks();
    await loadReferencesList();
    setupEventListeners();
    setupScreenshotDialogListeners();
    updateSkiaBadge();
}

async function loadSimdLevels() {
    try {
        let levels;
        if (state.executionMode === 'native' && state.isTauri) {
            levels = await invoke('get_simd_levels');
        } else {
            levels = [{ id: 'scalar', name: 'Scalar' }];
            if (state.wasmSimd128Available) {
                levels.unshift({ id: 'simd128', name: 'SIMD128' });
            }
        }

        const select = document.getElementById('simd-level');
        select.innerHTML = levels.map(l =>
            `<option value="${l.id}">${l.name}</option>`
        ).join('');

        if (state.executionMode === 'wasm') {
            select.value = state.wasmSimdLevel;
        }
    } catch (e) {
        console.error('Failed to load SIMD levels:', e);
    }
}

async function loadBenchmarks() {
    try {
        if (state.executionMode === 'native' && state.isTauri) {
            state.benchmarks = await invoke('list_benchmarks');
        } else if (state.wasmWorker) {
            state.benchmarks = await new Promise((resolve) => {
                state.pendingWasmResolve = resolve;
                state.wasmWorker.postMessage({ type: 'list' });
            });
        } else {
            state.benchmarks = [];
        }

        renderCategories(Array.from(getCategorySet()));
        renderBenchmarks();
        updateStats();
        updateRunButtons();
    } catch (e) {
        console.error('Failed to load benchmarks:', e);
    }
}

function buildCategoryTree(categories) {
    const tree = { children: {}, fullPath: '' };

    for (const cat of categories) {
        if (cat === 'all') continue;
        const parts = cat.split('/');
        let node = tree;
        let path = '';
        for (const part of parts) {
            path = path ? `${path}/${part}` : part;
            if (!node.children[part]) {
                node.children[part] = { name: part, fullPath: path, children: {} };
            }
            node = node.children[part];
        }
    }

    return tree;
}

function renderCategoryTree(node, depth = 0) {
    let html = '';
    const children = Object.values(node.children).sort((a, b) => a.name.localeCompare(b.name));

    for (const child of children) {
        const hasChildren = Object.keys(child.children).length > 0;
        const isActive = state.currentCategory === child.fullPath;
        const isExpanded = state.expandedCategories.has(child.fullPath);
        const indent = depth * 12;

        html += `
            <li class="category-item ${isActive ? 'active' : ''}"
                data-category="${child.fullPath}"
                style="padding-left: ${8 + indent}px;">
                ${hasChildren ? `<span class="tree-toggle" data-toggle="${child.fullPath}">${isExpanded ? '▼' : '▶'}</span>` : '<span class="tree-spacer"></span>'}
                ${child.name}
            </li>
        `;

        if (hasChildren && isExpanded) {
            html += renderCategoryTree(child, depth + 1);
        }
    }

    return html;
}

function renderCategories(categories) {
    const list = document.getElementById('category-list');
    const tree = buildCategoryTree(categories);

    if (state.expandedCategories.size === 0) {
        for (const child of Object.values(tree.children)) {
            state.expandedCategories.add(child.fullPath);
        }
    }

    let html = `
        <li class="category-item ${state.currentCategory === 'all' ? 'active' : ''}"
            data-category="all">
            All Benchmarks
        </li>
    `;

    html += renderCategoryTree(tree);
    list.innerHTML = html;
}

function getCategorySet() {
    const categories = new Set(['all']);
    state.benchmarks.forEach(b => {
        if (b.category) {
            // Only include categories matching the active tab
            const scene = isSceneCategory(b.category);
            if ((state.activeTab === 'scene' && scene) || (state.activeTab === 'micro' && !scene)) {
                categories.add(b.category);
            }
        }
    });
    return categories;
}

function getFilteredBenchmarks() {
    // First filter by active tab (scene vs micro)
    const tabFiltered = state.benchmarks.filter(b => {
        const scene = isSceneCategory(b.category);
        return state.activeTab === 'scene' ? scene : !scene;
    });

    if (state.currentCategory === 'all') return tabFiltered;
    return tabFiltered.filter(b =>
        b.category === state.currentCategory ||
        b.category.startsWith(state.currentCategory + '/')
    );
}

function renderBenchmarks() {
    const tbody = document.getElementById('benchmark-tbody');
    const filtered = getFilteredBenchmarks();

    const selectAll = document.getElementById('select-all');
    if (selectAll) {
        selectAll.checked = filtered.length > 0 && filtered.every(b => state.selectedBenchmarks.includes(b.id));
        selectAll.disabled = state.isRunning;
    }

    if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td colspan="8" class="no-results">No benchmarks available.</td></tr>';
        return;
    }

    tbody.innerHTML = filtered.map(bench => {
        const result = state.results.get(bench.id);
        const refResult = state.referenceResults.get(bench.id);
        const isSelected = state.selectedBenchmarks.includes(bench.id);

        let status = 'idle';
        let statusText = 'idle';
        if (state.runningBenchmark === bench.id) {
            status = state.runningPhase === 'calibrating' ? 'calibrating' : 'running';
            statusText = state.runningPhase;
        } else if (state.queuedBenchmarks.has(bench.id)) {
            status = 'queued';
            statusText = 'queued';
        } else if (result) {
            status = 'completed';
            statusText = 'done';
        }

        const meanStr = result
            ? (() => { const { mean, unit } = formatTime(result.statistics.mean_ns); return `${mean.toFixed(3)} ${unit}`; })()
            : '-';

        let refStr = '-';
        let changeStr = '-';
        let changeClass = '';

        if (refResult) {
            const { mean, unit } = formatTime(refResult.statistics.mean_ns);
            refStr = `${mean.toFixed(3)} ${unit}`;
        }

        if (result && refResult) {
            const comparison = calculateComparison(result.statistics.mean_ns, refResult.statistics.mean_ns);
            if (comparison) {
                const sign = comparison.percentChange > 0 ? '+' : '';
                changeStr = `${sign}${comparison.percentChange.toFixed(1)}%`;

                if (comparison.status === 'faster') {
                    changeClass = 'change-faster';
                    changeStr += ` (${comparison.speedup.toFixed(2)}x)`;
                } else if (comparison.status === 'slower') {
                    changeClass = 'change-slower';
                    changeStr += ` (${(1/comparison.speedup).toFixed(2)}x)`;
                } else {
                    changeClass = 'change-similar';
                }
            }
        }

        const rowClasses = [status];
        if (isSelected) rowClasses.push('selected');

        const isScene = isSceneCategory(bench.category);

        return `
            <tr class="${rowClasses.join(' ')}" data-id="${bench.id}">
                <td class="col-select">
                    <input type="checkbox" class="row-checkbox" ${isSelected ? 'checked' : ''} ${state.isRunning ? 'disabled' : ''}>
                </td>
                <td class="col-name">${bench.name}</td>
                <td class="col-category">${bench.category}</td>
                <td class="col-status"><span class="status-badge ${status}">${statusText}</span></td>
                <td class="col-mean"><span class="result-mean">${meanStr}</span></td>
                <td class="col-ref"><span class="result-ref">${refStr}</span></td>
                <td class="col-change"><span class="result-change ${changeClass}">${changeStr}</span></td>
                <td class="col-actions">${isScene
                    ? `<button class="screenshot-btn" data-screenshot="${bench.id}" title="Capture screenshot">&#128247;</button>`
                    : ''}</td>
            </tr>
        `;
    }).join('');
}

function formatTime(meanNs) {
    if (meanNs >= 1_000_000_000) {
        return { mean: meanNs / 1_000_000_000, unit: 's' };
    } else if (meanNs >= 1_000_000) {
        return { mean: meanNs / 1_000_000, unit: 'ms' };
    } else if (meanNs >= 1_000) {
        return { mean: meanNs / 1_000, unit: '\u00b5s' };
    } else {
        return { mean: meanNs, unit: 'ns' };
    }
}

function updateStats() {
    const tabFiltered = state.benchmarks.filter(b => {
        const scene = isSceneCategory(b.category);
        return state.activeTab === 'scene' ? scene : !scene;
    });
    const completedCount = tabFiltered.filter(b => state.results.has(b.id)).length;
    document.getElementById('bench-count').textContent =
        `${tabFiltered.length} benchmarks`;
    document.getElementById('bench-completed').textContent =
        `${completedCount} completed`;
}

function getTimingConfig() {
    const calibrationMs = Math.max(100, parseInt(document.getElementById('calibration-ms').value) || DEFAULT_CALIBRATION_MS);
    const measurementMs = Math.max(100, parseInt(document.getElementById('measurement-ms').value) || DEFAULT_MEASUREMENT_MS);
    return { calibrationMs, measurementMs };
}

async function runSingleBenchmark(id) {
    const simdLevel = document.getElementById('simd-level').value;
    const { calibrationMs, measurementMs } = getTimingConfig();

    if (state.executionMode === 'native' && state.isTauri) {
        return await invoke('run_benchmark', { id, simdLevel, calibrationMs, measurementMs });
    }

    // Hybrid WebGL benchmarks run on the main thread (needs canvas/WebGL context)
    if (isHybridBenchmark(id) && state.hybridInitialized && state.mainThreadWasm) {
        // Yield to let the UI update before blocking the main thread
        await new Promise(resolve => setTimeout(resolve, 0));
        const result = state.mainThreadWasm.run_hybrid_benchmark(id, calibrationMs, measurementMs);
        return result;
    }

    // All other benchmarks run in the web worker
    if (state.wasmWorker) {
        return new Promise((resolve) => {
            state.pendingWasmResolve = resolve;
            state.wasmWorker.postMessage({ type: 'run', id, calibrationMs, measurementMs });
        });
    }
    return null;
}

function abortBenchmarks() {
    if (state.isRunning) {
        state.abortRequested = true;
    }
}

async function runBenchmarks(ids) {
    if (state.isRunning || ids.length === 0) return;

    state.isRunning = true;
    state.abortRequested = false;

    for (const id of ids) {
        state.results.delete(id);
        state.queuedBenchmarks.add(id);
    }
    renderBenchmarks();
    updateStats();
    updateRunButtons();

    for (const id of ids) {
        if (state.abortRequested) break;

        state.queuedBenchmarks.delete(id);
        state.runningBenchmark = id;
        state.runningPhase = 'calibrating';
        renderBenchmarks();

        const phaseTimer = setTimeout(() => {
            if (state.runningBenchmark === id && state.runningPhase === 'calibrating') {
                state.runningPhase = 'measuring';
                renderBenchmarks();
            }
        }, getTimingConfig().calibrationMs);

        await new Promise(resolve => setTimeout(resolve, 0));

        try {
            const result = await runSingleBenchmark(id);
            if (result) {
                state.results.set(id, result);
            }
        } catch (e) {
            console.error(`Failed to run benchmark ${id}:`, e);
        }

        clearTimeout(phaseTimer);
        state.runningBenchmark = null;
        state.runningPhase = null;
        renderBenchmarks();
        updateStats();
    }

    state.isRunning = false;
    state.abortRequested = false;
    state.queuedBenchmarks.clear();
    renderBenchmarks();
    updateRunButtons();
}

function updateRunButtons() {
    const runBtn = document.getElementById('run-btn');
    const abortBtn = document.getElementById('abort-btn');

    if (state.isRunning) {
        runBtn.style.display = 'none';
        if (abortBtn) abortBtn.style.display = 'inline-block';
    } else {
        runBtn.style.display = 'inline-block';
        runBtn.disabled = state.benchmarks.length === 0;
        if (abortBtn) abortBtn.style.display = 'none';
    }
}

function exportResults() {
    const results = Array.from(state.results.values());
    const json = JSON.stringify(results, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = `vello-bench-results-${Date.now()}.json`;
    a.click();

    URL.revokeObjectURL(url);
}

async function loadReferencesList() {
    if (!state.isTauri) return;

    try {
        state.references = await invoke('list_references');
        renderReferencesDropdown();
    } catch (e) {
        console.error('Failed to load references list:', e);
    }
}

function renderReferencesDropdown() {
    const select = document.getElementById('reference-select');
    if (!select) return;

    const currentValue = select.value;
    select.innerHTML = '<option value="">No reference</option>';

    for (const entry of state.references) {
        select.innerHTML += `<option value="${entry.name}">${entry.name}</option>`;
    }

    if (currentValue && state.references.some(r => r.name === currentValue)) {
        select.value = currentValue;
    }

    const deleteBtn = document.getElementById('delete-reference-btn');
    if (deleteBtn) {
        deleteBtn.disabled = !select.value;
    }
}

function showSaveDialog() {
    return new Promise((resolve) => {
        const overlay = document.getElementById('save-dialog');
        const input = document.getElementById('save-dialog-input');
        const cancelBtn = document.getElementById('save-dialog-cancel');
        const confirmBtn = document.getElementById('save-dialog-confirm');

        input.value = '';
        overlay.style.display = 'flex';
        input.focus();

        const cleanup = () => {
            overlay.style.display = 'none';
            cancelBtn.removeEventListener('click', onCancel);
            confirmBtn.removeEventListener('click', onConfirm);
            input.removeEventListener('keydown', onKeydown);
        };

        const onCancel = () => { cleanup(); resolve(null); };
        const onConfirm = () => { cleanup(); resolve(input.value.trim() || null); };
        const onKeydown = (e) => {
            if (e.key === 'Enter') onConfirm();
            if (e.key === 'Escape') onCancel();
        };

        cancelBtn.addEventListener('click', onCancel);
        confirmBtn.addEventListener('click', onConfirm);
        input.addEventListener('keydown', onKeydown);
    });
}

async function saveReference() {
    if (!state.isTauri) {
        alert('Saving references is only available in the Tauri app.');
        return;
    }
    if (state.results.size === 0) {
        alert('No benchmark results to save.');
        return;
    }

    const name = await showSaveDialog();
    if (!name) return;

    try {
        const results = Array.from(state.results.values());
        await invoke('save_reference', { name, results });
        await loadReferencesList();
    } catch (e) {
        console.error('Failed to save reference:', e);
        alert(`Failed to save reference: ${e}`);
    }
}

async function loadReference(name) {
    if (!name) {
        state.loadedReference = null;
        state.referenceResults.clear();
        renderBenchmarks();
        updateReferenceUI();
        return;
    }

    try {
        const results = await invoke('load_reference', { name });
        state.loadedReference = name;
        state.referenceResults.clear();
        for (const result of results) {
            state.referenceResults.set(result.id, result);
        }
        renderBenchmarks();
        updateReferenceUI();
    } catch (e) {
        console.error('Failed to load reference:', e);
        alert(`Failed to load reference: ${e}`);
    }
}

function showConfirmDialog(title, message) {
    return new Promise((resolve) => {
        const overlay = document.getElementById('confirm-dialog');
        const titleEl = document.getElementById('confirm-dialog-title');
        const messageEl = document.getElementById('confirm-dialog-message');
        const cancelBtn = document.getElementById('confirm-dialog-cancel');
        const confirmBtn = document.getElementById('confirm-dialog-confirm');

        titleEl.textContent = title;
        messageEl.textContent = message;
        overlay.style.display = 'flex';

        const cleanup = () => {
            overlay.style.display = 'none';
            cancelBtn.removeEventListener('click', onCancel);
            confirmBtn.removeEventListener('click', onConfirm);
        };

        const onCancel = () => { cleanup(); resolve(false); };
        const onConfirm = () => { cleanup(); resolve(true); };

        cancelBtn.addEventListener('click', onCancel);
        confirmBtn.addEventListener('click', onConfirm);
    });
}

async function deleteReference() {
    const select = document.getElementById('reference-select');
    const name = select?.value;
    if (!name) return;

    const confirmed = await showConfirmDialog('Delete Reference', `Are you sure you want to delete "${name}"?`);
    if (!confirmed) return;

    try {
        await invoke('delete_reference', { name });

        if (state.loadedReference === name) {
            state.loadedReference = null;
            state.referenceResults.clear();
            select.value = '';
            renderBenchmarks();
            updateReferenceUI();
        }

        await loadReferencesList();
    } catch (e) {
        console.error('Failed to delete reference:', e);
    }
}

function updateReferenceUI() {
    const deleteBtn = document.getElementById('delete-reference-btn');
    const select = document.getElementById('reference-select');
    const currentName = document.getElementById('reference-current-name');

    if (deleteBtn && select) {
        deleteBtn.disabled = !select.value;
    }

    if (currentName) {
        if (state.loadedReference) {
            const entry = state.references.find(r => r.name === state.loadedReference);
            if (entry) {
                const date = new Date(entry.created_at).toLocaleDateString();
                currentName.innerHTML = `<strong>${entry.name}</strong><br><span class="reference-meta">${date}</span>`;
            } else {
                currentName.textContent = state.loadedReference;
            }
        } else {
            currentName.textContent = 'None';
        }
    }
}

function calculateComparison(currentNs, referenceNs) {
    if (!referenceNs || referenceNs === 0) return null;

    const diff = currentNs - referenceNs;
    let percentChange = (diff / referenceNs) * 100;
    const speedup = referenceNs / currentNs;

    // Round small values to just 0.
    if (Math.abs(percentChange) < 0.005) {
        percentChange = 0;
    }

    let status;
    if (Math.abs(percentChange) <= 5) {
        status = 'similar';
    } else if (percentChange < 0) {
        status = 'faster';
    } else {
        status = 'slower';
    }

    return { diff, percentChange, speedup, status };
}

// ---------------------------------------------------------------------------
// Screenshot capture
// ---------------------------------------------------------------------------

async function captureScreenshot(benchId) {
    const dialog = document.getElementById('screenshot-dialog');
    const title = document.getElementById('screenshot-dialog-title');
    const body = document.getElementById('screenshot-dialog-body');
    // Extract the scene name and category from the benchmark ID
    let sceneName, category;
    if (benchId.startsWith('scene_cpu/')) {
        sceneName = benchId.slice('scene_cpu/'.length);
        category = 'scene_cpu';
    } else if (benchId.startsWith('scene_hybrid/')) {
        sceneName = benchId.slice('scene_hybrid/'.length);
        category = 'scene_hybrid';
    } else if (benchId.startsWith('scene_skia/')) {
        sceneName = benchId.slice('scene_skia/'.length);
        category = 'scene_skia';
    } else {
        return;
    }

    title.textContent = `Screenshot: ${sceneName} (${category})`;
    body.innerHTML = '<p class="screenshot-loading">Rendering...</p>';
    dialog.style.display = 'flex';

    // Yield to let the dialog render
    await new Promise(r => setTimeout(r, 0));

    try {
        let dataUrl;

        if (state.isTauri && state.executionMode === 'native') {
            // Tauri native: render via Tauri command using the matching renderer
            // This handles all categories: scene_cpu, scene_hybrid, scene_skia
            const result = await invoke('screenshot', { sceneName, category });
            if (!result) throw new Error('Screenshot failed');
            const rgba = Uint8ClampedArray.from(atob(result.rgba_base64), c => c.charCodeAt(0));
            dataUrl = rgbaToDataUrl(rgba, result.width, result.height);
        } else if (category === 'scene_hybrid' && state.hybridInitialized && state.mainThreadWasm) {
            // Hybrid WebGL: render once to canvas, then grab its content
            const success = state.mainThreadWasm.render_hybrid_once(sceneName);
            if (!success) throw new Error('Hybrid render failed');
            dataUrl = state.hybridCanvas.toDataURL('image/png');
        } else if (category === 'scene_cpu' && state.mainThreadWasm) {
            // CPU: render via WASM and get raw pixel data
            const result = state.mainThreadWasm.screenshot_cpu(sceneName);
            if (!result) throw new Error('CPU screenshot failed');
            dataUrl = rgbaToDataUrl(result.data, result.width, result.height);
        } else if (category === 'scene_skia') {
            throw new Error('Skia screenshots are only available in native mode');
        } else {
            throw new Error('No rendering backend available for screenshots');
        }

        // Display the screenshot
        const img = new Image();
        img.onload = () => {
            body.innerHTML = '';
            const canvas = document.createElement('canvas');
            canvas.width = img.width;
            canvas.height = img.height;
            const ctx = canvas.getContext('2d');
            ctx.drawImage(img, 0, 0);
            body.appendChild(canvas);

            const info = document.createElement('p');
            info.className = 'screenshot-info';
            info.textContent = `${img.width} x ${img.height} px`;
            body.appendChild(info);

        };
        img.src = dataUrl;
    } catch (err) {
        body.innerHTML = `<p class="screenshot-loading" style="color: var(--danger);">Error: ${err.message}</p>`;
        console.error('Screenshot failed:', err);
    }
}

// Convert raw RGBA pixel data to a PNG data URL via an off-screen canvas.
function rgbaToDataUrl(rgbaBytes, width, height) {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    const imageData = new ImageData(rgbaBytes, width, height);
    ctx.putImageData(imageData, 0, 0);
    return canvas.toDataURL('image/png');
}

function setupScreenshotDialogListeners() {
    const dialog = document.getElementById('screenshot-dialog');
    const closeBtn = document.getElementById('screenshot-dialog-close');
    const dismissBtn = document.getElementById('screenshot-dialog-dismiss');

    const close = () => { dialog.style.display = 'none'; };
    closeBtn.addEventListener('click', close);
    dismissBtn.addEventListener('click', close);
    dialog.addEventListener('click', (e) => {
        if (e.target === dialog) close();
    });
}

// Update the Skia availability badge visibility based on active tab and mode.
function updateSkiaBadge() {
    const badge = document.getElementById('skia-badge');
    if (!badge) return;

    if (state.activeTab === 'scene') {
        badge.style.display = 'inline-block';
        if (state.executionMode === 'native' && state.isTauri) {
            badge.textContent = 'Skia: available';
            badge.classList.add('skia-available');
            badge.classList.remove('skia-unavailable');
        } else {
            badge.textContent = 'Skia: native only';
            badge.classList.add('skia-unavailable');
            badge.classList.remove('skia-available');
        }
    } else {
        badge.style.display = 'none';
    }
}

// Switch the active tab and re-render.
function switchTab(tab) {
    if (state.activeTab === tab) return;
    state.activeTab = tab;
    state.currentCategory = 'all';

    // Update tab button active states
    document.querySelectorAll('.tab-item').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tab);
    });

    document.getElementById('current-category').textContent = 'All Benchmarks';

    renderCategories(Array.from(getCategorySet()));
    renderBenchmarks();
    updateStats();
    updateSkiaBadge();
}

function setupEventListeners() {
    // Tab switching
    document.querySelectorAll('.tab-item').forEach(btn => {
        btn.addEventListener('click', () => switchTab(btn.dataset.tab));
    });

    document.getElementById('exec-mode').addEventListener('change', async (e) => {
        state.executionMode = e.target.value;
        await loadSimdLevels();
        await loadBenchmarks();
        updateSkiaBadge();
    });

    document.getElementById('simd-level').addEventListener('change', async (e) => {
        if (state.executionMode === 'wasm') {
            await switchWasmSimdLevel(e.target.value);
        }
    });

    document.getElementById('category-list').addEventListener('click', (e) => {
        const toggle = e.target.closest('.tree-toggle');
        if (toggle) {
            const category = toggle.dataset.toggle;
            if (state.expandedCategories.has(category)) {
                state.expandedCategories.delete(category);
            } else {
                state.expandedCategories.add(category);
            }
            renderCategories(Array.from(getCategorySet()));
            return;
        }

        const item = e.target.closest('.category-item');
        if (!item) return;

        state.currentCategory = item.dataset.category;

        if (state.currentCategory !== 'all') {
            state.expandedCategories.add(state.currentCategory);
        }

        document.getElementById('current-category').textContent =
            state.currentCategory === 'all' ? 'All Benchmarks' : state.currentCategory;

        renderCategories(Array.from(getCategorySet()));
        renderBenchmarks();
    });

    document.getElementById('benchmark-tbody').addEventListener('click', (e) => {
        // Handle screenshot button clicks
        const screenshotBtn = e.target.closest('.screenshot-btn');
        if (screenshotBtn) {
            e.stopPropagation();
            const benchId = screenshotBtn.dataset.screenshot;
            captureScreenshot(benchId);
            return;
        }

        if (state.isRunning) return;

        const row = e.target.closest('tr');
        if (!row) return;

        const id = row.dataset.id;
        const index = state.selectedBenchmarks.indexOf(id);

        if (index >= 0) {
            state.selectedBenchmarks.splice(index, 1);
        } else {
            state.selectedBenchmarks.push(id);
        }

        renderBenchmarks();
        updateRunButtons();
    });

    document.getElementById('run-btn').addEventListener('click', () => {
        const visible = getFilteredBenchmarks();
        let ids;
        if (state.selectedBenchmarks.length > 0) {
            const selectedSet = new Set(state.selectedBenchmarks);
            ids = visible.filter(b => selectedSet.has(b.id)).map(b => b.id);
        } else {
            ids = visible.map(b => b.id);
        }
        runBenchmarks(ids);
    });

    document.getElementById('abort-btn').addEventListener('click', abortBenchmarks);
    document.getElementById('export-results').addEventListener('click', exportResults);

    const saveRefBtn = document.getElementById('save-reference-btn');
    if (saveRefBtn) {
        saveRefBtn.addEventListener('click', saveReference);
    }

    const refSelect = document.getElementById('reference-select');
    if (refSelect) {
        refSelect.addEventListener('change', (e) => loadReference(e.target.value));
    }

    const deleteRefBtn = document.getElementById('delete-reference-btn');
    if (deleteRefBtn) {
        deleteRefBtn.addEventListener('click', deleteReference);
    }

    document.getElementById('select-all').addEventListener('change', (e) => {
        if (state.isRunning) {
            e.target.checked = !e.target.checked;
            return;
        }

        const filtered = getFilteredBenchmarks();

        if (e.target.checked) {
            for (const b of filtered) {
                if (!state.selectedBenchmarks.includes(b.id)) {
                    state.selectedBenchmarks.push(b.id);
                }
            }
        } else {
            const visibleIds = new Set(filtered.map(b => b.id));
            state.selectedBenchmarks = state.selectedBenchmarks.filter(id => !visibleIds.has(id));
        }

        renderBenchmarks();
        updateRunButtons();
    });
}

document.addEventListener('DOMContentLoaded', init);
