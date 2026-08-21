//! Competitor maps and aggregate views over many snapshots.

use prism_core::{CompetitorMap, CompetitorRow, Snapshot};
use std::collections::{BTreeMap, HashMap};

struct Acc {
    present: u32,
    sum_pos: f64,
    best: u32,
    worst: u32,
}

pub fn competitor_map(snaps: &[Snapshot]) -> CompetitorMap {
    let mut by_domain: HashMap<String, Acc> = HashMap::new();
    let mut queries = Vec::new();
    for snap in snaps {
        queries.push(snap.query.text.clone());
        let mut seen_in_query = std::collections::HashSet::new();
        for r in &snap.results {
            let d = r.domain.to_lowercase();
            if !seen_in_query.insert(d.clone()) {
                continue;
            }
            let e = by_domain.entry(d).or_insert(Acc {
                present: 0,
                sum_pos: 0.0,
                best: u32::MAX,
                worst: 0,
            });
            e.present += 1;
            e.sum_pos += r.position as f64;
            e.best = e.best.min(r.position);
            e.worst = e.worst.max(r.position);
        }
    }
    let mut rows: Vec<CompetitorRow> = by_domain
        .into_iter()
        .map(|(domain, a)| CompetitorRow {
            domain,
            queries_present: a.present,
            avg_position: if a.present > 0 {
                a.sum_pos / a.present as f64
            } else {
                0.0
            },
            best_position: if a.best == u32::MAX { 0 } else { a.best },
            worst_position: a.worst,
        })
        .collect();
    rows.sort_by(|a, b| {
        b.queries_present
            .cmp(&a.queries_present)
            .then(a.avg_position.partial_cmp(&b.avg_position).unwrap_or(std::cmp::Ordering::Equal))
    });
    CompetitorMap { queries, rows }
}

pub fn top_domains(map: &CompetitorMap, n: usize) -> Vec<&CompetitorRow> {
    map.rows.iter().take(n).collect()
}

#[derive(Debug, Clone)]
pub struct HistoryRow {
    pub id: String,
    pub query: String,
    pub captured_at: String,
    pub results: usize,
    pub top_domain: Option<String>,
}

pub fn history_rows(snaps: &[Snapshot]) -> Vec<HistoryRow> {
    snaps
        .iter()
        .map(|s| HistoryRow {
            id: s.id.to_string(),
            query: s.query.text.clone(),
            captured_at: format!("{}", s.captured_at),
            results: s.results.len(),
            top_domain: s
                .results
                .iter()
                .find(|r| r.position == 1)
                .map(|r| r.domain.clone()),
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct QueryShift {
    pub query: String,
    pub snapshots: usize,
    pub movers: Vec<(String, Option<u32>, Option<u32>)>,
}

/// First vs last snapshot per query. Position None means the domain was absent.
pub fn query_shifts(snaps: &[Snapshot]) -> Vec<QueryShift> {
    let mut by_query: BTreeMap<String, Vec<&Snapshot>> = BTreeMap::new();
    for s in snaps {
        by_query.entry(s.query.text.clone()).or_default().push(s);
    }
    let mut out = Vec::new();
    for (query, mut group) in by_query {
        group.sort_by_key(|s| s.captured_at);
        let snapshots = group.len();
        if snapshots == 0 {
            continue;
        }
        let first = group[0];
        let last = group[snapshots - 1];
        let mut domains = std::collections::BTreeSet::new();
        for r in first.results.iter().chain(last.results.iter()) {
            domains.insert(r.domain.to_lowercase());
        }
        let pos = |snap: &Snapshot, domain: &str| -> Option<u32> {
            snap.results
                .iter()
                .find(|r| r.domain.eq_ignore_ascii_case(domain))
                .map(|r| r.position)
        };
        let mut movers: Vec<(String, Option<u32>, Option<u32>)> = domains
            .into_iter()
            .map(|d| {
                let a = pos(first, &d);
                let b = pos(last, &d);
                (d, a, b)
            })
            .filter(|(_, a, b)| a != b)
            .collect();
        movers.sort_by(|x, y| x.0.cmp(&y.0));
        out.push(QueryShift {
            query,
            snapshots,
            movers,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{Device, Query, RankedResult, ResultType};
    use std::collections::HashMap;
    fn snap(q: &str, domains: &[&str]) -> Snapshot {
        let results = domains
            .iter()
            .enumerate()
            .map(|(i, d)| RankedResult {
                position: (i as u32) + 1,
                url: format!("https://{}/", d),
                domain: (*d).to_string(),
                title: d.to_string(),
                snippet: None,
                result_type: ResultType::Organic,
                has_sitelinks: false,
                has_rich_snippet: false,
                extra: HashMap::new(),
            })
            .collect();
        Snapshot::new(
            Query {
                text: q.into(),
                locale: None,
                device: Device::Desktop,
            },
            "test",
            results,
        )
    }
    #[test]
    fn map_counts_presence() {
        let snaps = vec![snap("a", &["x.com", "y.com"]), snap("b", &["x.com", "z.com"])];
        let map = competitor_map(&snaps);
        let x = map.rows.iter().find(|r| r.domain == "x.com").unwrap();
        assert_eq!(x.queries_present, 2);
    }

    #[test]
    fn history_lists_top_domain() {
        let snaps = vec![snap("rust", &["docs.rs", "example.com"])];
        let rows = history_rows(&snaps);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].top_domain.as_deref(), Some("docs.rs"));
        assert_eq!(rows[0].results, 2);
    }

    #[test]
    fn shift_detects_swap_and_drop() {
        let a = snap("q", &["example.com", "docs.rs", "wikipedia.org"]);
        let b = snap("q", &["docs.rs", "example.com", "github.com"]);
        let shifts = query_shifts(&[a, b]);
        assert_eq!(shifts.len(), 1);
        assert_eq!(shifts[0].snapshots, 2);
        let find = |d: &str| {
            shifts[0]
                .movers
                .iter()
                .find(|(name, _, _)| name == d)
                .cloned()
        };
        assert_eq!(find("docs.rs"), Some(("docs.rs".into(), Some(2), Some(1))));
        assert_eq!(
            find("wikipedia.org"),
            Some(("wikipedia.org".into(), Some(3), None))
        );
        assert_eq!(
            find("github.com"),
            Some(("github.com".into(), None, Some(3)))
        );
    }
}
