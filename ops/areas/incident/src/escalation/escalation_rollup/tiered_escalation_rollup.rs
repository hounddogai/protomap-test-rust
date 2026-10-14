use super::*;

use acme_kit::text;

/// Where a tiered escalation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredEscalationRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredEscalationRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredEscalationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered escalation rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationRollup {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub port: i64,
    pub source: String,
    pub timezone: String,
    pub ceiling: f64,
    pub archived: bool,
    pub notes: String,
    pub amount: f64,
    pub fingerprint: String,
    pub status: TieredEscalationRollupStatus,
}

impl TieredEscalationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredEscalationRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredEscalationRollupStatus) -> bool {
        match self.status {
            TieredEscalationRollupStatus::Created => {
                match next {
                    TieredEscalationRollupStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredEscalationRollupStatus::Validated => {
                return next == TieredEscalationRollupStatus::Published
                    || next == TieredEscalationRollupStatus::Retired;
            }
            TieredEscalationRollupStatus::Published => {
                return next == TieredEscalationRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredEscalationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered escalation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredEscalationRollup {
        TieredEscalationRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            port: self.port,
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            archived: self.archived,
            notes: self.notes.clone(),
            amount: self.amount,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered escalation rollup in status created.
pub fn new_tiered_escalation_rollup(code: &str, label: &str) -> TieredEscalationRollup {
    TieredEscalationRollup {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        port: 0,
        source: "".to_string(),
        timezone: "".to_string(),
        ceiling: 0.0,
        archived: false,
        notes: "".to_string(),
        amount: 0.0,
        fingerprint: "".to_string(),
        status: TieredEscalationRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_escalation_rollup("code-920", "label-214");
        assert_eq!(item.status, TieredEscalationRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_escalation_rollup("code-920", "label-214");
        if item.move_to(TieredEscalationRollupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredEscalationRollupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_escalation_rollup("code-920", "label-214");
        item.status = TieredEscalationRollupStatus::Retired;
        assert!(
            item.move_to(TieredEscalationRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
