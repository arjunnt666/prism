# prism

SERP research pipeline.

public ranking snapshots. feature extraction. competitor maps.
methodology first. public data only.

not a rank manipulator. not a cloaking toolkit. not a link network.
this is instrumentation for studying how search results look and change over time when you only use sources that are already public.

## why

ranking research is mostly tribal knowledge and screenshots.
prism is a structured way to:

- capture SERP shapes from public interfaces
- extract stable features (position, title, url, result type, sitelinks presence, etc.)
- store historical snapshots
- diff rank movement between captures
- build competitor maps without inventing private data sources

if you need black box secret sauce ranking hacks, this is the wrong repo.
if you want reproducible notes on what the public SERP showed on a given day, this is closer.

## public data only

allowed inputs in this design:

- public search result pages you are permitted to access
- published ranking datasets and open dumps
- your own crawl of sites you own or have rights to
- APIs that explicitly allow research use under their terms

not in scope:

- bypassing rate limits or access controls
- private index leaks
- user-specific personalized SERPs collected without consent
- anything that requires deception to obtain

methodology details live under `docs/methodology/`.

## what is implemented

deep enough to run locally as a research lab, not a one file stub.

- typed rank snapshots and result features
- collectors with a clear provider trait (public sources only)
- html structure parsers for common SERP layouts (best effort, will break when markup changes)
- feature extraction (position, domain, title tokens, rich result flags)
- append only snapshot store
- rank diff engine (gained, lost, moved, new, dropped)
- competitor map builder from a set of queries
- cli for capture, diff, and export
- python package for notebook style analysis

## status

this is a research instrument, not a product.
parsers will lag real world SERP markup.
treat outputs as structured notes, not ground truth for legal or financial decisions.

## crates

| crate | role |
|-------|------|
| prism-core | ids, snapshot types, errors |
| prism-collect | provider trait + public source adapters |
| prism-parse | serp html / json shape parsers |
| prism-features | feature extractors |
| prism-store | snapshot persistence |
| prism-diff | rank movement between snapshots |
| prism-analyze | competitor maps and aggregates |
| prism-cli | capture / diff / export tooling |

## quick start

```bash
cargo build -p prism-cli
./target/debug/prism version
./target/debug/prism capture --query "site architecture" --provider sample
./target/debug/prism diff --before data/samples/before.json --after data/samples/after.json
```

sample data is synthetic and checked in for offline demos.

## license

mit.
do not use this to violate search engine terms.
do not pretend a research snapshot is permission to spam.
