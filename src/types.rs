use serde::{Deserialize, Serialize};

/// 搜索参数
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// 世界种子
    pub seed: i64,
    /// 原点区块 X
    pub origin_x: i32,
    /// 原点区块 Z
    pub origin_z: i32,
    /// 搜索半径（区块数）
    pub search_radius: i32,
    /// 多联结构宽度（区块数）
    pub pattern_w: u32,
    /// 多联结构高度（区块数）
    pub pattern_h: u32,
    /// 返回结果数量，默认 10
    pub top_n: usize,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// 多联结构左上角区块 X
    pub chunk_x: i32,
    /// 多联结构左上角区块 Z
    pub chunk_z: i32,
    /// 匹配的史莱姆区块数量
    pub matched: u32,
    /// 多联结构总区块数
    pub total: u32,
}