use super::*;

use acme_kit::text;

/// Where a legacy redaction rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRedactionRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyRedactionRollupStatus {
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

impl std::fmt::Display for LegacyRedactionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy redaction rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionRollup {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub quota_units: i64,
    pub cost: f64,
    pub contact_email: String,
    pub sequence: i64,
    pub status: LegacyRedactionRollupStatus,
}

impl LegacyRedactionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRedactionRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRedactionRollupStatus) -> bool {
        match self.status {
            LegacyRedactionRollupStatus::Draft => {
                return next == LegacyRedactionRollupStatus::Active;
            }
            LegacyRedactionRollupStatus::Pending => {
                return next == LegacyRedactionRollupStatus::Active || next == LegacyRedactionRollupStatus::Closed;
            }
            LegacyRedactionRollupStatus::Active => {
                return next == LegacyRedactionRollupStatus::Suspended || next == LegacyRedactionRollupStatus::Closed;
            }
            LegacyRedactionRollupStatus::Suspended => {
                match next {
                    LegacyRedactionRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyRedactionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy redaction rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.sequence as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRedactionRollup {
        LegacyRedactionRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy redaction rollup in status draft.
pub fn new_legacy_redaction_rollup(code: &str, name: &str) -> LegacyRedactionRollup {
    LegacyRedactionRollup {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        quota_units: 0,
        cost: 0.0,
        contact_email: "".to_string(),
        sequence: 0,
        status: LegacyRedactionRollupStatus::Draft,
    }
}
