use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a sink snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SinkSnapshotStatus {
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

impl std::fmt::Display for SinkSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshot {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: SinkSnapshotStatus,
}

impl SinkSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkSnapshotStatus) -> bool {
        match self.status {
            SinkSnapshotStatus::Open => {
                match next {
                    SinkSnapshotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkSnapshotStatus::InReview => {
                match next {
                    SinkSnapshotStatus::Resolved | SinkSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkSnapshotStatus::Resolved => {
                return next == SinkSnapshotStatus::Reopened;
            }
            SinkSnapshotStatus::Reopened => {
                match next {
                    SinkSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: SinkSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink snapshot", self.code)
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

    pub fn net_load(&self) -> f64 {
        ((self.cost + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkSnapshot {
        SinkSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            cost: self.cost,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink snapshot in status open.
pub fn new_sink_snapshot(code: &str, label: &str) -> SinkSnapshot {
    SinkSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        cost: 0.0,
        attributes: BTreeMap::new(),
        status: SinkSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_sink_snapshot("code-69", "label-86");
        assert_eq!(item.status, SinkSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_sink_snapshot("code-69", "label-86");
        if item.move_to(SinkSnapshotStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkSnapshotStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_sink_snapshot("code-69", "label-86");
        item.status = SinkSnapshotStatus::Archived;
        assert!(item.move_to(SinkSnapshotStatus::Open).is_err(), "expected move to fail");
    }
}
