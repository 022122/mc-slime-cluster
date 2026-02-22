use clap::Parser;
use slime_search::search::search;
use slime_search::types::SearchParams;
use std::time::Instant;

/// Minecraft 多联结构史莱姆区块搜索工具
#[derive(Parser, Debug)]
#[command(name = "slime-search")]
#[command(about = "搜索 Minecraft 多联结构史莱姆区块群")]
struct Cli {
    /// 世界种子
    #[arg(short, long)]
    seed: i64,

    /// 原点区块 X 坐标
    #[arg(long, default_value_t = 0)]
    ox: i32,

    /// 原点区块 Z 坐标
    #[arg(long, default_value_t = 0)]
    oz: i32,

    /// 搜索半径（区块数）
    #[arg(short, long, default_value_t = 100)]
    radius: i32,

    /// 多联结构宽度（区块数）
    #[arg(short, long)]
    width: u32,

    /// 多联结构高度（区块数）
    #[arg(long)]
    height: u32,

    /// 返回结果数量
    #[arg(short, long, default_value_t = 10)]
    top: usize,

    /// 以 JSON 格式输出
    #[arg(long, default_value_t = false)]
    json: bool,

    /// 自定义图案掩码，用 X 和 . 表示（行用 / 分隔）
    /// 例如 3x3 十字形: ".X./XXX/.X."
    #[arg(long)]
    pattern: Option<String>,
}

fn parse_pattern(s: &str, w: u32, h: u32) -> Option<Vec<bool>> {
    let rows: Vec<&str> = s.split('/').collect();
    if rows.len() != h as usize { return None; }
    let mut mask = Vec::with_capacity((w * h) as usize);
    for row in &rows {
        let chars: Vec<char> = row.chars().collect();
        if chars.len() != w as usize { return None; }
        for &c in &chars {
            mask.push(matches!(c, 'X' | 'x' | '1'));
        }
    }
    Some(mask)
}

fn main() {
    let cli = Cli::parse();

    let pattern_mask = cli.pattern.as_ref().and_then(|p| {
        let mask = parse_pattern(p, cli.width, cli.height);
        if mask.is_none() {
            eprintln!("警告: 图案格式不匹配 {}x{}，忽略", cli.width, cli.height);
        }
        mask
    });

    let params = SearchParams {
        seed: cli.seed,
        origin_x: cli.ox,
        origin_z: cli.oz,
        search_radius: cli.radius,
        pattern_w: cli.width,
        pattern_h: cli.height,
        top_n: cli.top,
        pattern_mask,
    };

    let search_area = (params.search_radius * 2 + 1) as u64;
    let total_chunks = search_area * search_area;

    if !cli.json {
        println!("=== Minecraft 史莱姆区块多联结构搜索 ===");
        println!("种子: {}", params.seed);
        println!("原点: ({}, {})", params.origin_x, params.origin_z);
        println!("搜索半径: {} 区块", params.search_radius);
        println!("搜索范围: {}x{} = {} 区块", search_area, search_area, total_chunks);
        println!("目标结构: {}x{} ({} 区块)", params.pattern_w, params.pattern_h, params.pattern_w * params.pattern_h);
        println!("返回数量: {}", params.top_n);
        println!();
    }

    let start = Instant::now();
    let results = search(&params);
    let elapsed = start.elapsed();

    if cli.json {
        let output = serde_json::json!({
            "params": {
                "seed": params.seed,
                "origin_x": params.origin_x,
                "origin_z": params.origin_z,
                "search_radius": params.search_radius,
                "pattern_w": params.pattern_w,
                "pattern_h": params.pattern_h,
            },
            "elapsed_ms": elapsed.as_millis(),
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("搜索完成，耗时: {:.2?}", elapsed);
        println!();

        if results.is_empty() {
            println!("未找到任何结果。");
        } else {
            let target = params.pattern_w * params.pattern_h;
            println!("{:<6} {:<16} {:<12} {}", "排名", "区块坐标", "匹配数", "匹配率");
            println!("{}", "-".repeat(50));

            for (i, r) in results.iter().enumerate() {
                let perfect = if r.matched == target { " ★ 完美匹配" } else { "" };
                println!(
                    "{:<6} ({:>6}, {:>6})  {}/{:<8} {:.1}%{}",
                    i + 1,
                    r.chunk_x,
                    r.chunk_z,
                    r.matched,
                    r.total,
                    r.matched as f64 / r.total as f64 * 100.0,
                    perfect,
                );
            }
        }
    }
}
