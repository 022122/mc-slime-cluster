use crate::slime::is_slime_chunk;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// Union-Find with path compression and union by rank
struct UnionFind {
    parent: Vec<u32>,
    rank: Vec<u8>,
    size: Vec<u32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n as u32).collect(),
            rank: vec![0; n],
            size: vec![1; n],
        }
    }

    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            self.parent[x as usize] = self.parent[self.parent[x as usize] as usize];
            x = self.parent[x as usize];
        }
        x
    }

    fn union(&mut self, a: u32, b: u32) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return; }
        let (small, big) = if self.rank[ra as usize] < self.rank[rb as usize] {
            (ra, rb)
        } else {
            (rb, ra)
        };
        self.parent[small as usize] = big;
        self.size[big as usize] += self.size[small as usize];
        if self.rank[big as usize] == self.rank[small as usize] {
            self.rank[big as usize] += 1;
        }
    }
}

/// Result of connected component search
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectedResult {
    /// Representative chunk coordinate (leftmost-topmost cell of the component)
    pub chunk_x: i32,
    pub chunk_z: i32,
    /// Number of chunks in this connected component
    pub size: u32,
}

/// Search for the largest connected slime chunk clusters.
///
/// Uses row-by-row union-find: O(width) memory for the previous row's labels,
/// plus the union-find structure which grows with the number of distinct labels.
///
/// Only keeps two rows of slime data in memory at a time.
pub fn search_connected(
    seed: i64,
    origin_x: i32,
    origin_z: i32,
    search_radius: i32,
    top_n: usize,
) -> Vec<ConnectedResult> {
    let x_min = origin_x - search_radius;
    let x_max = origin_x + search_radius;
    let z_min = origin_z - search_radius;
    let z_max = origin_z + search_radius;

    let width = (x_max - x_min + 1) as usize;
    let height = (z_max - z_min + 1) as usize;

    // Label counter (0 = no label)
    let mut next_label: u32 = 1;
    // Previous row labels
    let mut prev_labels = vec![0u32; width];
    // Current row labels
    let mut curr_labels = vec![0u32; width];

    // Union-find (index 0 unused, labels start at 1)
    let mut uf = UnionFind::new(1); // start minimal, grows on demand

    // Track the first (topmost-leftmost) coordinate for each label
    let mut label_coords: Vec<(i32, i32)> = vec![(0, 0)]; // index 0 unused

    // Helper to ensure UF and label_coords have room for `label`
    let ensure_capacity = |uf: &mut UnionFind, label_coords: &mut Vec<(i32, i32)>, label: u32, cx: i32, cz: i32| {
        while uf.parent.len() <= label as usize {
            let n = uf.parent.len() as u32;
            uf.parent.push(n);
            uf.rank.push(0);
            uf.size.push(1);
            label_coords.push((cx, cz));
        }
    };

    for iz in 0..height {
        let cz = z_min + iz as i32;

        // Compute current row
        for ix in 0..width {
            curr_labels[ix] = 0;
            let cx = x_min + ix as i32;
            if !is_slime_chunk(seed, cx, cz) {
                continue;
            }

            let left = if ix > 0 { curr_labels[ix - 1] } else { 0 };
            let above = prev_labels[ix];

            if left == 0 && above == 0 {
                // New label
                let label = next_label;
                next_label += 1;
                ensure_capacity(&mut uf, &mut label_coords, label, cx, cz);
                uf.size[label as usize] = 1;
                label_coords[label as usize] = (cx, cz);
                curr_labels[ix] = label;
            } else if left != 0 && above == 0 {
                curr_labels[ix] = uf.find(left);
                uf.size[curr_labels[ix] as usize] += 1;
            } else if left == 0 && above != 0 {
                curr_labels[ix] = uf.find(above);
                uf.size[curr_labels[ix] as usize] += 1;
            } else {
                // Both neighbors have labels — union them
                let rl = uf.find(left);
                let ra = uf.find(above);
                uf.union(rl, ra);
                let root = uf.find(rl);
                // Only add 1 for the current cell (sizes already merged by union)
                uf.size[root as usize] += 1;
                curr_labels[ix] = root;
            }
        }

        std::mem::swap(&mut prev_labels, &mut curr_labels);
    }

    // Collect top-N components by size
    // Use a set to avoid counting the same root twice
    let mut seen_roots = std::collections::HashSet::new();
    let mut heap: BinaryHeap<Reverse<(u32, i32, i32)>> = BinaryHeap::new();

    for label in 1..next_label {
        let root = uf.find(label);
        if !seen_roots.insert(root) { continue; }
        let size = uf.size[root as usize];
        let (cx, cz) = label_coords[root as usize];

        if heap.len() < top_n {
            heap.push(Reverse((size, cx, cz)));
        } else if let Some(&Reverse((min_size, _, _))) = heap.peek() {
            if size > min_size {
                heap.pop();
                heap.push(Reverse((size, cx, cz)));
            }
        }
    }

    let mut results: Vec<ConnectedResult> = Vec::with_capacity(heap.len());
    while let Some(Reverse((size, cx, cz))) = heap.pop() {
        results.push(ConnectedResult { chunk_x: cx, chunk_z: cz, size });
    }
    results.reverse(); // Descending by size
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_basic() {
        let results = search_connected(0, 0, 0, 50, 5);
        assert!(!results.is_empty());
        // Results should be sorted descending by size
        for i in 1..results.len() {
            assert!(results[i - 1].size >= results[i].size);
        }
    }

    #[test]
    fn test_connected_finds_clusters() {
        let results = search_connected(12345, 0, 0, 100, 3);
        assert!(!results.is_empty());
        // Largest cluster should have at least a few chunks
        assert!(results[0].size >= 2, "Largest cluster: {}", results[0].size);
    }
}