# Getting started

```bash
cargo build -p prism-cli
./target/debug/prism version
./target/debug/prism capture --query "distributed systems" --provider sample --out /tmp/a.json
./target/debug/prism diff --before data/samples/before.json --after data/samples/after.json
./target/debug/prism features --snapshot data/samples/before.json
```
