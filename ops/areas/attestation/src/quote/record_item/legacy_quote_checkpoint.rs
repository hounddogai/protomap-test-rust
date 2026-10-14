use super::*;

use acme_kit::text;

/// Where a legacy quote checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQuoteCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyQuoteCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyQuoteCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy quote checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpoint {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub principal: String,
    pub region: String,
    pub priority: i64,
    pub country: String,
    pub updated_at: i64,
    pub timezone: String,
    pub grace_days: i64,
    pub status: LegacyQuoteCheckpointStatus,
}

impl LegacyQuoteCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQuoteCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQuoteCheckpointStatus) -> bool {
        match self.status {
            LegacyQuoteCheckpointStatus::Requested => {
                return next == LegacyQuoteCheckpointStatus::Revoked;
            }
            LegacyQuoteCheckpointStatus::Granted => {
                match next {
                    LegacyQuoteCheckpointStatus::Revoked | LegacyQuoteCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQuoteCheckpointStatus::Revoked => {
                return next == LegacyQuoteCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyQuoteCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy quote checkpoint {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQuoteCheckpoint {
        LegacyQuoteCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            priority: self.priority,
            country: self.country.clone(),
            updated_at: self.updated_at,
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy quote checkpoint in status requested.
pub fn new_legacy_quote_checkpoint(code: &str, title: &str) -> LegacyQuoteCheckpoint {
    LegacyQuoteCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        principal: "".to_string(),
        region: "".to_string(),
        priority: 0,
        country: "".to_string(),
        updated_at: 0,
        timezone: "".to_string(),
        grace_days: 0,
        status: LegacyQuoteCheckpointStatus::Requested,
    }
}
