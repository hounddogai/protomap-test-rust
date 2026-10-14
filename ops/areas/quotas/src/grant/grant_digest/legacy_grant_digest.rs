use super::*;

use acme_kit::text;

/// Where a legacy grant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyGrantDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyGrantDigestStatus {
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

impl std::fmt::Display for LegacyGrantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy grant digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantDigest {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub updated_at: i64,
    pub key_id: String,
    pub status: LegacyGrantDigestStatus,
}

impl LegacyGrantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyGrantDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyGrantDigestStatus) -> bool {
        match self.status {
            LegacyGrantDigestStatus::Planned => {
                return next == LegacyGrantDigestStatus::InProgress || next == LegacyGrantDigestStatus::Cancelled;
            }
            LegacyGrantDigestStatus::Scheduled => {
                return next == LegacyGrantDigestStatus::InProgress || next == LegacyGrantDigestStatus::Cancelled;
            }
            LegacyGrantDigestStatus::InProgress => {
                return next == LegacyGrantDigestStatus::Cancelled;
            }
            LegacyGrantDigestStatus::Completed => {
                return next == LegacyGrantDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyGrantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy grant digest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "legacy grant digest", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.updated_at as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyGrantDigest {
        LegacyGrantDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy grant digest in status planned.
pub fn new_legacy_grant_digest(code: &str, name: &str) -> LegacyGrantDigest {
    LegacyGrantDigest {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        updated_at: 0,
        key_id: "".to_string(),
        status: LegacyGrantDigestStatus::Planned,
    }
}
