use super::*;

use acme_kit::text;

/// Where a escalation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationRollupStatus {
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

impl std::fmt::Display for EscalationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation rollup as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRollup {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub quantity: i64,
    pub summary: String,
    pub status: EscalationRollupStatus,
}

impl EscalationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationRollupStatus) -> bool {
        match self.status {
            EscalationRollupStatus::Open => {
                match next {
                    EscalationRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationRollupStatus::InReview => {
                match next {
                    EscalationRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationRollupStatus::Resolved => {
                return next == EscalationRollupStatus::Archived;
            }
            EscalationRollupStatus::Reopened => {
                match next {
                    EscalationRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: EscalationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation rollup", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationRollup {
        EscalationRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            quantity: self.quantity,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation rollup in status open.
pub fn new_escalation_rollup(code: &str, name: &str) -> EscalationRollup {
    EscalationRollup {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        quantity: 0,
        summary: "".to_string(),
        status: EscalationRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_rollup("code-90", "name-15");
        assert_eq!(item.status, EscalationRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_escalation_rollup("code-90", "name-15");
        if item.move_to(EscalationRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_rollup("code-90", "name-15");
        item.status = EscalationRollupStatus::Archived;
        assert!(
            item.move_to(EscalationRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
