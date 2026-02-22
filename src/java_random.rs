/// 精确复现 Java `java.util.Random` 的线性同余生成器 (LCG)
///
/// Java LCG 参数:
/// - multiplier: 0x5DEECE66D
/// - increment: 0xB
/// - modulus: 2^48
pub struct JavaRandom {
    seed: i64,
}

const MULTIPLIER: i64 = 0x5DEECE66D;
const INCREMENT: i64 = 0xB;
const MASK: i64 = (1_i64 << 48) - 1;

impl JavaRandom {
    /// 创建新的 JavaRandom，与 Java 的 `new Random(seed)` 行为一致
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ MULTIPLIER) & MASK,
        }
    }

    /// 内部 LCG 步进，返回高 `bits` 位（与 Java `next(int bits)` 一致）
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT)) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    /// 与 Java 的 `nextInt(int bound)` 完全一致
    ///
    /// 注意：Java 的实现对 2 的幂次有特殊处理，且包含拒绝采样逻辑
    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");

        // 如果 bound 是 2 的幂
        if (bound & (bound - 1)) == 0 {
            return ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }

        // 拒绝采样，避免模偏差
        loop {
            let bits = self.next(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound - 1) >= 0 {
                return val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_random_seed_12345() {
        let mut rng = JavaRandom::new(12345);
        let expected = [1, 0, 1, 8, 5, 4, 5, 2, 1, 9];
        for &exp in &expected {
            assert_eq!(rng.next_int(10), exp);
        }
    }

    #[test]
    fn test_java_random_seed_0() {
        let mut rng = JavaRandom::new(0);
        let expected = [0, 8, 9, 7, 5];
        for &exp in &expected {
            assert_eq!(rng.next_int(10), exp);
        }
    }

    #[test]
    fn test_java_random_seed_neg1() {
        let mut rng = JavaRandom::new(-1);
        let expected = [3, 5, 9, 9, 4];
        for &exp in &expected {
            assert_eq!(rng.next_int(10), exp);
        }
    }

    #[test]
    fn test_power_of_two_bound() {
        let mut rng = JavaRandom::new(42);
        let expected = [11, 0, 10, 0, 4];
        for &exp in &expected {
            assert_eq!(rng.next_int(16), exp);
        }
    }

    #[test]
    fn test_next_int_deterministic() {
        // 同一种子应产生相同序列
        let mut rng1 = JavaRandom::new(999);
        let mut rng2 = JavaRandom::new(999);
        for _ in 0..100 {
            assert_eq!(rng1.next_int(100), rng2.next_int(100));
        }
    }
}