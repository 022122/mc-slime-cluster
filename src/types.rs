use serde::{Deserialize, Serialize, Deserializer};

/// 搜索参数
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// 世界种子（支持数字或字符串格式）
    #[serde(deserialize_with = "deserialize_seed")]
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
    /// 自定义图案掩码（可选）
    /// 长度 = pattern_w * pattern_h，行优先，true = 要求是史莱姆区块
    /// 为 None 时等同于全 true（矩形填满）
    #[serde(default)]
    pub pattern_mask: Option<Vec<bool>>,
}

impl SearchParams {
    /// 图案中要求为史莱姆的区块总数
    pub fn required_count(&self) -> u32 {
        match &self.pattern_mask {
            Some(mask) => mask.iter().filter(|&&v| v).count() as u32,
            None => self.pattern_w * self.pattern_h,
        }
    }

    /// 检查 (dx, dz) 是否在图案中被要求
    #[inline]
    pub fn is_required(&self, dx: usize, dz: usize) -> bool {
        match &self.pattern_mask {
            Some(mask) => mask[dz * self.pattern_w as usize + dx],
            None => true,
        }
    }
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

/// 支持从 JSON 数字或字符串反序列化 i64 种子
/// JS Number 只有 53 位精度，大种子需要用字符串传递
fn deserialize_seed<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SeedValue {
        Num(i64),
        Str(String),
    }

    match SeedValue::deserialize(deserializer)? {
        SeedValue::Num(n) => Ok(n),
        SeedValue::Str(s) => s.parse::<i64>().map_err(serde::de::Error::custom),
    }
}