use super::*;

use acme_kit::text;

/// Where a seasonal dns record checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDnsRecordCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalDnsRecordCheckpointStatus {
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

impl std::fmt::Display for SeasonalDnsRecordCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal dns record checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpoint {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub starts_at: i64,
    pub archived: bool,
    pub fingerprint: String,
    pub grace_days: i64,
    pub checksum: String,
    pub pinned: bool,
    pub retries: i64,
    pub locale: String,
    pub status: SeasonalDnsRecordCheckpointStatus,
}

impl SeasonalDnsRecordCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDnsRecordCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDnsRecordCheckpointStatus) -> bool {
        match self.status {
            SeasonalDnsRecordCheckpointStatus::Draft => {
                match next {
                    SeasonalDnsRecordCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordCheckpointStatus::Pending => {
                match next {
                    SeasonalDnsRecordCheckpointStatus::Active | SeasonalDnsRecordCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordCheckpointStatus::Active => {
                match next {
                    SeasonalDnsRecordCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordCheckpointStatus::Suspended => {
                return next == SeasonalDnsRecordCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDnsRecordCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dns record checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.starts_at as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDnsRecordCheckpoint {
        SeasonalDnsRecordCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            starts_at: self.starts_at,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            retries: self.retries,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal dns record checkpoint in status draft.
pub fn new_seasonal_dns_record_checkpoint(code: &str, name: &str) -> SeasonalDnsRecordCheckpoint {
    SeasonalDnsRecordCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        starts_at: 0,
        archived: false,
        fingerprint: "".to_string(),
        grace_days: 0,
        checksum: "".to_string(),
        pinned: false,
        retries: 0,
        locale: "".to_string(),
        status: SeasonalDnsRecordCheckpointStatus::Draft,
    }
}
