pub mod java_random;
pub mod slime;
pub mod search;
pub mod types;
pub mod connected;

#[cfg(feature = "wasm")]
mod wasm_api {
    use wasm_bindgen::prelude::*;
    use crate::search::search;
    use crate::types::SearchParams;

    /// 初始化 panic hook，让 WASM 中的 panic 显示为可读的错误信息
    #[wasm_bindgen(start)]
    pub fn init() {
        console_error_panic_hook::set_once();
    }

    /// WASM 导出：搜索史莱姆区块多联结构
    #[wasm_bindgen]
    pub fn search_slime_chunks(params_json: &str) -> String {
        let params: SearchParams = match serde_json::from_str(params_json) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({
                    "error": format!("参数解析失败: {}", e)
                }).to_string();
            }
        };

        // 参数校验
        if params.pattern_w == 0 || params.pattern_h == 0 {
            return serde_json::json!({
                "error": "结构宽度和高度必须大于 0"
            }).to_string();
        }
        if params.search_radius <= 0 {
            return serde_json::json!({
                "error": "搜索半径必须大于 0"
            }).to_string();
        }
        if params.top_n == 0 {
            return serde_json::json!({
                "error": "返回数量必须大于 0"
            }).to_string();
        }

        let results = search(&params);

        serde_json::json!({
            "results": results,
            "params": {
                "seed": params.seed,
                "origin_x": params.origin_x,
                "origin_z": params.origin_z,
                "search_radius": params.search_radius,
                "pattern_w": params.pattern_w,
                "pattern_h": params.pattern_h,
            }
        }).to_string()
    }

    /// WASM 导出：判定单个区块是否为史莱姆区块
    /// JS 无法直接传 i64，用两个 i32 拼接
    #[wasm_bindgen]
    pub fn is_slime_chunk(seed_hi: i32, seed_lo: i32, chunk_x: i32, chunk_z: i32) -> bool {
        let seed = ((seed_hi as i64) << 32) | (seed_lo as u32 as i64);
        crate::slime::is_slime_chunk(seed, chunk_x, chunk_z)
    }

    /// WASM 导出：获取指定区域的史莱姆区块位图
    ///
    /// 返回一个 Uint8Array，每个字节 0 或 1，按行优先排列
    /// 用于前端 Canvas 绘制地图
    /// WASM 导出：搜索最大连通史莱姆区块群
    #[wasm_bindgen]
    pub fn search_connected_chunks(seed_hi: i32, seed_lo: i32, origin_x: i32, origin_z: i32, search_radius: i32, top_n: usize) -> String {
        let seed = ((seed_hi as i64) << 32) | (seed_lo as u32 as i64);
        let results = crate::connected::search_connected(seed, origin_x, origin_z, search_radius, top_n);
        serde_json::json!({ "results": results }).to_string()
    }

    #[wasm_bindgen]
    pub fn get_slime_bitmap(seed_hi: i32, seed_lo: i32, cx_min: i32, cz_min: i32, width: i32, height: i32) -> Vec<u8> {
        let seed = ((seed_hi as i64) << 32) | (seed_lo as u32 as i64);
        let w = width.max(0) as usize;
        let h = height.max(0) as usize;
        let mut bitmap = vec![0u8; w * h];
        for iz in 0..h {
            for ix in 0..w {
                let cx = cx_min + ix as i32;
                let cz = cz_min + iz as i32;
                if crate::slime::is_slime_chunk(seed, cx, cz) {
                    bitmap[iz * w + ix] = 1;
                }
            }
        }
        bitmap
    }
}