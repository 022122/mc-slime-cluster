use crate::slime::is_slime_chunk;
use crate::types::{SearchParams, SearchResult};
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// 执行多联结构史莱姆区块搜索
///
/// 算法：
/// 1. 预计算搜索范围内所有区块的史莱姆状态
/// 2. 构建二维前缀和数组
/// 3. 滑动窗口遍历，用最小堆维护 Top-N 结果
pub fn search(params: &SearchParams) -> Vec<SearchResult> {
    let x_min = params.origin_x - params.search_radius;
    let x_max = params.origin_x + params.search_radius;
    let z_min = params.origin_z - params.search_radius;
    let z_max = params.origin_z + params.search_radius;

    let width = (x_max - x_min + 1) as usize;
    let height = (z_max - z_min + 1) as usize;
    let pw_pattern = params.pattern_w as usize;
    let ph_pattern = params.pattern_h as usize;
    let total = params.pattern_w * params.pattern_h;

    if pw_pattern > width || ph_pattern > height {
        return Vec::new();
    }

    // Step 1: 预计算史莱姆位图
    let mut bitmap = vec![0u32; width * height];
    for iz in 0..height {
        for ix in 0..width {
            let cx = x_min + ix as i32;
            let cz = z_min + iz as i32;
            if is_slime_chunk(params.seed, cx, cz) {
                bitmap[iz * width + ix] = 1;
            }
        }
    }

    // Step 2: 构建二维前缀和 (大小 (height+1) x (width+1))
    let pw = width + 1;
    let ph = height + 1;
    let mut prefix = vec![0u32; ph * pw];

    for iz in 1..ph {
        for ix in 1..pw {
            prefix[iz * pw + ix] = bitmap[(iz - 1) * width + (ix - 1)]
                + prefix[(iz - 1) * pw + ix]
                + prefix[iz * pw + (ix - 1)]
                - prefix[(iz - 1) * pw + (ix - 1)];
        }
    }

    // 查询矩形 [x1, x2) x [z1, z2) 内的史莱姆区块数（基于 0-indexed bitmap 坐标）
    let rect_sum = |x1: usize, z1: usize, x2: usize, z2: usize| -> u32 {
        prefix[z2 * pw + x2] + prefix[z1 * pw + x1]
            - prefix[z1 * pw + x2]
            - prefix[z2 * pw + x1]
    };

    // Step 3: 滑动窗口 + 最小堆维护 Top-N
    let mut heap: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();

    let max_ix = width - pw_pattern;
    let max_iz = height - ph_pattern;

    for iz in 0..=max_iz {
        for ix in 0..=max_ix {
            let matched = rect_sum(ix, iz, ix + pw_pattern, iz + ph_pattern);
            let cx = x_min + ix as i32;
            let cz = z_min + iz as i32;

            if heap.len() < params.top_n {
                heap.push(Reverse((matched, cx, cz)));
            } else if let Some(&Reverse((min_matched, _, _))) = heap.peek() {
                if matched > min_matched {
                    heap.pop();
                    heap.push(Reverse((matched, cx, cz)));
                }
            }
        }
    }

    // 从堆中提取结果，按 matched 降序排列
    let mut results: Vec<SearchResult> = Vec::with_capacity(heap.len());
    while let Some(Reverse((matched, cx, cz))) = heap.pop() {
        results.push(SearchResult {
            chunk_x: cx,
            chunk_z: cz,
            matched,
            total,
        });
    }
    // heap.pop() 返回最小的，所以 results 是升序，需要反转
    results.reverse();
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_1x1() {
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 10,
            pattern_w: 1,
            pattern_h: 1,
            top_n: 10,
        };
        let results = search(&params);
        assert!(!results.is_empty(), "Should find slime chunks");
        // 1x1 的完美匹配 = matched 1
        assert_eq!(results[0].matched, 1);
        assert_eq!(results[0].total, 1);
    }

    #[test]
    fn test_search_2x2() {
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 50,
            pattern_w: 2,
            pattern_h: 2,
            top_n: 5,
        };
        let results = search(&params);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
        // 结果应按 matched 降序排列
        for i in 1..results.len() {
            assert!(results[i - 1].matched >= results[i].matched);
        }
    }

    #[test]
    fn test_search_3x3() {
        let params = SearchParams {
            seed: 12345,
            origin_x: 0,
            origin_z: 0,
            search_radius: 100,
            pattern_w: 3,
            pattern_h: 3,
            top_n: 10,
        };
        let results = search(&params);
        assert!(results.len() <= 10);
        for r in &results {
            assert_eq!(r.total, 9);
            assert!(r.matched <= 9);
        }
    }

    #[test]
    fn test_search_empty_when_pattern_too_large() {
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 1,
            pattern_w: 100,
            pattern_h: 100,
            top_n: 10,
        };
        let results = search(&params);
        assert!(results.is_empty());
    }
}