/* tslint:disable */
/* eslint-disable */

/**
 * WASM 导出：获取指定区域的史莱姆区块位图
 *
 * 返回一个 Uint8Array，每个字节 0 或 1，按行优先排列
 * 用于前端 Canvas 绘制地图
 */
export function get_slime_bitmap(seed_hi: number, seed_lo: number, cx_min: number, cz_min: number, width: number, height: number): Uint8Array;

/**
 * 初始化 panic hook，让 WASM 中的 panic 显示为可读的错误信息
 */
export function init(): void;

/**
 * WASM 导出：判定单个区块是否为史莱姆区块
 * JS 无法直接传 i64，用两个 i32 拼接
 */
export function is_slime_chunk(seed_hi: number, seed_lo: number, chunk_x: number, chunk_z: number): boolean;

/**
 * WASM 导出：搜索史莱姆区块多联结构
 */
export function search_slime_chunks(params_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_slime_bitmap: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly init: () => void;
    readonly is_slime_chunk: (a: number, b: number, c: number, d: number) => number;
    readonly search_slime_chunks: (a: number, b: number) => [number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
