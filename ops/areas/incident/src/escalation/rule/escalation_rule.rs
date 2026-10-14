use super::*;

use acme_kit::text;

/// Where a escalation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationRuleStatus {
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

impl std::fmt::Display for EscalationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRule {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub score: f64,
    pub scope: String,
    pub retries: i64,
    pub visible: bool,
    pub status: EscalationRuleStatus,
}

impl EscalationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationRuleStatus) -> bool {
        match self.status {
            EscalationRuleStatus::Planned => {
                return next == EscalationRuleStatus::InProgress;
            }
            EscalationRuleStatus::Scheduled => {
                return next == EscalationRuleStatus::InProgress || next == EscalationRuleStatus::Cancelled;
            }
            EscalationRuleStatus::InProgress => {
                return next == EscalationRuleStatus::Cancelled;
            }
            EscalationRuleStatus::Completed => {
                return next == EscalationRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation rule", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.score - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationRule {
        EscalationRule {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            score: self.score,
            scope: self.scope.clone(),
            retries: self.retries,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a escalation rule in status planned.
pub fn new_escalation_rule(code: &str, label: &str) -> EscalationRule {
    EscalationRule {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        score: 0.0,
        scope: "".to_string(),
        retries: 0,
        visible: false,
        status: EscalationRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_escalation_rule("code-249", "label-503");
        assert_eq!(item.status, EscalationRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_escalation_rule("code-249", "label-503");
        if item.move_to(EscalationRuleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationRuleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_escalation_rule("code-249", "label-503");
        item.status = EscalationRuleStatus::Cancelled;
        assert!(
            item.move_to(EscalationRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
