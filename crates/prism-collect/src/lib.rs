//! Collectors for public SERP sources.
//!
//! Providers must only use public, permitted data.
//! The sample provider ships offline fixtures so the pipeline runs without network.

use async_trait::async_trait;
use prism_core::{PrismError, Query, RankedResult, Result, ResultType, Snapshot};
use std::collections::HashMap;
use tracing::info;

#[async_trait]
pub trait SerpProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn capture(&self, query: &Query) -> Result<Snapshot>;
}

pub struct SampleProvider;

#[async_trait]
impl SerpProvider for SampleProvider {
    fn name(&self) -> &str {
        "sample"
    }

    async fn capture(&self, query: &Query) -> Result<Snapshot> {
        info!(query = %query.text, "sample capture");
        let results = sample_results_for(&query.text);
        Ok(Snapshot::new(query.clone(), self.name(), results))
    }
}

fn sample_results_for(q: &str) -> Vec<RankedResult> {
    let seed = q.bytes().map(|b| b as u32).sum::<u32>();
    let domains = [
        ("example.com", "Example Home"),
        ("docs.rs", "Rust Docs"),
        ("wikipedia.org", "Wikipedia"),
        ("github.com", "GitHub"),
        ("arxiv.org", "arXiv"),
        ("stackoverflow.com", "Stack Overflow"),
        ("medium.com", "Medium"),
        ("reddit.com", "Reddit"),
        ("nytimes.com", "New York Times"),
        ("bbc.com", "BBC"),
    ];
    let mut results = Vec::new();
    for (i, (domain, title)) in domains.iter().enumerate() {
        let position = ((i as u32 + seed) % 10) + 1;
        results.push(RankedResult {
            position,
            url: format!("https://{}/{}", domain, q.replace(' ', "-")),
            domain: (*domain).to_string(),
            title: format!("{} -- {}", title, q),
            snippet: Some(format!("Public sample snippet for {} about {}", domain, q)),
            result_type: if i == 0 {
                ResultType::FeaturedSnippet
            } else {
                ResultType::Organic
            },
            has_sitelinks: i % 3 == 0,
            has_rich_snippet: i % 4 == 0,
            extra: HashMap::new(),
        });
    }
    results.sort_by_key(|r| r.position);
    for (i, r) in results.iter_mut().enumerate() {
        r.position = (i as u32) + 1;
    }
    results
}

pub fn provider_by_name(name: &str) -> Result<Box<dyn SerpProvider>> {
    match name {
        "sample" => Ok(Box::new(SampleProvider)),
        other => Err(PrismError::Provider(format!(
            "unknown or disabled provider: {}. only public permitted sources are supported",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::Device;

    #[tokio::test]
    async fn sample_capture_returns_results() {
        let p = SampleProvider;
        let q = Query {
            text: "site architecture".into(),
            locale: Some("en-US".into()),
            device: Device::Desktop,
        };
        let snap = p.capture(&q).await.unwrap();
        assert!(!snap.results.is_empty());
        assert_eq!(snap.provider, "sample");
    }
}
