//! Feature extraction from snapshots.

use prism_core::{RankedResult, ResultType, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultFeatures {
    pub position: u32,
    pub domain: String,
    pub title_token_count: usize,
    pub title_has_query_term: bool,
    pub snippet_len: usize,
    pub is_organic: bool,
    pub is_featured: bool,
    pub has_sitelinks: bool,
    pub has_rich_snippet: bool,
    pub path_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFeatures {
    pub snapshot_id: String,
    pub query: String,
    pub organic_count: usize,
    pub featured_count: usize,
    pub unique_domains: usize,
    pub results: Vec<ResultFeatures>,
}

pub fn extract_result(query: &str, r: &RankedResult) -> ResultFeatures {
    let q_terms: Vec<&str> = query.split_whitespace().collect();
    let title_lower = r.title.to_lowercase();
    let title_has = q_terms.iter().any(|t| title_lower.contains(&t.to_lowercase()));
    let path_depth = r.url.split("://").nth(1)
        .map(|rest| rest.split('/').filter(|s| !s.is_empty()).count().saturating_sub(1))
        .unwrap_or(0);
    ResultFeatures {
        position: r.position,
        domain: r.domain.clone(),
        title_token_count: r.title.split_whitespace().count(),
        title_has_query_term: title_has,
        snippet_len: r.snippet.as_ref().map(|s| s.len()).unwrap_or(0),
        is_organic: r.result_type == ResultType::Organic,
        is_featured: r.result_type == ResultType::FeaturedSnippet,
        has_sitelinks: r.has_sitelinks,
        has_rich_snippet: r.has_rich_snippet,
        path_depth,
    }
}

pub fn extract_snapshot(snap: &Snapshot) -> SnapshotFeatures {
    let mut domains = std::collections::HashSet::new();
    let mut organic = 0usize;
    let mut featured = 0usize;
    let mut results = Vec::new();
    for r in &snap.results {
        domains.insert(r.domain.to_lowercase());
        match r.result_type {
            ResultType::Organic => organic += 1,
            ResultType::FeaturedSnippet => featured += 1,
            _ => {}
        }
        results.push(extract_result(&snap.query.text, r));
    }
    SnapshotFeatures {
        snapshot_id: snap.id.to_string(),
        query: snap.query.text.clone(),
        organic_count: organic,
        featured_count: featured,
        unique_domains: domains.len(),
        results,
    }
}

pub fn domain_frequency(snaps: &[Snapshot]) -> HashMap<String, u32> {
    let mut freq = HashMap::new();
    for s in snaps {
        for r in &s.results {
            *freq.entry(r.domain.to_lowercase()).or_insert(0) += 1;
        }
    }
    freq
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{Device, Query, SnapshotId};
    use chrono::Utc;
    #[test]
    fn extracts_basic_features() {
        let snap = Snapshot {
            id: SnapshotId::new(),
            query: Query { text: "rust docs".into(), locale: None, device: Device::Desktop },
            captured_at: Utc::now(),
            provider: "sample".into(),
            results: vec![RankedResult {
                position: 1, url: "https://docs.rs/foo".into(), domain: "docs.rs".into(),
                title: "Rust docs for foo".into(), snippet: Some("hello".into()),
                result_type: ResultType::Organic, has_sitelinks: true, has_rich_snippet: false,
                extra: HashMap::new(),
            }],
            notes: None,
        };
        let f = extract_snapshot(&snap);
        assert_eq!(f.organic_count, 1);
        assert!(f.results[0].title_has_query_term);
    }
}
