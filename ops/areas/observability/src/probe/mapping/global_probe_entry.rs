use super::*;

use acme_kit::text;

/// Where a global probe entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalProbeEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalProbeEntryStatus {
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

impl std::fmt::Display for GlobalProbeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global probe entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeEntry {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub threshold: f64,
    pub updated_at: i64,
    pub fingerprint: String,
    pub category: String,
    pub status: GlobalProbeEntryStatus,
}

impl GlobalProbeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalProbeEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalProbeEntryStatus) -> bool {
        match self.status {
            GlobalProbeEntryStatus::New => {
                match next {
                    GlobalProbeEntryStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalProbeEntryStatus::Queued => {
                match next {
                    GlobalProbeEntryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalProbeEntryStatus::Running => {
                match next {
                    GlobalProbeEntryStatus::Done | GlobalProbeEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalProbeEntryStatus::Done => {
                return next == GlobalProbeEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalProbeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global probe entry {} cannot move from {} to {}",
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
        format!("{} {}", "global probe entry", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalProbeEntry {
        GlobalProbeEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            threshold: self.threshold,
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global probe entry in status new.
pub fn new_global_probe_entry(code: &str, label: &str) -> GlobalProbeEntry {
    GlobalProbeEntry {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        threshold: 0.0,
        updated_at: 0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        status: GlobalProbeEntryStatus::New,
    }
}
