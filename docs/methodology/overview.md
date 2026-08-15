# Methodology

Prism treats a SERP research run as:

1. define a query (text, locale, device)
2. capture a snapshot from a permitted public source
3. extract stable features
4. store the snapshot with a timestamp
5. diff later captures of the same query
6. aggregate across queries into competitor maps

## units of analysis

- Snapshot: one query at one time from one provider
- RankedResult: one visible result row with position and features
- RankDelta: movement of a domain between two snapshots
- CompetitorMap: domain presence across a set of queries

## what is not claimed

- causal proof that a page change caused a rank change
- complete coverage of personalized or geo-variant SERPs
- immunity to markup changes on public search pages
