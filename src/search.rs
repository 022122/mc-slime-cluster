use crate::slime::is_slime_chunk;
use crate::types::{SearchParams, SearchResult};
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// 执行多联结构史莱姆区块搜索
///
/// 两种模式：
/// - 无 mask（全填充矩形）：流式滑动窗口 O(W×H)
/// - 有 mask（自定义图案）：流式 + 逐窗口匹配 O(W×H×pw×ph)
pub fn search(params: &SearchParams) -> Vec<SearchResult> {
    if params.pattern_mask.is_some() {
        search_masked(params)
    } else {
        search_full_rect(params)
    }
}

/// 快速路径：全填充矩形，滑动窗口
fn search_full_rect(params: &SearchParams) -> Vec<SearchResult> {
    let x_min = params.origin_x - params.search_radius;
    let x_max = params.origin_x + params.search_radius;
    let z_min = params.origin_z - params.search_radius;
    let z_max = params.origin_z + params.search_radius;

    let width = (x_max - x_min + 1) as usize;
    let height = (z_max - z_min + 1) as usize;
    let pw = params.pattern_w as usize;
    let ph = params.pattern_h as usize;
    let total = params.pattern_w * params.pattern_h;

    if pw > width || ph > height {
        return Vec::new();
    }

    let mut col_sum = vec![0u32; width];
    let mut row_buf = vec![vec![0u32; width]; ph];
    let mut heap: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();
    let collect_n = params.top_n * 8;

    for iz in 0..height {
        let cz = z_min + iz as i32;
        let buf_idx = iz % ph;

        for ix in 0..width {
            let cx = x_min + ix as i32;
            let val = if is_slime_chunk(params.seed, cx, cz) { 1u32 } else { 0 };
            col_sum[ix] -= row_buf[buf_idx][ix];
            row_buf[buf_idx][ix] = val;
            col_sum[ix] += val;
        }

        if iz + 1 < ph { continue; }

        let window_z = z_min + (iz + 1 - ph) as i32;
        let mut window_sum: u32 = col_sum[..pw].iter().sum();
        check_and_push(&mut heap, collect_n, window_sum, x_min, window_z);

        for ix in 1..=(width - pw) {
            window_sum += col_sum[ix + pw - 1];
            window_sum -= col_sum[ix - 1];
            check_and_push(&mut heap, collect_n, window_sum, x_min + ix as i32, window_z);
        }
    }

    collect_results(heap, params, total)
}

/// 自定义图案搜索：流式行缓冲 + 逐窗口掩码匹配
fn search_masked(params: &SearchParams) -> Vec<SearchResult> {
    let x_min = params.origin_x - params.search_radius;
    let x_max = params.origin_x + params.search_radius;
    let z_min = params.origin_z - params.search_radius;
    let z_max = params.origin_z + params.search_radius;

    let width = (x_max - x_min + 1) as usize;
    let height = (z_max - z_min + 1) as usize;
    let pw = params.pattern_w as usize;
    let ph = params.pattern_h as usize;
    let total = (pw * ph) as u32; // 精确匹配：每个格子都要对

    if pw > width || ph > height {
        return Vec::new();
    }

    // 环形行缓冲区
    let mut row_buf = vec![vec![false; width]; ph];
    let mut exact_results: Vec<(i32, i32)> = Vec::new();

    for iz in 0..height {
        let cz = z_min + iz as i32;
        let buf_idx = iz % ph;

        // 填充当前行
        for ix in 0..width {
            let cx = x_min + ix as i32;
            row_buf[buf_idx][ix] = is_slime_chunk(params.seed, cx, cz);
        }

        if iz + 1 < ph { continue; }

        let window_z = z_min + (iz + 1 - ph) as i32;
        // 窗口最老行在 row_buf 中的起始索引
        let base_buf = (iz + 1 - ph) % ph;

        // 遍历所有 x 窗口位置（精确匹配：绿=史莱姆，空=非史莱姆）
        for wx in 0..=(width - pw) {
            let mut matched = 0u32;
            for dz in 0..ph {
                let row_idx = (base_buf + dz) % ph;
                for dx in 0..pw {
                    let is_slime = row_buf[row_idx][wx + dx];
                    let want_slime = params.is_required(dx, dz);
                    if is_slime == want_slime {
                        matched += 1;
                    }
                }
            }
            // 只收集精确匹配的结果
            if matched == total {
                exact_results.push((x_min + wx as i32, window_z));
                if exact_results.len() >= params.top_n {
                    break;
                }
            }
        }
        if exact_results.len() >= params.top_n { break; }
    }

    // 精确匹配模式：直接返回，不需要堆排序
    let mut results: Vec<SearchResult> = Vec::new();
    for &(cx, cz) in &exact_results {
        let overlaps = results.iter().any(|r| {
            let dx = (cx - r.chunk_x).unsigned_abs();
            let dz = (cz - r.chunk_z).unsigned_abs();
            dx < pw as u32 && dz < ph as u32
        });
        if !overlaps {
            results.push(SearchResult {
                chunk_x: cx,
                chunk_z: cz,
                matched: total,
                total,
            });
            if results.len() >= params.top_n { break; }
        }
    }
    results
}

/// 从堆中提取候选并去重
fn collect_results(
    mut heap: BinaryHeap<Reverse<(u32, i32, i32)>>,
    params: &SearchParams,
    total: u32,
) -> Vec<SearchResult> {
    let pw = params.pattern_w as usize;
    let ph = params.pattern_h as usize;

    let mut candidates: Vec<(u32, i32, i32)> = Vec::with_capacity(heap.len());
    while let Some(Reverse(item)) = heap.pop() {
        candidates.push(item);
    }
    candidates.reverse();

    let mut results: Vec<SearchResult> = Vec::new();
    for &(matched, cx, cz) in &candidates {
        let overlaps = results.iter().any(|r| {
            let dx = (cx - r.chunk_x).unsigned_abs();
            let dz = (cz - r.chunk_z).unsigned_abs();
            dx < pw as u32 && dz < ph as u32
        });
        if !overlaps {
            results.push(SearchResult {
                chunk_x: cx,
                chunk_z: cz,
                matched,
                total,
            });
            if results.len() >= params.top_n {
                break;
            }
        }
    }

    results
}

#[inline]
fn check_and_push(
    heap: &mut BinaryHeap<Reverse<(u32, i32, i32)>>,
    top_n: usize,
    matched: u32,
    cx: i32,
    cz: i32,
) {
    if heap.len() < top_n {
        heap.push(Reverse((matched, cx, cz)));
    } else if let Some(&Reverse((min_matched, _, _))) = heap.peek() {
        if matched > min_matched {
            heap.pop();
            heap.push(Reverse((matched, cx, cz)));
        }
    }
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
            pattern_mask: None,
        };
        let results = search(&params);
        assert!(!results.is_empty(), "Should find slime chunks");
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
            pattern_mask: None,
        };
        let results = search(&params);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
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
            pattern_mask: None,
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
            pattern_mask: None,
        };
        let results = search(&params);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_large_radius() {
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 1000,
            pattern_w: 3,
            pattern_h: 3,
            top_n: 5,
            pattern_mask: None,
        };
        let results = search(&params);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_search_masked_cross_exact() {
        // 十字形图案 3x3 — 精确匹配
        // .X.
        // XXX
        // .X.
        let mask = vec![
            false, true, false,
            true,  true, true,
            false, true, false,
        ];
        let params = SearchParams {
            seed: 12345,
            origin_x: 0,
            origin_z: 0,
            search_radius: 200,
            pattern_w: 3,
            pattern_h: 3,
            top_n: 5,
            pattern_mask: Some(mask),
        };
        let results = search(&params);
        // 精确匹配：total = 9（每个格子都要对），matched = 9
        for r in &results {
            assert_eq!(r.total, 9);
            assert_eq!(r.matched, 9);
        }
    }

    #[test]
    fn test_search_masked_all_true_finds_exact() {
        // 全 true 的 2x2 mask = 精确匹配 4 个全是史莱姆
        let mask = vec![true; 4];
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 100,
            pattern_w: 2,
            pattern_h: 2,
            top_n: 5,
            pattern_mask: Some(mask),
        };
        let results = search(&params);
        for r in &results {
            assert_eq!(r.total, 4);
            assert_eq!(r.matched, 4); // 精确匹配
        }
    }
}