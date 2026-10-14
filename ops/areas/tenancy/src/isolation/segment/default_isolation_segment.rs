use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default isolation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIsolationSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultIsolationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DefaultIsolationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default isolation segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegment {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub priority: i64,
    pub checksum: String,
    pub fingerprint: String,
    pub burst: i64,
    pub scope: String,
    pub rate: f64,
    pub starts_at: i64,
    pub status: DefaultIsolationSegmentStatus,
}

impl DefaultIsolationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIsolationSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIsolationSegmentStatus) -> bool {
        match self.status {
            DefaultIsolationSegmentStatus::Draft => {
                return next == DefaultIsolationSegmentStatus::Pending || next == DefaultIsolationSegmentStatus::Active;
            }
            DefaultIsolationSegmentStatus::Pending => {
                return next == DefaultIsolationSegmentStatus::Suspended;
            }
            DefaultIsolationSegmentStatus::Active => {
                return next == DefaultIsolationSegmentStatus::Closed;
            }
            DefaultIsolationSegmentStatus::Suspended => {
                match next {
                    DefaultIsolationSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultIsolationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default isolation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.starts_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIsolationSegment {
        DefaultIsolationSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            attributes: self.attributes.clone(),
            priority: self.priority,
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            scope: self.scope.clone(),
            rate: self.rate,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default isolation segment in status draft.
pub fn new_default_isolation_segment(code: &str, label: &str) -> DefaultIsolationSegment {
    DefaultIsolationSegment {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        attributes: BTreeMap::new(),
        priority: 0,
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        scope: "".to_string(),
        rate: 0.0,
        starts_at: 0,
        status: DefaultIsolationSegmentStatus::Draft,
    }
}
