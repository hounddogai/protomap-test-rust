use super::*;

use acme_kit::text;

/// Where a strict segment manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSegmentManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictSegmentManifestStatus {
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

impl std::fmt::Display for StrictSegmentManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict segment manifest as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifest {
    pub code: String,
    pub label: String,
    pub source: String,
    pub starts_at: i64,
    pub cluster: String,
    pub status: StrictSegmentManifestStatus,
}

impl StrictSegmentManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSegmentManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSegmentManifestStatus) -> bool {
        match self.status {
            StrictSegmentManifestStatus::Planned => {
                return next == StrictSegmentManifestStatus::Scheduled
                    || next == StrictSegmentManifestStatus::Completed;
            }
            StrictSegmentManifestStatus::Scheduled => {
                return next == StrictSegmentManifestStatus::InProgress
                    || next == StrictSegmentManifestStatus::Cancelled;
            }
            StrictSegmentManifestStatus::InProgress => {
                return next == StrictSegmentManifestStatus::Completed
                    || next == StrictSegmentManifestStatus::Cancelled;
            }
            StrictSegmentManifestStatus::Completed => {
                return next == StrictSegmentManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSegmentManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict segment manifest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> StrictSegmentManifest {
        StrictSegmentManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            starts_at: self.starts_at,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict segment manifest in status planned.
pub fn new_strict_segment_manifest(code: &str, label: &str) -> StrictSegmentManifest {
    StrictSegmentManifest {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        starts_at: 0,
        cluster: "".to_string(),
        status: StrictSegmentManifestStatus::Planned,
    }
}
