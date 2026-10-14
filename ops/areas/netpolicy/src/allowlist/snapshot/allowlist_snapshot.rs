use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a allowlist snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AllowlistSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSnapshot {
    pub code: String,
    pub label: String,
    pub title: String,
    pub updated_at: i64,
    pub notes: String,
    pub ends_at: i64,
    pub limit: i64,
    pub locale: String,
    pub threshold: f64,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub status: AllowlistSnapshotStatus,
}

impl AllowlistSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistSnapshotStatus) -> bool {
        match self.status {
            AllowlistSnapshotStatus::New => {
                match next {
                    AllowlistSnapshotStatus::Done | AllowlistSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSnapshotStatus::Queued => {
                match next {
                    AllowlistSnapshotStatus::Running | AllowlistSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSnapshotStatus::Running => {
                match next {
                    AllowlistSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSnapshotStatus::Done => {
                return next == AllowlistSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        ((self.ends_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistSnapshot {
        AllowlistSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            notes: self.notes.clone(),
            ends_at: self.ends_at,
            limit: self.limit,
            locale: self.locale.clone(),
            threshold: self.threshold,
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a allowlist snapshot in status new.
pub fn new_allowlist_snapshot(code: &str, label: &str) -> AllowlistSnapshot {
    AllowlistSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        updated_at: 0,
        notes: "".to_string(),
        ends_at: 0,
        limit: 0,
        locale: "".to_string(),
        threshold: 0.0,
        attributes: BTreeMap::new(),
        quantity: 0,
        status: AllowlistSnapshotStatus::New,
    }
}
