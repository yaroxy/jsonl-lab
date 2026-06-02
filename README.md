# jsonl-lab

`jsonl-lab` is a fast JSONL inspection tool for large datasets. It provides a Rust CLI, random-access indexing, an HTTP API, and a React Web Viewer.

The goal is to quickly locate, sample, inspect, and browse large JSONL files without opening the full file in an editor.

```bash
jsonl-lab index data.jsonl
jsonl-lab get data.jsonl --idx 123
jsonl-lab range data.jsonl --start 100 --limit 20
jsonl-lab inspect data.jsonl
jsonl-lab serve data.jsonl --port 7860
```

## Features

- Build line-offset indexes for JSONL files and access records by row index.
- Read large files with mmap-backed random access instead of loading the full file.
- Read single records, bounded ranges, and sampled structure summaries.
- Choose between `serde` and `simd` JSON parsers with `--parser`.
- Format output as pretty JSON, compact JSON, or raw JSONL with `--format`.
- Write output to files with `--output`.
- Serve JSONL records through a lightweight Axum HTTP API with compression.
- Browse records in a React Web Viewer with index navigation and pretty JSON display.

## Quick Start

Build the release binary:

```bash
cargo build --release
```

Create a small demo file:

```bash
cat > demo.jsonl <<'EOF'
{"id":1,"messages":[{"role":"user","content":"hello"}],"answer":"hi"}
{"id":2,"messages":[{"role":"user","content":"1+1"}],"answer":"2"}
{"id":3,"messages":[{"role":"user","content":"bye"}],"answer":"goodbye"}
EOF
```

Index and read the data:

```bash
./target/release/jsonl-lab index demo.jsonl
./target/release/jsonl-lab get demo.jsonl --idx 1
./target/release/jsonl-lab range demo.jsonl --start 0 --limit 2
./target/release/jsonl-lab inspect demo.jsonl --sample 3
```

Start the HTTP API:

```bash
./target/release/jsonl-lab serve demo.jsonl --port 7860
```

In another terminal, start the Web Viewer:

```bash
npm install --prefix web/viewer
npm run dev --prefix web/viewer
```

Open:

```text
http://127.0.0.1:5173
```

## Install

During development, run the CLI through Cargo:

```bash
cargo run -p jsonl-cli -- --help
```

To make `jsonl-lab` available from any directory, install the CLI into Cargo's bin directory:

```bash
cargo install --path crates/jsonl-cli --force
```

Verify the installation:

```bash
jsonl-lab --help
```

## CLI Usage

### index

Build an index for a JSONL file:

```bash
jsonl-lab index data.jsonl
```

By default, this creates:

```text
data.jsonl.idx
```

Use a custom index output path:

```bash
jsonl-lab index data.jsonl --output data.idx
```

### get

Read one record by row index. Pretty JSON output is the default:

```bash
jsonl-lab get data.jsonl --idx 123
```

Print the original JSONL line:

```bash
jsonl-lab get data.jsonl --idx 123 --format raw
```

Write to a file:

```bash
jsonl-lab get data.jsonl --idx 123 --output item.json
```

Use the SIMD parser:

```bash
jsonl-lab get data.jsonl --idx 123 --parser simd
```

Use a custom index file:

```bash
jsonl-lab get data.jsonl --idx 123 --index data.idx
```

### range

Read a bounded range of records. Pretty JSON array is the default:

```bash
jsonl-lab range data.jsonl --start 100 --limit 20
```

Print raw JSONL lines:

```bash
jsonl-lab range data.jsonl --start 100 --limit 20 --format jsonl
```

Write to a file:

```bash
jsonl-lab range data.jsonl --start 100 --limit 20 --output rows.json
```

### inspect

Sample records and summarize top-level JSON structure and field types:

```bash
jsonl-lab inspect data.jsonl
```

Set the sample start and size:

```bash
jsonl-lab inspect data.jsonl --start 10000 --sample 1000
```

Print the report as JSON:

```bash
jsonl-lab inspect data.jsonl --format json
```

Write to a file:

```bash
jsonl-lab inspect data.jsonl --output inspect.json
```

Use the SIMD parser:

```bash
jsonl-lab inspect data.jsonl --parser simd
```

### serve

Start the HTTP API:

```bash
jsonl-lab serve data.jsonl --host 127.0.0.1 --port 7860
```

Allow access from other machines on the local network:

```bash
jsonl-lab serve data.jsonl --host 0.0.0.0 --port 7860
```

Use the SIMD parser for all API requests:

```bash
jsonl-lab serve data.jsonl --parser simd
```

## Index Validity

The index stores the source JSONL file size and modified timestamp.

If the JSONL file changes, rebuild the index:

```bash
jsonl-lab index data.jsonl
```

If a stale index is used, `get`, `range`, `inspect`, and `serve` fail with an actionable error that asks you to rerun `jsonl-lab index`.

This project is in active early development and does not preserve historical compatibility. When index formats, CLI behavior, or API design change, the current design takes precedence and old index files may become invalid.

## HTTP API

Start the server:

```bash
jsonl-lab serve data.jsonl --port 7860
```

Example requests:

```bash
curl http://127.0.0.1:7860/api/meta
curl http://127.0.0.1:7860/api/item/1
curl 'http://127.0.0.1:7860/api/range?start=0&limit=2'
curl 'http://127.0.0.1:7860/api/range-preview?start=0&limit=20&max_bytes=256'
```

API overview:

| Method | Path | Description |
| ------ | ---- | ----------- |
| GET | `/api/meta` | Dataset metadata |
| GET | `/api/item/{idx}` | Parsed JSON record by row index |
| GET | `/api/range?start=0&limit=20` | Parsed JSON records in a bounded range |
| GET | `/api/range-preview?start=0&limit=20&max_bytes=256` | Lightweight preview rows (truncated raw text) |

The server enables HTTP compression (gzip/brotli) for all responses.

## Web Viewer

The Web Viewer lives in `web/viewer` and uses Vite, React, and TypeScript.

Start the Rust backend first:

```bash
jsonl-lab serve data.jsonl --port 7860
```

Then start the frontend development server:

```bash
npm install --prefix web/viewer
npm run dev --prefix web/viewer
```

Open:

```text
http://127.0.0.1:5173
```

The Vite development server proxies `/api/*` to:

```text
http://127.0.0.1:7860
```

Build the frontend:

```bash
npm run build --prefix web/viewer
```

Build output is written to:

```text
web/viewer/dist
```

## Development

Common verification commands:

```bash
cargo fmt --all
cargo build
cargo test
npm run build --prefix web/viewer
```

Project layout:

```text
jsonl-lab/
  Cargo.toml
  crates/
    jsonl-core/
    jsonl-cli/
    jsonl-server/
  web/
    viewer/
  dev/
    design/
```
