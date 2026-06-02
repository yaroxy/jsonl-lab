use anyhow::Result;
use clap::{Parser, Subcommand};
use jsonl_core::{
    InspectReport, JsonlDataset, JsonlIndex, build_index, inspect_dataset, load_index, save_index,
    validate_index,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jsonl-lab")]
#[command(about = "High-performance JSONL tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Index {
        path: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Get {
        path: PathBuf,

        #[arg(long)]
        idx: usize,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, default_value = "pretty")]
        mode: String,
    },

    Range {
        path: PathBuf,

        #[arg(long)]
        start: usize,

        #[arg(long, default_value_t = 20)]
        limit: usize,

        #[arg(long)]
        index: Option<PathBuf>,
    },

    Inspect {
        path: PathBuf,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, default_value_t = 1000)]
        sample: usize,

        #[arg(long, default_value_t = 0)]
        start: usize,

        #[arg(long)]
        json: bool,
    },

    Serve {
        path: PathBuf,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 7860)]
        port: u16,
    },
}

fn default_index_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(format!("{}.idx", path.display()))
}

fn load_valid_index(path: &PathBuf, index_path: &PathBuf) -> Result<JsonlIndex> {
    let index = load_index(index_path)?;
    validate_index(path, &index)?;
    Ok(index)
}

fn print_inspect_report(report: &InspectReport) {
    println!("file: {}", report.path);
    println!("file_size: {} bytes", report.file_size);
    println!("lines: {}", report.num_lines);
    println!(
        "sample: start={}, requested={}, actual={}",
        report.start, report.requested_sample, report.sample_size
    );
    println!("valid_json: {}", report.valid_json);
    println!("invalid_json: {}", report.invalid_json);

    println!();
    println!("top_level:");
    if report.top_level.is_empty() {
        println!("  none");
    } else {
        for (kind, count) in &report.top_level {
            println!("  {}: {}", kind, count);
        }
    }

    println!();
    println!("fields:");
    if report.fields.is_empty() {
        println!("  none");
    } else {
        for (field, stats) in &report.fields {
            println!(
                "  {}: present {}/{}; types: {}",
                field,
                stats.present,
                report.sample_size,
                format_type_counts(&stats.types)
            );
        }
    }
}

fn format_type_counts(types: &BTreeMap<String, usize>) -> String {
    if types.is_empty() {
        return "none".to_string();
    }

    types
        .iter()
        .map(|(kind, count)| format!("{}={}", kind, count))
        .collect::<Vec<_>>()
        .join(", ")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Index { path, output } => {
            let output = output.unwrap_or_else(|| default_index_path(&path));
            let index = build_index(&path)?;
            save_index(&output, &index)?;

            eprintln!("indexed: {}", path.display());
            eprintln!("lines: {}", index.meta.num_lines);
            eprintln!("index: {}", output.display());
        }

        Command::Get {
            path,
            idx,
            index,
            mode,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;

            if mode == "raw" {
                let line = dataset.raw_line(idx)?;
                println!("{}", String::from_utf8_lossy(line));
            } else {
                let value = dataset.json_value(idx)?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        }

        Command::Range {
            path,
            start,
            limit,
            index,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;

            for line in dataset.range_raw(start, limit)? {
                println!("{}", line);
            }
        }

        Command::Inspect {
            path,
            index,
            sample,
            start,
            json,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;
            let report = inspect_dataset(&dataset, start, sample)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_inspect_report(&report);
            }
        }

        Command::Serve {
            path,
            index,
            host,
            port,
        } => {
            let index = index.unwrap_or_else(|| default_index_path(&path));
            jsonl_server::serve(path, index, host, port).await?;
        }
    }

    Ok(())
}
