// Web Worker: 在后台线程执行 WASM 搜索
import init, { search_slime_chunks, search_connected_chunks } from './pkg/slime_search.js';

let wasmReady = false;

async function initWasm() {
    await init();
    wasmReady = true;
    self.postMessage({ type: 'ready' });
}

self.onmessage = async function(e) {
    if (e.data.type === 'init') {
        await initWasm();
    } else if (e.data.type === 'search') {
        if (!wasmReady) {
            self.postMessage({ type: 'error', error: 'WASM 未加载' });
            return;
        }

        const params = e.data.params;
        const radius = params.search_radius || 500;
        const totalChunks = (radius * 2 + 1) ** 2;

        self.postMessage({ type: 'progress', progress: 0, status: `搜索 ${totalChunks.toLocaleString()} 个区块...` });

        try {
            const start = performance.now();
            const resultJson = search_slime_chunks(JSON.stringify(params));
            const elapsed = performance.now() - start;
            const data = JSON.parse(resultJson);

            if (data.error) {
                self.postMessage({ type: 'error', error: data.error });
            } else {
                self.postMessage({ type: 'result', results: data.results, elapsed });
            }
        } catch (err) {
            self.postMessage({ type: 'error', error: err.message || String(err) });
        }
    } else if (e.data.type === 'search_connected') {
        if (!wasmReady) {
            self.postMessage({ type: 'error', error: 'WASM 未加载' });
            return;
        }

        const { seedHi, seedLo, originX, originZ, radius, topN } = e.data;
        const totalChunks = (radius * 2 + 1) ** 2;

        self.postMessage({ type: 'progress', progress: 0, status: `搜索连通区块 ${totalChunks.toLocaleString()} ...` });

        try {
            const start = performance.now();
            const resultJson = search_connected_chunks(seedHi, seedLo, originX, originZ, radius, topN);
            const elapsed = performance.now() - start;
            const data = JSON.parse(resultJson);

            self.postMessage({ type: 'result_connected', results: data.results, elapsed });
        } catch (err) {
            self.postMessage({ type: 'error', error: err.message || String(err) });
        }
    }
};