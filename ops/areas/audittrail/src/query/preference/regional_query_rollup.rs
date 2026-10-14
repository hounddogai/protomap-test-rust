use super::*;

use acme_kit::text;

/// Where a regional query rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQueryRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalQueryRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalQueryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional query rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollup {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub duration_minutes: i64,
    pub currency: String,
    pub created_at: i64,
    pub principal: String,
    pub status: RegionalQueryRollupStatus,
}

impl RegionalQueryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQueryRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQueryRollupStatus) -> bool {
        match self.status {
            RegionalQueryRollupStatus::Proposed => {
                return next == RegionalQueryRollupStatus::Withdrawn;
            }
            RegionalQueryRollupStatus::Approved => {
                return next == RegionalQueryRollupStatus::Rejected || next == RegionalQueryRollupStatus::Withdrawn;
            }
            RegionalQueryRollupStatus::Rejected => {
                return next == RegionalQueryRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalQueryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional query rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalQueryRollup {
        RegionalQueryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            created_at: self.created_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional query rollup in status proposed.
pub fn new_regional_query_rollup(code: &str, title: &str) -> RegionalQueryRollup {
    RegionalQueryRollup {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        duration_minutes: 0,
        currency: "".to_string(),
        created_at: 0,
        principal: "".to_string(),
        status: RegionalQueryRollupStatus::Proposed,
    }
}
