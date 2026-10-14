use super::*;

use acme_kit::text;

/// Where a peering rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PeeringRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRule {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub ceiling: f64,
    pub active: bool,
    pub amount: f64,
    pub issuer: String,
    pub grace_days: i64,
    pub timezone: String,
    pub status: PeeringRuleStatus,
}

impl PeeringRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringRuleStatus) -> bool {
        match self.status {
            PeeringRuleStatus::New => {
                return next == PeeringRuleStatus::Running || next == PeeringRuleStatus::Failed;
            }
            PeeringRuleStatus::Queued => {
                return next == PeeringRuleStatus::Done || next == PeeringRuleStatus::Failed;
            }
            PeeringRuleStatus::Running => {
                match next {
                    PeeringRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringRuleStatus::Done => {
                return next == PeeringRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.amount * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringRule {
        PeeringRule {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            active: self.active,
            amount: self.amount,
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a peering rule in status new.
pub fn new_peering_rule(code: &str, label: &str) -> PeeringRule {
    PeeringRule {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        ceiling: 0.0,
        active: false,
        amount: 0.0,
        issuer: "".to_string(),
        grace_days: 0,
        timezone: "".to_string(),
        status: PeeringRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_peering_rule("code-20", "label-558");
        assert_eq!(item.status, PeeringRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_peering_rule("code-20", "label-558");
        if item.move_to(PeeringRuleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringRuleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_peering_rule("code-20", "label-558");
        item.status = PeeringRuleStatus::Failed;
        assert!(item.move_to(PeeringRuleStatus::New).is_err(), "expected move to fail");
    }
}
