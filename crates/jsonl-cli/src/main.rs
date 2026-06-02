use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use jsonl_core::{
    InspectReport, JsonParser, JsonlDataset, JsonlIndex, build_index, format_bytes, format_count,
    format_duration, format_throughput, inspect_dataset, load_index, save_index, validate_index,
};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "jsonl-lab")]
#[command(about = "High-performance JSONL tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Pretty,
    Compact,
    Raw,
    Jsonl,
    Json,
    Human,
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

        #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,

        #[arg(long)]
        output: Option<PathBuf>,

        #[arg(long, value_parser = parse_parser, default_value = "serde")]
        parser: JsonParser,
    },

    Range {
        path: PathBuf,

        #[arg(long)]
        start: usize,

        #[arg(long, default_value_t = 20)]
        limit: usize,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,

        #[arg(long)]
        output: Option<PathBuf>,

        #[arg(long, value_parser = parse_parser, default_value = "serde")]
        parser: JsonParser,
    },

    Inspect {
        path: PathBuf,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, default_value_t = 1000)]
        sample: usize,

        #[arg(long, default_value_t = 0)]
        start: usize,

        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,

        #[arg(long)]
        output: Option<PathBuf>,

        #[arg(long, value_parser = parse_parser, default_value = "serde")]
        parser: JsonParser,
    },

    Serve {
        path: PathBuf,

        #[arg(long)]
        index: Option<PathBuf>,

        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 7860)]
        port: u16,

        #[arg(long, value_parser = parse_parser, default_value = "serde")]
        parser: JsonParser,
    },
}

fn parse_parser(s: &str) -> Result<JsonParser, String> {
    s.parse::<JsonParser>()
}

fn default_index_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(format!("{}.idx", path.display()))
}

fn load_valid_index(path: &PathBuf, index_path: &PathBuf) -> Result<JsonlIndex> {
    let index = load_index(index_path)?;
    validate_index(path, &index)?;
    Ok(index)
}

fn make_writer(output: Option<&PathBuf>) -> Result<Box<dyn Write>> {
    match output {
        Some(path) => {
            let file = fs::File::create(path)?;
            Ok(Box::new(io::BufWriter::new(file)))
        }
        None => Ok(Box::new(io::stdout().lock())),
    }
}

fn print_inspect_human(report: &InspectReport) {
    println!("file: {}", report.path);
    println!("file_size: {}", format_bytes(report.file_size));
    println!("lines: {}", format_count(report.num_lines as u64));
    println!(
        "sample: start={}, requested={}, actual={}",
        format_count(report.start as u64),
        format_count(report.requested_sample as u64),
        format_count(report.sample_size as u64)
    );
    println!("valid_json: {}", format_count(report.valid_json as u64));
    println!("invalid_json: {}", format_count(report.invalid_json as u64));

    println!();
    println!("top_level:");
    if report.top_level.is_empty() {
        println!("  none");
    } else {
        for (kind, count) in &report.top_level {
            println!("  {}: {}", kind, format_count(*count as u64));
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
                format_count(stats.present as u64),
                format_count(report.sample_size as u64),
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
        .map(|(kind, count)| format!("{}={}", kind, format_count(*count as u64)))
        .collect::<Vec<_>>()
        .join(", ")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Index { path, output } => {
            let output = output.unwrap_or_else(|| default_index_path(&path));
            let start = Instant::now();
            let index = build_index(&path)?;
            save_index(&output, &index)?;
            let elapsed = start.elapsed();

            eprintln!("indexed: {}", path.display());
            eprintln!("file_size: {}", format_bytes(index.meta.data_file_size));
            eprintln!("lines: {}", format_count(index.meta.num_lines));
            eprintln!("elapsed: {}", format_duration(elapsed));
            eprintln!(
                "throughput: {}",
                format_throughput(index.meta.data_file_size, elapsed)
            );
            eprintln!("index: {}", output.display());
        }

        Command::Get {
            path,
            idx,
            index,
            format,
            output,
            parser,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;
            let mut writer = make_writer(output.as_ref())?;

            match format {
                OutputFormat::Raw | OutputFormat::Jsonl => {
                    let line = dataset.raw_line(idx)?;
                    writeln!(writer, "{}", String::from_utf8_lossy(line))?;
                }
                OutputFormat::Compact => {
                    let value = dataset.json_value_with(idx, parser)?;
                    writeln!(writer, "{}", serde_json::to_string(&value)?)?;
                }
                OutputFormat::Pretty | OutputFormat::Json => {
                    let value = dataset.json_value_with(idx, parser)?;
                    writeln!(writer, "{}", serde_json::to_string_pretty(&value)?)?;
                }
                OutputFormat::Human => {
                    let value = dataset.json_value_with(idx, parser)?;
                    writeln!(writer, "{}", serde_json::to_string_pretty(&value)?)?;
                }
            }
        }

        Command::Range {
            path,
            start,
            limit,
            index,
            format,
            output,
            parser,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;
            let mut writer = make_writer(output.as_ref())?;

            match format {
                OutputFormat::Raw | OutputFormat::Jsonl => {
                    for line in dataset.range_raw(start, limit)? {
                        writeln!(writer, "{}", line)?;
                    }
                }
                OutputFormat::Compact => {
                    let end = start.saturating_add(limit).min(dataset.len());
                    let mut rows = Vec::with_capacity(end.saturating_sub(start));
                    for idx in start..end {
                        let value = dataset.json_value_with(idx, parser)?;
                        rows.push(serde_json::json!({"idx": idx, "value": value}));
                    }
                    writeln!(writer, "{}", serde_json::to_string(&rows)?)?;
                }
                OutputFormat::Pretty | OutputFormat::Json => {
                    let end = start.saturating_add(limit).min(dataset.len());
                    let mut rows = Vec::with_capacity(end.saturating_sub(start));
                    for idx in start..end {
                        let value = dataset.json_value_with(idx, parser)?;
                        rows.push(serde_json::json!({"idx": idx, "value": value}));
                    }
                    writeln!(writer, "{}", serde_json::to_string_pretty(&rows)?)?;
                }
                OutputFormat::Human => {
                    let end = start.saturating_add(limit).min(dataset.len());
                    let mut rows = Vec::with_capacity(end.saturating_sub(start));
                    for idx in start..end {
                        let value = dataset.json_value_with(idx, parser)?;
                        rows.push(serde_json::json!({"idx": idx, "value": value}));
                    }
                    writeln!(writer, "{}", serde_json::to_string_pretty(&rows)?)?;
                }
            }
        }

        Command::Inspect {
            path,
            index,
            sample,
            start,
            format,
            output,
            parser,
        } => {
            let index_path = index.unwrap_or_else(|| default_index_path(&path));
            let index = load_valid_index(&path, &index_path)?;
            let dataset = JsonlDataset::open(path, index.offsets)?;
            let inspect_start = Instant::now();
            let report = inspect_dataset(&dataset, start, sample, parser)?;
            let elapsed = inspect_start.elapsed();
            let mut writer = make_writer(output.as_ref())?;

            match format {
                OutputFormat::Human => {
                    print_inspect_human(&report);
                    eprintln!("elapsed: {}", format_duration(elapsed));
                }
                OutputFormat::Json | OutputFormat::Pretty => {
                    writeln!(writer, "{}", serde_json::to_string_pretty(&report)?)?;
                }
                OutputFormat::Compact => {
                    writeln!(writer, "{}", serde_json::to_string(&report)?)?;
                }
                _ => {
                    print_inspect_human(&report);
                    eprintln!("elapsed: {}", format_duration(elapsed));
                }
            }
        }

        Command::Serve {
            path,
            index,
            host,
            port,
            parser,
        } => {
            let index = index.unwrap_or_else(|| default_index_path(&path));
            jsonl_server::serve(path, index, host, port, parser).await?;
        }
    }

    Ok(())
}
