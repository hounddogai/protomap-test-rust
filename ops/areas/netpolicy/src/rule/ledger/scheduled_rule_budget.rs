use super::*;

use acme_kit::text;

/// Where a scheduled rule budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRuleBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledRuleBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledRuleBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rule budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleBudget {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub description: String,
    pub pinned: bool,
    pub margin: f64,
    pub port: i64,
    pub contact_email: String,
    pub rotation_days: i64,
    pub checksum: String,
    pub status: ScheduledRuleBudgetStatus,
}

impl ScheduledRuleBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRuleBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRuleBudgetStatus) -> bool {
        match self.status {
            ScheduledRuleBudgetStatus::Proposed => {
                return next == ScheduledRuleBudgetStatus::Approved || next == ScheduledRuleBudgetStatus::Withdrawn;
            }
            ScheduledRuleBudgetStatus::Approved => {
                match next {
                    ScheduledRuleBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRuleBudgetStatus::Rejected => {
                match next {
                    ScheduledRuleBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledRuleBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rule budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.discount + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRuleBudget {
        ScheduledRuleBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            description: self.description.clone(),
            pinned: self.pinned,
            margin: self.margin,
            port: self.port,
            contact_email: self.contact_email.clone(),
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled rule budget in status proposed.
pub fn new_scheduled_rule_budget(code: &str, title: &str) -> ScheduledRuleBudget {
    ScheduledRuleBudget {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        description: "".to_string(),
        pinned: false,
        margin: 0.0,
        port: 0,
        contact_email: "".to_string(),
        rotation_days: 0,
        checksum: "".to_string(),
        status: ScheduledRuleBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_rule_budget("code-168", "title-596");
        assert_eq!(item.status, ScheduledRuleBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_rule_budget("code-168", "title-596");
        if item.move_to(ScheduledRuleBudgetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRuleBudgetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_rule_budget("code-168", "title-596");
        item.status = ScheduledRuleBudgetStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledRuleBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
