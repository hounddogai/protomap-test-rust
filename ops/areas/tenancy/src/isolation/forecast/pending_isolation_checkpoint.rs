use super::*;

use acme_kit::text;

/// Where a pending isolation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIsolationCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIsolationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIsolationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending isolation checkpoint as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationCheckpoint {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub sequence: i64,
    pub ends_at: i64,
    pub summary: String,
    pub namespace: String,
    pub notes: String,
    pub region: String,
    pub margin: f64,
    pub port: i64,
    pub status: PendingIsolationCheckpointStatus,
}

impl PendingIsolationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIsolationCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIsolationCheckpointStatus) -> bool {
        match self.status {
            PendingIsolationCheckpointStatus::Created => {
                return next == PendingIsolationCheckpointStatus::Retired;
            }
            PendingIsolationCheckpointStatus::Validated => {
                match next {
                    PendingIsolationCheckpointStatus::Published | PendingIsolationCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIsolationCheckpointStatus::Published => {
                match next {
                    PendingIsolationCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingIsolationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending isolation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.margin * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIsolationCheckpoint {
        PendingIsolationCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            notes: self.notes.clone(),
            region: self.region.clone(),
            margin: self.margin,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending isolation checkpoint in status created.
pub fn new_pending_isolation_checkpoint(code: &str, title: &str) -> PendingIsolationCheckpoint {
    PendingIsolationCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        sequence: 0,
        ends_at: 0,
        summary: "".to_string(),
        namespace: "".to_string(),
        notes: "".to_string(),
        region: "".to_string(),
        margin: 0.0,
        port: 0,
        status: PendingIsolationCheckpointStatus::Created,
    }
}
