use crate::java_random::JavaRandom;

/// 判定指定区块是否为史莱姆区块
///
/// 完全复现 Minecraft Java Edition 的判定逻辑：
/// ```java
/// Random rng = new Random(
///     seed
///     + (long)(chunkX * chunkX * 0x4c1906)
///     + (long)(chunkX * 0x5ac0db)
///     + (long)(chunkZ * chunkZ) * 0x4307a7L
///     + (long)(chunkZ * 0x5f24f)
///     ^ 0x3ad8025fL
/// );
/// return rng.nextInt(10) == 0;
/// ```
///
/// 注意 Java 运算符优先级和类型提升规则：
/// - `chunkX * chunkX * 0x4c1906` 是 int 乘法（32-bit 溢出），然后 cast to long
/// - `(long)(chunkZ * chunkZ) * 0x4307a7L` 是 int 乘法 cast to long，再与 long 相乘
/// - `^ 0x3ad8025fL` 的优先级低于 `+`
pub fn is_slime_chunk(seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    // (long)(chunkX * chunkX * 0x4c1906) — 全部 int 乘法，然后 sign-extend to long
    let t1 = chunk_x.wrapping_mul(chunk_x).wrapping_mul(0x4c1906) as i64;

    // (long)(chunkX * 0x5ac0db) — int 乘法，然后 sign-extend to long
    let t2 = chunk_x.wrapping_mul(0x5ac0db) as i64;

    // (long)(chunkZ * chunkZ) * 0x4307a7L — int 乘法 cast to long，再 long 乘法
    let t3 = (chunk_z.wrapping_mul(chunk_z) as i64).wrapping_mul(0x4307a7_i64);

    // (long)(chunkZ * 0x5f24f) — int 乘法，然后 sign-extend to long
    let t4 = chunk_z.wrapping_mul(0x5f24f) as i64;

    let scrambled_seed = seed
        .wrapping_add(t1)
        .wrapping_add(t2)
        .wrapping_add(t3)
        .wrapping_add(t4)
        ^ 0x3ad8025f_i64;

    let mut rng = JavaRandom::new(scrambled_seed);
    rng.next_int(10) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slime_chunk_ratio() {
        // 史莱姆区块大约占 10%，在大范围内验证比例
        let seed = 12345_i64;
        let mut count = 0;
        let total = 10000;
        let side = 100;
        for x in 0..side {
            for z in 0..side {
                if is_slime_chunk(seed, x, z) {
                    count += 1;
                }
            }
        }
        let ratio = count as f64 / total as f64;
        assert!(
            (0.07..=0.13).contains(&ratio),
            "Slime chunk ratio {:.2}% is outside expected range",
            ratio * 100.0
        );
    }

    #[test]
    fn test_find_slime_chunks_seed_0() {
        // 在 seed=0 附近找到一些史莱姆区块，验证不全为 false
        let mut found = Vec::new();
        for x in -20..20 {
            for z in -20..20 {
                if is_slime_chunk(0, x, z) {
                    found.push((x, z));
                }
            }
        }
        // 1600 个区块中大约应有 ~160 个史莱姆区块
        assert!(
            found.len() > 50 && found.len() < 300,
            "Expected ~160 slime chunks in 1600, got {}",
            found.len()
        );
    }
}