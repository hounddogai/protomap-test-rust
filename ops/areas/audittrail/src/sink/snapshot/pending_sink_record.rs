use super::*;

use acme_kit::text;

/// Where a pending sink record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSinkRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingSinkRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingSinkRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending sink record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkRecord {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub burst: i64,
    pub starts_at: i64,
    pub status: PendingSinkRecordStatus,
}

impl PendingSinkRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSinkRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSinkRecordStatus) -> bool {
        match self.status {
            PendingSinkRecordStatus::Created => {
                return next == PendingSinkRecordStatus::Validated;
            }
            PendingSinkRecordStatus::Validated => {
                match next {
                    PendingSinkRecordStatus::Published | PendingSinkRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSinkRecordStatus::Published => {
                return next == PendingSinkRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSinkRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending sink record {} cannot move from {} to {}",
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.starts_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSinkRecord {
        PendingSinkRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            burst: self.burst,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending sink record in status created.
pub fn new_pending_sink_record(code: &str, label: &str) -> PendingSinkRecord {
    PendingSinkRecord {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        burst: 0,
        starts_at: 0,
        status: PendingSinkRecordStatus::Created,
    }
}
