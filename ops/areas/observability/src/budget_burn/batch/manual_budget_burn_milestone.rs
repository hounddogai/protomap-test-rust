use super::*;

use acme_kit::text;

/// Where a manual budget burn milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualBudgetBurnMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualBudgetBurnMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual budget burn milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnMilestone {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub tenant_id: String,
    pub priority: i64,
    pub limit: i64,
    pub namespace: String,
    pub version: i64,
    pub status: ManualBudgetBurnMilestoneStatus,
}

impl ManualBudgetBurnMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnMilestoneStatus) -> bool {
        match self.status {
            ManualBudgetBurnMilestoneStatus::Created => {
                return next == ManualBudgetBurnMilestoneStatus::Retired;
            }
            ManualBudgetBurnMilestoneStatus::Validated => {
                match next {
                    ManualBudgetBurnMilestoneStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnMilestoneStatus::Published => {
                match next {
                    ManualBudgetBurnMilestoneStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualBudgetBurnMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.limit as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBudgetBurnMilestone {
        ManualBudgetBurnMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            limit: self.limit,
            namespace: self.namespace.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual budget burn milestone in status created.
pub fn new_manual_budget_burn_milestone(code: &str, label: &str) -> ManualBudgetBurnMilestone {
    ManualBudgetBurnMilestone {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        tenant_id: "".to_string(),
        priority: 0,
        limit: 0,
        namespace: "".to_string(),
        version: 0,
        status: ManualBudgetBurnMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_budget_burn_milestone("code-427", "label-672");
        assert_eq!(item.status, ManualBudgetBurnMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_manual_budget_burn_milestone("code-427", "label-672");
        if item.move_to(ManualBudgetBurnMilestoneStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBudgetBurnMilestoneStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_budget_burn_milestone("code-427", "label-672");
        item.status = ManualBudgetBurnMilestoneStatus::Retired;
        assert!(
            item.move_to(ManualBudgetBurnMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
