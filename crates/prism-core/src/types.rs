//! Domain types for ranking snapshots and SERP features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub Uuid);

impl SnapshotId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for SnapshotId {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    pub text: String,
    pub locale: Option<String>,
    pub device: Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Device { Desktop, Mobile, Tablet }

impl Default for Device {
    fn default() -> Self { Device::Desktop }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Organic, FeaturedSnippet, PeopleAlsoAsk, LocalPack, ImagePack,
    Video, News, Shopping, KnowledgePanel, Ad, Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedResult {
    pub position: u32,
    pub url: String,
    pub domain: String,
    pub title: String,
    pub snippet: Option<String>,
    pub result_type: ResultType,
    pub has_sitelinks: bool,
    pub has_rich_snippet: bool,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub query: Query,
    pub captured_at: DateTime<Utc>,
    pub provider: String,
    pub results: Vec<RankedResult>,
    pub notes: Option<String>,
}

impl Snapshot {
    pub fn new(query: Query, provider: impl Into<String>, results: Vec<RankedResult>) -> Self {
        Self {
            id: SnapshotId::new(),
            query,
            captured_at: Utc::now(),
            provider: provider.into(),
            results,
            notes: None,
        }
    }

    pub fn organic(&self) -> impl Iterator<Item = &RankedResult> {
        self.results.iter().filter(|r| r.result_type == ResultType::Organic)
    }

    pub fn position_of_domain(&self, domain: &str) -> Option<u32> {
        self.results.iter().find(|r| r.domain.eq_ignore_ascii_case(domain)).map(|r| r.position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankChangeKind {
    Gained, Lost, MovedUp, MovedDown, Unchanged, New, Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankDelta {
    pub domain: String,
    pub url: String,
    pub before: Option<u32>,
    pub after: Option<u32>,
    pub kind: RankChangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub query: Query,
    pub before_id: SnapshotId,
    pub after_id: SnapshotId,
    pub deltas: Vec<RankDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorRow {
    pub domain: String,
    pub queries_present: u32,
    pub avg_position: f64,
    pub best_position: u32,
    pub worst_position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorMap {
    pub queries: Vec<String>,
    pub rows: Vec<CompetitorRow>,
}
