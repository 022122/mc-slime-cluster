pub mod java_random;
pub mod slime;
pub mod search;
pub mod types;

#[cfg(feature = "wasm")]
mod wasm_api {
    use wasm_bindgen::prelude::*;
    use crate::search::search;
    use crate::types::SearchParams;

    /// WASM 导出：搜索史莱姆区块多联结构
    ///
    /// 接收 JSON 字符串参数，返回 JSON 字符串结果
    ///
    /// 参数格式:
    /// ```json
    /// {
    ///   "seed": "12345",
    ///   "origin_x": 0,
    ///   "origin_z": 0,
    ///   "search_radius": 100,
    ///   "pattern_w": 3,
    ///   "pattern_h": 3,
    ///   "top_n": 10
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn search_slime_chunks(params_json: &str) -> String {
        let params: SearchParams = match serde_json::from_str(params_json) {
            Ok(p) => p,
            Err(e) => {
                return serde_json::json!({
                    "error": format!("Invalid params: {}", e)
                }).to_string();
            }
        };

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
    #[wasm_bindgen]
    pub fn is_slime_chunk(seed_hi: i32, seed_lo: i32, chunk_x: i32, chunk_z: i32) -> bool {
        // JS 无法直接传 i64，用两个 i32 拼接
        let seed = ((seed_hi as i64) << 32) | (seed_lo as u32 as i64);
        crate::slime::is_slime_chunk(seed, chunk_x, chunk_z)
    }
}