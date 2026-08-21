# prism

SERP research you can actually explain in a meeting.

Not a rank tracker SaaS. Not a scraping product. A small pipeline for public ranking snapshots, feature extraction, and competitor maps, with the methodology written down so nobody has to guess what the numbers mean.

## what it does

1. capture a ranking snapshot (offline sample provider ships with the repo)
2. extract boring, checkable features (title tokens, path depth, rich bits)
3. diff two snapshots and say who moved up, who dropped, who is new
4. list stored runs (`history`) and first-vs-last shifts (`report`)
5. roll snapshots into a competitor map across queries

Public data only. If a source is not clearly public and permitted, it is not in here.

## quick start

```bash
cargo test --workspace
cargo build -p prism-cli
./target/debug/prism capture --query "rust async" --provider sample --out /tmp/a.json
./target/debug/prism features --snapshot /tmp/a.json
./target/debug/prism diff --before data/samples/before.json --after data/samples/after.json
./target/debug/prism history --store-dir data/samples
./target/debug/prism report --store-dir data/samples
```

## crates

prism-core, prism-collect, prism-parse, prism-features, prism-store, prism-diff, prism-analyze, prism-cli

## honesty

the sample provider is offline and deterministic. live HTML parsers will break. that is fine. the rank math is the part that has to stay true.

## license

mit.
