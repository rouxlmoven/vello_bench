// Vello Benchmark Suite - Web UI

// State
const state = {
    benchmarks: [],
    results: new Map(),
    selectedBenchmarks: [], // Array to preserve selection order
    queuedBenchmarks: new Set(),
    runningBenchmark: null,
    runningPhase: null, // 'warmup' or 'measuring'
    currentCategory: 'all',
    expandedCategories: new Set(), // Track expanded tree nodes
    isRunning: false,
    abortRequested: false,
    isTauri: false,
    wasmModule: null,
    wasmSimdLevel: 'scalar', // 'scalar' or 'simd128'
    wasmSimd128Available: false, // whether pkg-simd exists
    executionMode: 'native', // 'native' or 'wasm'
};

// Check if running in Tauri (v2 API)
function detectTauri() {
    return window.__TAURI__ !== undefined;
}

// Invoke a Tauri command (v2 API)
async function invoke(cmd, args = {}) {
    console.log('Invoking command:', cmd, 'with args:', args);

    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        const result = await window.__TAURI__.core.invoke(cmd, args);
        console.log('Result:', result);
        return result;
    } else if (window.__TAURI__ && window.__TAURI__.invoke) {
        const result = await window.__TAURI__.invoke(cmd, args);
        console.log('Result:', result);
        return result;
    }
    console.error('Tauri not available');
    throw new Error('Tauri not available');
}

// Load WASM module from a specific path
async function loadWasmFrom(pkgDir) {
    try {
        const wasmPath = `./${pkgDir}/vello_bench_wasm.js`;
        console.log('Loading WASM from:', wasmPath);

        const wasm = await import(wasmPath);
        await wasm.default(); // Initialize the module
        state.wasmModule = wasm;
        console.log('WASM module loaded successfully from', pkgDir);
        return true;
    } catch (e) {
        console.error('Failed to load WASM module from', pkgDir, ':', e);
        return false;
    }
}

// Check if SIMD128 WASM build is available
async function checkSimd128Available() {
    try {
        const response = await fetch('./pkg-simd/vello_bench_wasm.js', { method: 'HEAD' });
        return response.ok;
    } catch (e) {
        return false;
    }
}

// Load WASM module (scalar by default)
async function loadWasm() {
    // Check if SIMD128 build is available
    state.wasmSimd128Available = await checkSimd128Available();
    console.log('WASM SIMD128 available:', state.wasmSimd128Available);

    // Load scalar version by default, or SIMD128 if available and preferred
    const pkgDir = state.wasmSimd128Available ? 'pkg-simd' : 'pkg';
    state.wasmSimdLevel = state.wasmSimd128Available ? 'simd128' : 'scalar';
    return await loadWasmFrom(pkgDir);
}

// Switch WASM SIMD level
async function switchWasmSimdLevel(level) {
    if (level === state.wasmSimdLevel) return true;

    const pkgDir = level === 'simd128' ? 'pkg-simd' : 'pkg';
    const success = await loadWasmFrom(pkgDir);
    if (success) {
        state.wasmSimdLevel = level;
        // Reload benchmarks for the new module
        await loadBenchmarks();
    }
    return success;
}

// Initialize the application
async function init() {
    state.isTauri = detectTauri();
    console.log('Tauri detected:', state.isTauri);

    // Update execution mode dropdown
    const execMode = document.getElementById('exec-mode');

    if (state.isTauri) {
        // In Tauri, offer both Native and WASM
        execMode.innerHTML = `
            <option value="native">Native (Tauri)</option>
            <option value="wasm">WASM (Browser)</option>
        `;
        execMode.value = 'native';
        state.executionMode = 'native';
    } else {
        // In browser, only WASM is available
        execMode.innerHTML = '<option value="wasm">WASM (Browser)</option>';
        execMode.value = 'wasm';
        state.executionMode = 'wasm';
    }

    // Try to load WASM module
    const wasmLoaded = await loadWasm();

    if (!state.isTauri && !wasmLoaded) {
        document.getElementById('benchmark-tbody').innerHTML =
            '<tr><td colspan="7" class="no-results">Failed to load WASM module. Make sure to build it with: ./scripts/build-wasm.sh</td></tr>';
        return;
    }

    // Load platform info
    await loadPlatformInfo();

    // Load SIMD levels
    await loadSimdLevels();

    // Load benchmarks
    await loadBenchmarks();

    // Set up event listeners
    setupEventListeners();
}

// Load platform information
async function loadPlatformInfo() {
    try {
        let info;
        if (state.executionMode === 'native' && state.isTauri) {
            info = await invoke('get_platform_info');
        } else if (state.wasmModule) {
            info = state.wasmModule.get_platform_info();
        } else {
            info = { arch: 'unknown', os: 'unknown', simd_features: ['unknown'] };
        }
        console.log('Platform info:', info);

        document.getElementById('platform-arch').textContent = info.arch;
        document.getElementById('platform-os').textContent = info.os;
    } catch (e) {
        console.error('Failed to load platform info:', e);
        document.getElementById('platform-arch').textContent = 'error';
        document.getElementById('platform-os').textContent = 'error';
    }
}

// Load available SIMD levels
async function loadSimdLevels() {
    try {
        let levels;
        if (state.executionMode === 'native' && state.isTauri) {
            levels = await invoke('get_simd_levels');
        } else {
            // For WASM mode, we determine levels based on available builds
            levels = [{ id: 'scalar', name: 'Scalar' }];
            if (state.wasmSimd128Available) {
                // SIMD128 is better, so put it first
                levels.unshift({ id: 'simd128', name: 'SIMD128' });
            }
        }
        console.log('SIMD levels:', levels);

        const select = document.getElementById('simd-level');
        select.innerHTML = levels.map(l =>
            `<option value="${l.id}">${l.name}</option>`
        ).join('');

        // Select the current level
        if (state.executionMode === 'wasm') {
            select.value = state.wasmSimdLevel;
        }
    } catch (e) {
        console.error('Failed to load SIMD levels:', e);
    }
}

// Load benchmarks
async function loadBenchmarks() {
    try {
        if (state.executionMode === 'native' && state.isTauri) {
            state.benchmarks = await invoke('list_benchmarks');
        } else if (state.wasmModule) {
            state.benchmarks = state.wasmModule.list_benchmarks();
        } else {
            state.benchmarks = [];
        }
        console.log('Benchmarks:', state.benchmarks);

        // Build category list
        const categories = new Set(['all']);
        state.benchmarks.forEach(b => {
            if (b.category) categories.add(b.category);
        });

        renderCategories(Array.from(categories));
        renderBenchmarks();
        updateStats();
        updateRunButtons();
    } catch (e) {
        console.error('Failed to load benchmarks:', e);
    }
}

// Build category tree from flat list
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

// Render category tree recursively
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

// Render category list as tree
function renderCategories(categories) {
    const list = document.getElementById('category-list');
    const tree = buildCategoryTree(categories);

    // Auto-expand top-level categories on first render
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

// Render benchmark table
function renderBenchmarks() {
    const tbody = document.getElementById('benchmark-tbody');
    const filtered = state.currentCategory === 'all'
        ? state.benchmarks
        : state.benchmarks.filter(b => b.category === state.currentCategory ||
            b.category.startsWith(state.currentCategory + '/'));

    // Update select-all checkbox state
    const selectAll = document.getElementById('select-all');
    if (selectAll) {
        const allSelected = filtered.length > 0 && filtered.every(b => state.selectedBenchmarks.includes(b.id));
        selectAll.checked = allSelected;
        selectAll.disabled = state.isRunning;
    }

    if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" class="no-results">No benchmarks available.</td></tr>';
        return;
    }

    tbody.innerHTML = filtered.map(bench => {
        const result = state.results.get(bench.id);
        const isSelected = state.selectedBenchmarks.includes(bench.id);

        let status = 'idle';
        let statusText = 'idle';
        if (state.runningBenchmark === bench.id) {
            status = state.runningPhase === 'warmup' ? 'warmup' : 'running';
            statusText = state.runningPhase === 'warmup' ? 'warmup' : 'measuring';
        } else if (state.queuedBenchmarks.has(bench.id)) {
            status = 'queued';
            statusText = 'queued';
        } else if (result) {
            status = 'completed';
            statusText = 'done';
        }

        let meanStr = '-';
        let itersStr = '-';
        if (result) {
            const { mean, unit } = formatTime(result.statistics.mean_ns);
            meanStr = `${mean.toFixed(3)} ${unit}`;
            itersStr = result.statistics.iterations.toLocaleString();
        }

        const rowClasses = [status];
        if (isSelected) rowClasses.push('selected');

        return `
            <tr class="${rowClasses.join(' ')}" data-id="${bench.id}">
                <td class="col-select">
                    <input type="checkbox" class="row-checkbox" ${isSelected ? 'checked' : ''} ${state.isRunning ? 'disabled' : ''}>
                </td>
                <td class="col-name">${bench.name}</td>
                <td class="col-category">${bench.category}</td>
                <td class="col-status"><span class="status-badge ${status}">${statusText}</span></td>
                <td class="col-mean"><span class="result-mean">${meanStr}</span></td>
                <td class="col-iters"><span class="result-iters">${itersStr}</span></td>
            </tr>
        `;
    }).join('');
}

// Format time with appropriate unit
function formatTime(meanNs) {
    if (meanNs >= 1_000_000_000) {
        return { mean: meanNs / 1_000_000_000, unit: 's' };
    } else if (meanNs >= 1_000_000) {
        return { mean: meanNs / 1_000_000, unit: 'ms' };
    } else if (meanNs >= 1_000) {
        return { mean: meanNs / 1_000, unit: 'µs' };
    } else {
        return { mean: meanNs, unit: 'ns' };
    }
}

// Update stats display
function updateStats() {
    document.getElementById('bench-count').textContent =
        `${state.benchmarks.length} benchmarks`;
    document.getElementById('bench-completed').textContent =
        `${state.results.size} completed`;
}

// Render results panel (no-op, results shown inline in table)
function renderResults() {
    // Results are displayed inline in the benchmark table
}

// Run a single benchmark
async function runSingleBenchmark(id, warmup, measurement) {
    const simdLevel = document.getElementById('simd-level').value;

    if (state.executionMode === 'native' && state.isTauri) {
        return await invoke('run_benchmark', {
            id: id,
            simdLevel: simdLevel,
            warmupMs: warmup,
            measurementMs: measurement,
        });
    } else if (state.wasmModule) {
        // WASM benchmarks run synchronously but we wrap in a promise
        // to allow UI updates between benchmarks
        return new Promise((resolve) => {
            // Use setTimeout to allow UI to update
            setTimeout(() => {
                const result = state.wasmModule.run_benchmark(id, BigInt(warmup), BigInt(measurement));
                resolve(result);
            }, 10);
        });
    }
    return null;
}

// Abort running benchmarks
function abortBenchmarks() {
    if (state.isRunning) {
        console.log('Abort requested');
        state.abortRequested = true;
    }
}

// Run benchmarks
async function runBenchmarks(ids) {
    if (state.isRunning || ids.length === 0) {
        console.log('Cannot run:', state.isRunning ? 'already running' : 'no benchmarks selected');
        return;
    }

    console.log('Running benchmarks:', ids, 'mode:', state.executionMode);
    state.isRunning = true;
    state.abortRequested = false;

    // Mark all as queued initially
    for (const id of ids) {
        state.queuedBenchmarks.add(id);
    }
    renderBenchmarks();
    updateRunButtons();

    const warmup = 1500;     // Hardcoded warmup time in ms
    const measurement = 4000; // Hardcoded measurement time in ms

    for (const id of ids) {
        // Check for abort
        if (state.abortRequested) {
            console.log('Benchmarks aborted');
            break;
        }

        // Move from queued to running (warmup phase first)
        state.queuedBenchmarks.delete(id);
        state.runningBenchmark = id;
        state.runningPhase = 'warmup';
        renderBenchmarks();

        // Set timer to transition to measuring phase
        const phaseTimer = setTimeout(() => {
            state.runningPhase = 'measuring';
            renderBenchmarks();
        }, warmup);

        // Allow UI to update
        await new Promise(resolve => setTimeout(resolve, 0));

        try {
            console.log('Running benchmark:', id);
            const result = await runSingleBenchmark(id, warmup, measurement);
            console.log('Result:', result);

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
        renderResults();
        updateStats();
    }

    state.isRunning = false;
    state.abortRequested = false;
    state.queuedBenchmarks.clear();
    // Keep selections after running
    renderBenchmarks();
    updateRunButtons();
}

// Update run/abort button visibility and state
function updateRunButtons() {
    const runBtn = document.getElementById('run-btn');
    const abortBtn = document.getElementById('abort-btn');

    if (state.isRunning) {
        runBtn.style.display = 'none';
        if (abortBtn) abortBtn.style.display = 'inline-block';
    } else {
        runBtn.style.display = 'inline-block';
        runBtn.disabled = state.selectedBenchmarks.length === 0;
        if (abortBtn) abortBtn.style.display = 'none';
    }
}

// Export results as JSON
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

// Set up event listeners
function setupEventListeners() {
    // Execution mode change
    document.getElementById('exec-mode').addEventListener('change', async (e) => {
        state.executionMode = e.target.value;
        console.log('Execution mode changed to:', state.executionMode);

        // Keep results when switching modes

        // Reload platform info, SIMD levels, and benchmarks for new mode
        await loadPlatformInfo();
        await loadSimdLevels();
        await loadBenchmarks();
    });

    // SIMD level change
    document.getElementById('simd-level').addEventListener('change', async (e) => {
        const level = e.target.value;
        console.log('SIMD level changed to:', level);

        if (state.executionMode === 'wasm') {
            // For WASM, we need to load a different module
            await switchWasmSimdLevel(level);
        }
        // For native mode, the SIMD level is just stored and used when running benchmarks
    });

    // Category selection and tree toggle
    document.getElementById('category-list').addEventListener('click', (e) => {
        // Check if clicking on toggle arrow
        const toggle = e.target.closest('.tree-toggle');
        if (toggle) {
            const category = toggle.dataset.toggle;
            if (state.expandedCategories.has(category)) {
                state.expandedCategories.delete(category);
            } else {
                state.expandedCategories.add(category);
            }
            // Re-render categories to show/hide children
            const categories = new Set(['all']);
            state.benchmarks.forEach(b => {
                if (b.category) categories.add(b.category);
            });
            renderCategories(Array.from(categories));
            return;
        }

        const item = e.target.closest('.category-item');
        if (!item) return;

        const category = item.dataset.category;
        state.currentCategory = category;

        // Auto-expand when selecting a parent category
        if (category !== 'all') {
            state.expandedCategories.add(category);
        }

        document.getElementById('current-category').textContent =
            category === 'all' ? 'All Benchmarks' : category;

        // Re-render categories and benchmarks
        const categories = new Set(['all']);
        state.benchmarks.forEach(b => {
            if (b.category) categories.add(b.category);
        });
        renderCategories(Array.from(categories));
        renderBenchmarks();
    });

    // Select all checkbox
    document.getElementById('select-all').addEventListener('change', (e) => {
        if (state.isRunning) {
            e.target.checked = !e.target.checked; // Revert
            return;
        }

        const filtered = state.currentCategory === 'all'
            ? state.benchmarks
            : state.benchmarks.filter(b => b.category === state.currentCategory ||
                b.category.startsWith(state.currentCategory + '/'));

        if (e.target.checked) {
            // Select all visible benchmarks (add ones not already selected)
            for (const b of filtered) {
                if (!state.selectedBenchmarks.includes(b.id)) {
                    state.selectedBenchmarks.push(b.id);
                }
            }
        } else {
            // Deselect all visible benchmarks
            const visibleIds = new Set(filtered.map(b => b.id));
            state.selectedBenchmarks = state.selectedBenchmarks.filter(id => !visibleIds.has(id));
        }

        renderBenchmarks();
        updateRunButtons();
    });

    // Benchmark table row selection
    document.getElementById('benchmark-tbody').addEventListener('click', (e) => {
        if (state.isRunning) return;

        const row = e.target.closest('tr');
        if (!row) return;

        const id = row.dataset.id;
        const index = state.selectedBenchmarks.indexOf(id);

        if (index >= 0) {
            // Remove from selection
            state.selectedBenchmarks.splice(index, 1);
        } else {
            // Add to selection (preserves order)
            state.selectedBenchmarks.push(id);
        }

        renderBenchmarks();
        updateRunButtons();
    });

    // Run button - runs selected if any, otherwise all visible (in listed order)
    document.getElementById('run-btn').addEventListener('click', () => {
        // Get visible benchmarks in listed order
        const visible = state.benchmarks
            .filter(b => state.currentCategory === 'all' ||
                b.category === state.currentCategory ||
                b.category.startsWith(state.currentCategory + '/'));

        let ids;
        if (state.selectedBenchmarks.length > 0) {
            // Run selected benchmarks in listed order (not selection order)
            const selectedSet = new Set(state.selectedBenchmarks);
            ids = visible.filter(b => selectedSet.has(b.id)).map(b => b.id);
        } else {
            // Run all visible benchmarks
            ids = visible.map(b => b.id);
        }
        console.log('Run clicked, ids:', ids);
        runBenchmarks(ids);
    });
    // Abort button
    document.getElementById('abort-btn').addEventListener('click', abortBenchmarks);

    // Export button
    document.getElementById('export-results').addEventListener('click', exportResults);
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', init);
