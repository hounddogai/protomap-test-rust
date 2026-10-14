use super::*;

use acme_kit::text;

/// Where a failover score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverScore {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub budget: f64,
    pub unit: String,
    pub maximum: f64,
    pub reviewer: String,
    pub status: FailoverScoreStatus,
}

impl FailoverScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverScoreStatus) -> bool {
        match self.status {
            FailoverScoreStatus::Proposed => {
                return next == FailoverScoreStatus::Approved || next == FailoverScoreStatus::Rejected;
            }
            FailoverScoreStatus::Approved => {
                match next {
                    FailoverScoreStatus::Rejected | FailoverScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverScoreStatus::Rejected => {
                match next {
                    FailoverScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FailoverScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverScore {
        FailoverScore {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            budget: self.budget,
            unit: self.unit.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a failover score in status proposed.
pub fn new_failover_score(code: &str, label: &str) -> FailoverScore {
    FailoverScore {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        budget: 0.0,
        unit: "".to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        status: FailoverScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_failover_score("code-95", "label-985");
        assert_eq!(item.status, FailoverScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_failover_score("code-95", "label-985");
        if item.move_to(FailoverScoreStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverScoreStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_failover_score("code-95", "label-985");
        item.status = FailoverScoreStatus::Withdrawn;
        assert!(
            item.move_to(FailoverScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
