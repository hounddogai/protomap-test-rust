use super::*;

use acme_kit::text;

/// Where a failover snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FailoverSnapshotStatus {
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

impl std::fmt::Display for FailoverSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover snapshot as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshot {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub score: f64,
    pub unit: String,
    pub retries: i64,
    pub expires_at: i64,
    pub fingerprint: String,
    pub starts_at: i64,
    pub sequence: i64,
    pub version: i64,
    pub status: FailoverSnapshotStatus,
}

impl FailoverSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverSnapshotStatus) -> bool {
        match self.status {
            FailoverSnapshotStatus::Planned => {
                match next {
                    FailoverSnapshotStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverSnapshotStatus::Scheduled => {
                return next == FailoverSnapshotStatus::Cancelled;
            }
            FailoverSnapshotStatus::InProgress => {
                return next == FailoverSnapshotStatus::Completed;
            }
            FailoverSnapshotStatus::Completed => {
                return next == FailoverSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverSnapshot {
        FailoverSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            score: self.score,
            unit: self.unit.clone(),
            retries: self.retries,
            expires_at: self.expires_at,
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            sequence: self.sequence,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a failover snapshot in status planned.
pub fn new_failover_snapshot(code: &str, title: &str) -> FailoverSnapshot {
    FailoverSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        score: 0.0,
        unit: "".to_string(),
        retries: 0,
        expires_at: 0,
        fingerprint: "".to_string(),
        starts_at: 0,
        sequence: 0,
        version: 0,
        status: FailoverSnapshotStatus::Planned,
    }
}
