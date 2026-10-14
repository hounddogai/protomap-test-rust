use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a release batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseBatchStatus {
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

impl std::fmt::Display for ReleaseBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatch {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub quota_units: i64,
    pub revision: i64,
    pub status: ReleaseBatchStatus,
}

impl ReleaseBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseBatchStatus) -> bool {
        match self.status {
            ReleaseBatchStatus::Open => {
                match next {
                    ReleaseBatchStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBatchStatus::InReview => {
                match next {
                    ReleaseBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseBatchStatus::Resolved => {
                return next == ReleaseBatchStatus::Archived;
            }
            ReleaseBatchStatus::Reopened => {
                match next {
                    ReleaseBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseBatch {
        ReleaseBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            quota_units: self.quota_units,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release batch in status open.
pub fn new_release_batch(code: &str, title: &str) -> ReleaseBatch {
    ReleaseBatch {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        quota_units: 0,
        revision: 0,
        status: ReleaseBatchStatus::Open,
    }
}
