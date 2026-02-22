# 🟢 mc-slime-cluster

Minecraft slime chunk cluster finder — search for multi-chunk slime formations by seed.

Rust + WASM, blazingly fast.

## 功能

- 🔍 在指定种子和范围内搜索多联结构史莱姆区块群（如 3×3、4×2 等）
- 📊 返回 Top-N 最佳匹配结果，按匹配度降序排列
- ⚡ 极速搜索：半径 100 区块（4 万区块）< 1ms，半径 1000（400 万区块）~27ms
- 🎮 适用于 Minecraft Java Edition 全版本（1.0 ~ 1.21+）
- 📦 支持 JSON 输出，方便程序化调用

## 安装

### 从源码构建

```bash
git clone https://github.com/yourname/mc-slime-cluster.git
cd mc-slime-cluster
cargo build --release
```

构建产物在 `target/release/slime-search`（Linux/macOS）或 `target/release/slime-search.exe`（Windows）。

## 使用

### 基本用法

```bash
# 搜索种子 0，以原点为中心，半径 100 区块，寻找 3×3 史莱姆区块群
slime-search --seed 0 --radius 100 --width 3 --height 3
```

输出：
```
=== Minecraft 史莱姆区块多联结构搜索 ===
种子: 0
原点: (0, 0)
搜索半径: 100 区块
搜索范围: 201x201 = 40401 区块
目标结构: 3x3 (9 区块)
返回数量: 10

搜索完成，耗时: 311.80µs

排名     区块坐标             匹配数          匹配率
--------------------------------------------------
1      (    41,    -53)  6/9        66.7%
2      (    40,    -53)  6/9        66.7%
3      (    39,    -53)  6/9        66.7%
...
```

### 完整参数

```bash
slime-search \
  --seed <SEED>       # 世界种子（必填）
  --ox <X>            # 原点区块 X 坐标（默认 0）
  --oz <Z>            # 原点区块 Z 坐标（默认 0）
  --radius <R>        # 搜索半径，区块数（默认 100）
  --width <W>         # 多联结构宽度（必填）
  --height <H>        # 多联结构高度（必填）
  --top <N>           # 返回结果数量（默认 10）
  --json              # 以 JSON 格式输出
```

### JSON 输出

```bash
slime-search --seed 12345 --radius 1000 --width 3 --height 3 --top 5 --json
```

```json
{
  "elapsed_ms": 27,
  "params": {
    "seed": 12345,
    "origin_x": 0,
    "origin_z": 0,
    "search_radius": 1000,
    "pattern_w": 3,
    "pattern_h": 3
  },
  "results": [
    {
      "chunk_x": -267,
      "chunk_z": -638,
      "matched": 7,
      "total": 9
    }
  ]
}
```

## 技术细节

### 史莱姆区块判定

精确复现 Minecraft Java Edition 的判定算法：

```java
Random rng = new Random(
    seed
    + (long)(chunkX * chunkX * 0x4c1906)
    + (long)(chunkX * 0x5ac0db)
    + (long)(chunkZ * chunkZ) * 0x4307a7L
    + (long)(chunkZ * 0x5f24f)
    ^ 0x3ad8025fL
);
return rng.nextInt(10) == 0;
```

包含完整的 Java `java.util.Random` LCG 实现（48-bit 线性同余生成器）。

### 搜索算法

1. 预计算搜索范围内所有区块的史莱姆状态（位图）
2. 构建二维前缀和数组，实现 O(1) 矩形区域查询
3. 滑动窗口遍历 + 最小堆维护 Top-N 结果

### 性能

| 搜索半径 | 区块数 | 耗时 |
|---------|--------|------|
| 100 | 40,401 | ~0.3ms |
| 500 | 1,002,001 | ~7ms |
| 1,000 | 4,004,001 | ~27ms |
| 5,000 | 100,020,001 | ~700ms |

## 项目结构

```
src/
├── main.rs           # CLI 入口
├── lib.rs            # 库导出
├── java_random.rs    # Java Random LCG 实现
├── slime.rs          # 史莱姆区块判定
├── search.rs         # 搜索算法
└── types.rs          # 公共类型
```

## 路线图

- [x] CLI 可执行文件
- [ ] WASM 绑定
- [ ] Web 前端（纯前端，无需服务器）
- [ ] GitHub Actions 多平台自动构建
- [ ] Bedrock Edition 支持

## License

MIT