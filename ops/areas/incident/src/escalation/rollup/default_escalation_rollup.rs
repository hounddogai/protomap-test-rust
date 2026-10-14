use super::*;

use acme_kit::text;

/// Where a default escalation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEscalationRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultEscalationRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultEscalationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default escalation rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollup {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub weight: f64,
    pub phone: String,
    pub timezone: String,
    pub principal: String,
    pub sequence: i64,
    pub status: DefaultEscalationRollupStatus,
}

impl DefaultEscalationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEscalationRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEscalationRollupStatus) -> bool {
        match self.status {
            DefaultEscalationRollupStatus::Open => {
                return next == DefaultEscalationRollupStatus::Resolved
                    || next == DefaultEscalationRollupStatus::Archived;
            }
            DefaultEscalationRollupStatus::InReview => {
                return next == DefaultEscalationRollupStatus::Reopened;
            }
            DefaultEscalationRollupStatus::Resolved => {
                return next == DefaultEscalationRollupStatus::Reopened
                    || next == DefaultEscalationRollupStatus::Archived;
            }
            DefaultEscalationRollupStatus::Reopened => {
                match next {
                    DefaultEscalationRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultEscalationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default escalation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.sequence as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEscalationRollup {
        DefaultEscalationRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            weight: self.weight,
            phone: self.phone.clone(),
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default escalation rollup in status open.
pub fn new_default_escalation_rollup(code: &str, name: &str) -> DefaultEscalationRollup {
    DefaultEscalationRollup {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        weight: 0.0,
        phone: "".to_string(),
        timezone: "".to_string(),
        principal: "".to_string(),
        sequence: 0,
        status: DefaultEscalationRollupStatus::Open,
    }
}
