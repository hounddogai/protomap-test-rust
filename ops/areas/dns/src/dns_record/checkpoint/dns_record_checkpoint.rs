use super::*;

use acme_kit::text;

/// Where a dns record checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordCheckpointStatus {
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

impl std::fmt::Display for DnsRecordCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record checkpoint as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpoint {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub port: i64,
    pub description: String,
    pub notes: String,
    pub checksum: String,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub score: f64,
    pub scope: String,
    pub status: DnsRecordCheckpointStatus,
}

impl DnsRecordCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordCheckpointStatus) -> bool {
        match self.status {
            DnsRecordCheckpointStatus::New => {
                return next == DnsRecordCheckpointStatus::Running || next == DnsRecordCheckpointStatus::Done;
            }
            DnsRecordCheckpointStatus::Queued => {
                match next {
                    DnsRecordCheckpointStatus::Running | DnsRecordCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordCheckpointStatus::Running => {
                return next == DnsRecordCheckpointStatus::Done || next == DnsRecordCheckpointStatus::Failed;
            }
            DnsRecordCheckpointStatus::Done => {
                return next == DnsRecordCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record checkpoint {} cannot move from {} to {}",
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordCheckpoint {
        DnsRecordCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            port: self.port,
            description: self.description.clone(),
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            score: self.score,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a dns record checkpoint in status new.
pub fn new_dns_record_checkpoint(code: &str, title: &str) -> DnsRecordCheckpoint {
    DnsRecordCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        port: 0,
        description: "".to_string(),
        notes: "".to_string(),
        checksum: "".to_string(),
        created_at: 0,
        duration_minutes: 0,
        score: 0.0,
        scope: "".to_string(),
        status: DnsRecordCheckpointStatus::New,
    }
}
