//! Best-effort parsers for public SERP shapes.

use once_cell::sync::Lazy;
use prism_core::{PrismError, RankedResult, Result, ResultType};
use regex::Regex;
use std::collections::HashMap;
use url::Url;

static HREF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"href=["'](https?://[^"']+)["']"#).expect("href regex"));
static TITLE_HINT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<h3[^>]*>(.*?)</h3>").expect("title regex"));

pub fn domain_of(url_str: &str) -> Result<String> {
    let u = Url::parse(url_str).map_err(|e| PrismError::Parse(e.to_string()))?;
    u.host_str()
        .map(|h| h.trim_start_matches("www.").to_string())
        .ok_or_else(|| PrismError::Parse(format!("no host in {}", url_str)))
}

pub fn parse_basic_html(html: &str) -> Result<Vec<RankedResult>> {
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut position = 1u32;
    for cap in HREF_RE.captures_iter(html) {
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if href.contains("google.") || href.contains("bing.") || href.starts_with('#') { continue; }
        let domain = match domain_of(href) { Ok(d) => d, Err(_) => continue };
        if !seen.insert(domain.clone()) { continue; }
        let title = TITLE_HINT_RE.captures(html)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_else(|| domain.clone());
        results.push(RankedResult {
            position, url: href.to_string(), domain, title, snippet: None,
            result_type: ResultType::Organic, has_sitelinks: false, has_rich_snippet: false,
            extra: HashMap::new(),
        });
        position += 1;
        if position > 20 { break; }
    }
    Ok(results)
}

pub fn parse_snapshot_json(raw: &str) -> Result<prism_core::Snapshot> {
    Ok(serde_json::from_str(raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn domain_strips_www() {
        assert_eq!(domain_of("https://www.example.com/a").unwrap(), "example.com");
    }
    #[test]
    fn basic_html_finds_links() {
        let html = r#"<a href="https://docs.rs/foo">x</a><h3>Rust Docs</h3><a href="https://github.com/bar">y</a>"#;
        let results = parse_basic_html(html).unwrap();
        assert!(results.len() >= 2);
        assert_eq!(results[0].domain, "docs.rs");
    }
}
