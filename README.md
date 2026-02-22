# Slime Cluster Finder

Search for multi-chunk slime cluster patterns in Minecraft Java Edition worlds.

Built in Rust, compiles to both native CLI and WebAssembly for a pure client-side web app.

**Live Demo:** https://demo.hypersyn.de/slime/

## Features

- Exact replica of Java `java.util.Random` LCG (48-bit, all versions)
- Streaming search algorithm: O(W * pattern_h) memory, handles radius 100k+
- Two search modes:
  - **Full rectangle**: finds areas with the most slime chunks (top-N ranking)
  - **Custom pattern**: exact match against a user-drawn shape (cross, L-shape, etc.)
- CLI with JSON output
- WASM web frontend with canvas map, drag/zoom, pattern editor
- GitHub Actions CI for multi-platform releases

## Web App

The `web/` directory contains a self-contained static site. Serve it with any HTTP server:

```
cd web
python -m http.server 8080
```

Or use the pre-built WASM package from GitHub Releases.

### Pattern Editor

Click "Custom Pattern" in the sidebar to open a grid editor. Paint which cells should be slime (green) and which should not. The search will find locations that exactly match your pattern.

## CLI

```
cargo run --release -- -s 12345 -w 3 --height 3 -r 1000
```

Options:

```
-s, --seed <SEED>        World seed
    --ox <OX>            Origin chunk X (default: 0)
    --oz <OZ>            Origin chunk Z (default: 0)
-r, --radius <RADIUS>    Search radius in chunks (default: 100)
-w, --width <WIDTH>      Pattern width in chunks
    --height <HEIGHT>    Pattern height in chunks
-t, --top <TOP>          Number of results (default: 10)
    --pattern <PATTERN>  Custom pattern mask, rows separated by /
                         X = slime, . = not slime
                         Example: ".X./XXX/.X." (3x3 cross)
    --json               Output as JSON
```

Example with custom pattern:

```
cargo run --release -- -s 12345 -w 3 --height 3 -r 500 --pattern ".X./XXX/.X."
```

## Building

### Native CLI

```
cargo build --release
```

### WASM

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```
wasm-pack build --target web --no-default-features --features wasm
cp -r pkg/* web/pkg/
```

### Running Tests

```
cargo test
```

## How It Works

1. Slime chunk detection uses the same formula as Minecraft: seed + chunk coordinate scrambling through `java.util.Random`, checking `nextInt(10) == 0`.

2. For full-rectangle search, a streaming sliding window scans the search area row by row, maintaining column sums in a circular buffer. A min-heap tracks the top-N candidates, followed by greedy overlap deduplication.

3. For custom pattern search, each window position is checked cell-by-cell against the mask. Green cells must be slime chunks, empty cells must not be. Only exact matches are returned.

## License

MIT