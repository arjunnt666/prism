# prism

SERP research you can actually explain in a meeting.

Not a rank tracker SaaS. Not a scraping product. A small pipeline for public ranking snapshots, feature extraction, and competitor maps, with the methodology written down so nobody has to guess what the numbers mean.

## what it does

1. capture a ranking snapshot (offline sample provider ships with the repo)
2. extract boring, checkable features (title tokens, path depth, rich bits)
3. diff two snapshots and say who moved up, who dropped, who is new
4. roll snapshots into a competitor map across queries

Public data only. If a source is not clearly public and permitted, it is not in here.

## quick start

```bash
cargo build -p prism-cli
./target/debug/prism version
./target/debug/prism capture --query "rust async" --provider sample --out /tmp/a.json
./target/debug/prism features --snapshot /tmp/a.json
./target/debug/prism diff --before data/samples/before.json --after data/samples/after.json
```

## crates

| crate | job |
|-------|-----|
| prism-core | snapshot + result types |
| prism-collect | providers (sample offline) |
| prism-parse | html helpers for public pages |
| prism-features | feature extraction |
| prism-store | memory + json directory store |
| prism-diff | rank movement |
| prism-analyze | competitor maps |
| prism-cli | the tool you actually run |

## honesty

The sample provider is deterministic offline data so CI and demos do not depend on live SERP HTML. Live parsers will break when markup changes. That is expected. Keep methodology notes in docs/methodology.

## license

mit. measure carefully. cite your sources.
