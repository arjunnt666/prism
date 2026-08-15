//! Rank movement between two snapshots of the same query.

use prism_core::{DiffReport, PrismError, RankChangeKind, RankDelta, Result, Snapshot};
use std::collections::HashMap;

pub fn diff_snapshots(before: &Snapshot, after: &Snapshot) -> Result<DiffReport> {
    if before.query.text != after.query.text {
        return Err(PrismError::Invalid(format!("query mismatch: '{}' vs '{}'", before.query.text, after.query.text)));
    }
    let mut before_map: HashMap<String, (u32, String)> = HashMap::new();
    for r in &before.results {
        before_map.entry(r.domain.to_lowercase()).or_insert((r.position, r.url.clone()));
    }
    let mut after_map: HashMap<String, (u32, String)> = HashMap::new();
    for r in &after.results {
        after_map.entry(r.domain.to_lowercase()).or_insert((r.position, r.url.clone()));
    }
    let mut domains: std::collections::HashSet<String> = before_map.keys().cloned().collect();
    domains.extend(after_map.keys().cloned());
    let mut deltas = Vec::new();
    for domain in domains {
        let b = before_map.get(&domain);
        let a = after_map.get(&domain);
        let (before_pos, after_pos, url, kind) = match (b, a) {
            (None, Some((pos, url))) => (None, Some(*pos), url.clone(), RankChangeKind::New),
            (Some((pos, url)), None) => (Some(*pos), None, url.clone(), RankChangeKind::Dropped),
            (Some((bp, url)), Some((ap, _))) => {
                let kind = if bp == ap { RankChangeKind::Unchanged }
                    else if ap < bp { RankChangeKind::MovedUp }
                    else { RankChangeKind::MovedDown };
                (Some(*bp), Some(*ap), url.clone(), kind)
            }
            (None, None) => continue,
        };
        deltas.push(RankDelta { domain, url, before: before_pos, after: after_pos, kind });
    }
    deltas.sort_by(|x, y| x.after.unwrap_or(u32::MAX).cmp(&y.after.unwrap_or(u32::MAX)));
    Ok(DiffReport { query: before.query.clone(), before_id: before.id, after_id: after.id, deltas })
}

pub fn summarize(report: &DiffReport) -> HashMap<&'static str, usize> {
    let mut m = HashMap::new();
    for d in &report.deltas {
        let key = match d.kind {
            RankChangeKind::New => "new",
            RankChangeKind::Dropped => "dropped",
            RankChangeKind::MovedUp => "moved_up",
            RankChangeKind::MovedDown => "moved_down",
            RankChangeKind::Unchanged => "unchanged",
            RankChangeKind::Gained => "gained",
            RankChangeKind::Lost => "lost",
        };
        *m.entry(key).or_insert(0) += 1;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{Device, Query, RankedResult, ResultType};
    use std::collections::HashMap;
    fn snap(q: &str, pairs: &[(&str, u32)]) -> Snapshot {
        let results = pairs.iter().map(|(domain, pos)| RankedResult {
            position: *pos, url: format!("https://{}/", domain), domain: (*domain).to_string(),
            title: domain.to_string(), snippet: None, result_type: ResultType::Organic,
            has_sitelinks: false, has_rich_snippet: false, extra: HashMap::new(),
        }).collect();
        Snapshot::new(Query { text: q.into(), locale: None, device: Device::Desktop }, "test", results)
    }
    #[test]
    fn detects_move_and_drop() {
        let before = snap("q", &[("a.com", 1), ("b.com", 2), ("c.com", 3)]);
        let after = snap("q", &[("b.com", 1), ("a.com", 2)]);
        let report = diff_snapshots(&before, &after).unwrap();
        let kinds: Vec<_> = report.deltas.iter().map(|d| d.kind).collect();
        assert!(kinds.contains(&RankChangeKind::MovedUp));
        assert!(kinds.contains(&RankChangeKind::Dropped));
    }
}
