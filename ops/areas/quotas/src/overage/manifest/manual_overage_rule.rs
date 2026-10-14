use super::*;

use acme_kit::text;

/// Where a manual overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualOverageRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualOverageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualOverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual overage rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRule {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub rate: f64,
    pub unit: String,
    pub sequence: i64,
    pub currency: String,
    pub external_ref: String,
    pub approved: bool,
    pub name: String,
    pub minimum: f64,
    pub status: ManualOverageRuleStatus,
}

impl ManualOverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualOverageRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualOverageRuleStatus) -> bool {
        match self.status {
            ManualOverageRuleStatus::Proposed => {
                return next == ManualOverageRuleStatus::Withdrawn;
            }
            ManualOverageRuleStatus::Approved => {
                match next {
                    ManualOverageRuleStatus::Rejected | ManualOverageRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualOverageRuleStatus::Rejected => {
                return next == ManualOverageRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualOverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual overage rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual overage rule", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.minimum * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualOverageRule {
        ManualOverageRule {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            rate: self.rate,
            unit: self.unit.clone(),
            sequence: self.sequence,
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            name: self.name.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual overage rule in status proposed.
pub fn new_manual_overage_rule(code: &str, title: &str) -> ManualOverageRule {
    ManualOverageRule {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        rate: 0.0,
        unit: "".to_string(),
        sequence: 0,
        currency: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        name: "".to_string(),
        minimum: 0.0,
        status: ManualOverageRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_overage_rule("code-333", "title-998");
        assert_eq!(item.status, ManualOverageRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_overage_rule("code-333", "title-998");
        if item.move_to(ManualOverageRuleStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualOverageRuleStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_overage_rule("code-333", "title-998");
        item.status = ManualOverageRuleStatus::Withdrawn;
        assert!(
            item.move_to(ManualOverageRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
