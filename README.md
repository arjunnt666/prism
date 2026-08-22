How I would explain prism if someone asked where the numbers came from

I capture a ranking snapshot. the repo ships an offline sample provider so you can do this without hitting the live web. then I extract boring, checkable features (title tokens, path depth, rich bits). then I diff two snapshots and say who moved up, who dropped, who is new. `history` lists stored runs. `report` is first vs last shifts. those snapshots can roll into a competitor map across queries.

Public data only. if a source is not clearly public and permitted, it is not in here.

The commands I actually use:

```bash
cargo test --workspace
cargo build -p prism-cli
./target/debug/prism capture --query "rust async" --provider sample --out /tmp/a.json
./target/debug/prism features --snapshot /tmp/a.json
./target/debug/prism diff --before data/samples/before.json --after data/samples/after.json
./target/debug/prism history --store-dir data/samples
./target/debug/prism report --store-dir data/samples
```

crates in the tree: prism-core, prism-collect, prism-parse, prism-features, prism-store, prism-diff, prism-analyze, prism-cli

sample provider is offline and deterministic. live HTML parsers will break. that is fine. the rank math is the part that has to stay true.

MIT.
