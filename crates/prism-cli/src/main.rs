use clap::{Parser, Subcommand};
use prism_analyze::competitor_map;
use prism_collect::provider_by_name;
use prism_core::{Device, Query};
use prism_diff::{diff_snapshots, summarize};
use prism_features::extract_snapshot;
use prism_store::{load_snapshot_file, save_snapshot_file, JsonDirStore, SnapshotStore};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prism", about = "SERP research pipeline \u2014 public data only")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Capture {
        #[arg(long)] query: String,
        #[arg(long, default_value = "sample")] provider: String,
        #[arg(long)] out: Option<PathBuf>,
        #[arg(long)] store_dir: Option<PathBuf>,
        #[arg(long, default_value = "desktop")] device: String,
    },
    Diff {
        #[arg(long)] before: PathBuf,
        #[arg(long)] after: PathBuf,
        #[arg(long)] json: bool,
    },
    Features { #[arg(long)] snapshot: PathBuf },
    Competitors {
        #[arg(long)] store_dir: PathBuf,
        #[arg(long, default_value_t = 20)] top: usize,
    },
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Capture { query, provider, out, store_dir, device } => {
            let device = match device.as_str() {
                "mobile" => Device::Mobile,
                "tablet" => Device::Tablet,
                _ => Device::Desktop,
            };
            let q = Query { text: query.clone(), locale: Some("en-US".into()), device };
            let p = provider_by_name(&provider)?;
            let snap = p.capture(&q).await?;
            println!("captured id={} query={:?} results={}", snap.id, snap.query.text, snap.results.len());
            if let Some(dir) = store_dir {
                JsonDirStore::open(dir)?.put(&snap)?;
                println!("stored in json dir");
            }
            if let Some(path) = out {
                save_snapshot_file(&path, &snap)?;
                println!("wrote {}", path.display());
            } else if store_dir.is_none() {
                println!("{}", serde_json::to_string_pretty(&snap)?);
            }
        }
        Commands::Diff { before, after, json } => {
            let b = load_snapshot_file(&before)?;
            let a = load_snapshot_file(&after)?;
            let report = diff_snapshots(&b, &a)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("query: {}", report.query.text);
                println!("summary: {:?}", summarize(&report));
                for d in &report.deltas {
                    println!("  {:<20} {:>3} -> {:>3}  {:?}",
                        d.domain,
                        d.before.map(|x| x.to_string()).unwrap_or("-".into()),
                        d.after.map(|x| x.to_string()).unwrap_or("-".into()),
                        d.kind);
                }
            }
        }
        Commands::Features { snapshot } => {
            let snap = load_snapshot_file(&snapshot)?;
            println!("{}", serde_json::to_string_pretty(&extract_snapshot(&snap))?);
        }
        Commands::Competitors { store_dir, top } => {
            let snaps = JsonDirStore::open(store_dir)?.list_all()?;
            let map = competitor_map(&snaps);
            println!("queries: {}", map.queries.len());
            for row in map.rows.iter().take(top) {
                println!("{:<24} present={:<3} avg={:.1} best={} worst={}",
                    row.domain, row.queries_present, row.avg_position, row.best_position, row.worst_position);
            }
        }
        Commands::Version => println!("prism 0.1.0"),
    }
    Ok(())
}
