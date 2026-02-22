use crate::slime::is_slime_chunk;
use crate::types::{SearchParams, SearchResult};
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// 执行多联结构史莱姆区块搜索
///
/// 流式算法（内存 O(width × pattern_h)，不再需要 O(width × height)）：
/// 1. 逐行扫描 Z 方向
/// 2. 维护列累加数组 col_sum[x]（最近 H 行的史莱姆数）
/// 3. 对 col_sum 做宽度 W 的滑动窗口求和
/// 4. 最小堆维护 Top-N
pub fn search(params: &SearchParams) -> Vec<SearchResult> {
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

    // col_sum[x] = 最近 ph 行中，列 x 的史莱姆区块数
    let mut col_sum = vec![0u32; width];

    // 环形缓冲区，存储最近 ph 行的史莱姆数据
    // row_buf[row_idx % ph][x] = 该行该列是否为史莱姆区块
    let mut row_buf = vec![vec![0u32; width]; ph];

    // 最小堆维护 Top-N
    let mut heap: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();

    for iz in 0..height {
        let cz = z_min + iz as i32;
        let buf_idx = iz % ph;

        // 计算当前行的史莱姆数据
        for ix in 0..width {
            let cx = x_min + ix as i32;
            let val = if is_slime_chunk(params.seed, cx, cz) { 1u32 } else { 0 };

            // 从 col_sum 中减去即将被覆盖的旧行数据
            col_sum[ix] -= row_buf[buf_idx][ix];
            // 写入新数据
            row_buf[buf_idx][ix] = val;
            // 加上新行数据
            col_sum[ix] += val;
        }

        // 只有积累了 ph 行后才开始计算窗口
        if iz + 1 < ph {
            continue;
        }

        // 窗口起始 Z 坐标
        let window_z = z_min + (iz + 1 - ph) as i32;

        // 对 col_sum 做宽度 pw 的滑动窗口求和
        let mut window_sum: u32 = col_sum[..pw].iter().sum();

        // 检查第一个窗口位置
        check_and_push(&mut heap, params.top_n, window_sum, x_min, window_z);

        // 滑动窗口
        for ix in 1..=(width - pw) {
            window_sum += col_sum[ix + pw - 1];
            window_sum -= col_sum[ix - 1];
            let window_x = x_min + ix as i32;
            check_and_push(&mut heap, params.top_n, window_sum, window_x, window_z);
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
    results.reverse();
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

    #[test]
    fn test_search_large_radius() {
        // 验证大半径不会 panic（流式算法）
        let params = SearchParams {
            seed: 0,
            origin_x: 0,
            origin_z: 0,
            search_radius: 1000,
            pattern_w: 3,
            pattern_h: 3,
            top_n: 5,
        };
        let results = search(&params);
        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }
}