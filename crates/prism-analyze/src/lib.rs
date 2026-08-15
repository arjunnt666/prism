//! Competitor maps and aggregate views over many snapshots.

use prism_core::{CompetitorMap, CompetitorRow, Snapshot};
use std::collections::HashMap;

struct Acc { present: u32, sum_pos: f64, best: u32, worst: u32 }

pub fn competitor_map(snaps: &[Snapshot]) -> CompetitorMap {
    let mut by_domain: HashMap<String, Acc> = HashMap::new();
    let mut queries = Vec::new();
    for snap in snaps {
        queries.push(snap.query.text.clone());
        let mut seen_in_query = std::collections::HashSet::new();
        for r in &snap.results {
            let d = r.domain.to_lowercase();
            if !seen_in_query.insert(d.clone()) { continue; }
            let e = by_domain.entry(d).or_insert(Acc { present: 0, sum_pos: 0.0, best: u32::MAX, worst: 0 });
            e.present += 1;
            e.sum_pos += r.position as f64;
            e.best = e.best.min(r.position);
            e.worst = e.worst.max(r.position);
        }
    }
    let mut rows: Vec<CompetitorRow> = by_domain.into_iter().map(|(domain, a)| CompetitorRow {
        domain,
        queries_present: a.present,
        avg_position: if a.present > 0 { a.sum_pos / a.present as f64 } else { 0.0 },
        best_position: if a.best == u32::MAX { 0 } else { a.best },
        worst_position: a.worst,
    }).collect();
    rows.sort_by(|a, b| b.queries_present.cmp(&a.queries_present).then(
        a.avg_position.partial_cmp(&b.avg_position).unwrap_or(std::cmp::Ordering::Equal)
    ));
    CompetitorMap { queries, rows }
}

pub fn top_domains(map: &CompetitorMap, n: usize) -> Vec<&CompetitorRow> {
    map.rows.iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{Device, Query, RankedResult, ResultType};
    use std::collections::HashMap;
    fn snap(q: &str, domains: &[&str]) -> Snapshot {
        let results = domains.iter().enumerate().map(|(i, d)| RankedResult {
            position: (i as u32) + 1, url: format!("https://{}/", d), domain: (*d).to_string(),
            title: d.to_string(), snippet: None, result_type: ResultType::Organic,
            has_sitelinks: false, has_rich_snippet: false, extra: HashMap::new(),
        }).collect();
        Snapshot::new(Query { text: q.into(), locale: None, device: Device::Desktop }, "test", results)
    }
    #[test]
    fn map_counts_presence() {
        let snaps = vec![snap("a", &["x.com", "y.com"]), snap("b", &["x.com", "z.com"])];
        let map = competitor_map(&snaps);
        let x = map.rows.iter().find(|r| r.domain == "x.com").unwrap();
        assert_eq!(x.queries_present, 2);
    }
}
