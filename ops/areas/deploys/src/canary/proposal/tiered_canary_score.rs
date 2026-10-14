use super::*;

use acme_kit::text;

/// Where a tiered canary score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCanaryScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredCanaryScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredCanaryScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered canary score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScore {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub batch_size: i64,
    pub limit: i64,
    pub scope: String,
    pub attempts: i64,
    pub amount: f64,
    pub status: TieredCanaryScoreStatus,
}

impl TieredCanaryScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCanaryScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCanaryScoreStatus) -> bool {
        match self.status {
            TieredCanaryScoreStatus::Proposed => {
                return next == TieredCanaryScoreStatus::Withdrawn;
            }
            TieredCanaryScoreStatus::Approved => {
                return next == TieredCanaryScoreStatus::Rejected;
            }
            TieredCanaryScoreStatus::Rejected => {
                return next == TieredCanaryScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredCanaryScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered canary score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCanaryScore {
        TieredCanaryScore {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            limit: self.limit,
            scope: self.scope.clone(),
            attempts: self.attempts,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered canary score in status proposed.
pub fn new_tiered_canary_score(code: &str, label: &str) -> TieredCanaryScore {
    TieredCanaryScore {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        batch_size: 0,
        limit: 0,
        scope: "".to_string(),
        attempts: 0,
        amount: 0.0,
        status: TieredCanaryScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_canary_score("code-360", "label-150");
        assert_eq!(item.status, TieredCanaryScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_tiered_canary_score("code-360", "label-150");
        if item.move_to(TieredCanaryScoreStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCanaryScoreStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_canary_score("code-360", "label-150");
        item.status = TieredCanaryScoreStatus::Withdrawn;
        assert!(
            item.move_to(TieredCanaryScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
