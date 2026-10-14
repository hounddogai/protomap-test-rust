use super::*;

use acme_kit::text;

/// Where a retention record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RetentionRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecord {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub attempts: i64,
    pub unit: String,
    pub cluster: String,
    pub source: String,
    pub region: String,
    pub revision: i64,
    pub tags: Vec<String>,
    pub status: RetentionRecordStatus,
}

impl RetentionRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionRecordStatus) -> bool {
        match self.status {
            RetentionRecordStatus::Open => {
                match next {
                    RetentionRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRecordStatus::InReview => {
                match next {
                    RetentionRecordStatus::Resolved | RetentionRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRecordStatus::Resolved => {
                return next == RetentionRecordStatus::Reopened;
            }
            RetentionRecordStatus::Reopened => {
                return next == RetentionRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.port as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionRecord {
        RetentionRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            attempts: self.attempts,
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            revision: self.revision,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a retention record in status open.
pub fn new_retention_record(code: &str, title: &str) -> RetentionRecord {
    RetentionRecord {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        attempts: 0,
        unit: "".to_string(),
        cluster: "".to_string(),
        source: "".to_string(),
        region: "".to_string(),
        revision: 0,
        tags: Vec::new(),
        status: RetentionRecordStatus::Open,
    }
}
