use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a grant adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GrantAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GrantAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustment {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub key_id: String,
    pub retries: i64,
    pub tags: Vec<String>,
    pub rate: f64,
    pub checksum: String,
    pub namespace: String,
    pub attributes: BTreeMap<String, String>,
    pub status: GrantAdjustmentStatus,
}

impl GrantAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantAdjustmentStatus) -> bool {
        match self.status {
            GrantAdjustmentStatus::Planned => {
                match next {
                    GrantAdjustmentStatus::Completed | GrantAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAdjustmentStatus::Scheduled => {
                match next {
                    GrantAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAdjustmentStatus::InProgress => {
                match next {
                    GrantAdjustmentStatus::Completed | GrantAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAdjustmentStatus::Completed => {
                return next == GrantAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "grant adjustment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantAdjustment {
        GrantAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            retries: self.retries,
            tags: self.tags.clone(),
            rate: self.rate,
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a grant adjustment in status planned.
pub fn new_grant_adjustment(code: &str, title: &str) -> GrantAdjustment {
    GrantAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        retries: 0,
        tags: Vec::new(),
        rate: 0.0,
        checksum: "".to_string(),
        namespace: "".to_string(),
        attributes: BTreeMap::new(),
        status: GrantAdjustmentStatus::Planned,
    }
}
