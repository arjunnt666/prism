use clap::{Parser, Subcommand};
use prism_analyze::{competitor_map, history_rows, query_shifts};
use prism_collect::provider_by_name;
use prism_core::{Device, Query};
use prism_diff::{diff_snapshots, summarize};
use prism_features::extract_snapshot;
use prism_store::{load_snapshot_file, save_snapshot_file, JsonDirStore, SnapshotStore};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prism", about = "SERP research pipeline - public data only")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Capture {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "sample")]
        provider: String,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        store_dir: Option<PathBuf>,
        #[arg(long, default_value = "desktop")]
        device: String,
    },
    Diff {
        #[arg(long)]
        before: PathBuf,
        #[arg(long)]
        after: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Features {
        #[arg(long)]
        snapshot: PathBuf,
    },
    Competitors {
        #[arg(long)]
        store_dir: PathBuf,
        #[arg(long, default_value_t = 20)]
        top: usize,
    },
    /// List stored snapshots in capture order.
    History {
        #[arg(long)]
        store_dir: PathBuf,
    },
    /// First vs last rank positions per query, plus the competitor map.
    Report {
        #[arg(long)]
        store_dir: PathBuf,
        #[arg(long)]
        query: Option<String>,
    },
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Capture {
            query,
            provider,
            out,
            store_dir,
            device,
        } => {
            let device = match device.as_str() {
                "mobile" => Device::Mobile,
                "tablet" => Device::Tablet,
                _ => Device::Desktop,
            };
            let q = Query {
                text: query.clone(),
                locale: Some("en-US".into()),
                device,
            };
            let p = provider_by_name(&provider)?;
            let snap = p.capture(&q).await?;
            println!(
                "captured id={} query={:?} results={}",
                snap.id,
                snap.query.text,
                snap.results.len()
            );
            let had_store = store_dir.is_some();
            if let Some(dir) = store_dir {
                JsonDirStore::open(dir)?.put(&snap)?;
                println!("stored in json dir");
            }
            if let Some(path) = out {
                save_snapshot_file(&path, &snap)?;
                println!("wrote {}", path.display());
            } else if !had_store {
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
                    println!(
                        "  {:<20} {:>3} -> {:>3}  {:?}",
                        d.domain,
                        d.before.map(|x| x.to_string()).unwrap_or_else(|| "-".into()),
                        d.after.map(|x| x.to_string()).unwrap_or_else(|| "-".into()),
                        d.kind
                    );
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
                println!(
                    "{:<24} present={:<3} avg={:.1} best={} worst={}",
                    row.domain,
                    row.queries_present,
                    row.avg_position,
                    row.best_position,
                    row.worst_position
                );
            }
        }
        Commands::History { store_dir } => {
            let snaps = JsonDirStore::open(store_dir)?.list_all()?;
            anyhow::ensure!(!snaps.is_empty(), "no snapshots in store");
            for row in history_rows(&snaps) {
                println!(
                    "{}  {}  results={}  top={}  {}",
                    row.captured_at,
                    row.id,
                    row.results,
                    row.top_domain.as_deref().unwrap_or("-"),
                    row.query
                );
            }
            println!("count={}", snaps.len());
        }
        Commands::Report { store_dir, query } => {
            let snaps = JsonDirStore::open(store_dir)?.list_all()?;
            anyhow::ensure!(!snaps.is_empty(), "no snapshots in store");
            let snaps: Vec<_> = match query {
                Some(q) => snaps
                    .into_iter()
                    .filter(|s| s.query.text == q)
                    .collect(),
                None => snaps,
            };
            anyhow::ensure!(!snaps.is_empty(), "no snapshots matched");
            let map = competitor_map(&snaps);
            println!("snapshots={} queries={}", snaps.len(), map.queries.len());
            let mut any_move = false;
            for shift in query_shifts(&snaps) {
                if shift.movers.is_empty() {
                    println!("query {:?} snapshots={} (no position changes)", shift.query, shift.snapshots);
                    continue;
                }
                any_move = true;
                println!(
                    "query {:?} snapshots={} movers={}",
                    shift.query,
                    shift.snapshots,
                    shift.movers.len()
                );
                for (domain, before, after) in shift.movers {
                    println!(
                        "  {:<20} {:>3} -> {:>3}",
                        domain,
                        before.map(|x| x.to_string()).unwrap_or_else(|| "-".into()),
                        after.map(|x| x.to_string()).unwrap_or_else(|| "-".into())
                    );
                }
            }
            anyhow::ensure!(
                any_move || snaps.len() < 2,
                "expected at least one rank change across stored runs"
            );
        }
        Commands::Version => println!("prism 0.1.0"),
    }
    Ok(())
}
