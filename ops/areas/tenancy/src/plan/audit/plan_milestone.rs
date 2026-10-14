use super::*;

use acme_kit::text;

/// Where a plan milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan milestone as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestone {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub locale: String,
    pub ends_at: i64,
    pub summary: String,
    pub status: PlanMilestoneStatus,
}

impl PlanMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanMilestoneStatus) -> bool {
        match self.status {
            PlanMilestoneStatus::Proposed => {
                return next == PlanMilestoneStatus::Rejected;
            }
            PlanMilestoneStatus::Approved => {
                match next {
                    PlanMilestoneStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanMilestoneStatus::Rejected => {
                return next == PlanMilestoneStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PlanMilestone {
        PlanMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a plan milestone in status proposed.
pub fn new_plan_milestone(code: &str, label: &str) -> PlanMilestone {
    PlanMilestone {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        locale: "".to_string(),
        ends_at: 0,
        summary: "".to_string(),
        status: PlanMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_milestone("code-841", "label-335");
        assert_eq!(item.status, PlanMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_plan_milestone("code-841", "label-335");
        if item.move_to(PlanMilestoneStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanMilestoneStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_milestone("code-841", "label-335");
        item.status = PlanMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(PlanMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
