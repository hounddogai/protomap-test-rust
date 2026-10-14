use super::*;

use acme_kit::text;

/// Where a seasonal slo milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalSloMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalSloMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalSloMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal slo milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloMilestone {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub ratio: f64,
    pub unit: String,
    pub category: String,
    pub status: SeasonalSloMilestoneStatus,
}

impl SeasonalSloMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalSloMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalSloMilestoneStatus) -> bool {
        match self.status {
            SeasonalSloMilestoneStatus::Proposed => {
                return next == SeasonalSloMilestoneStatus::Rejected || next == SeasonalSloMilestoneStatus::Withdrawn;
            }
            SeasonalSloMilestoneStatus::Approved => {
                match next {
                    SeasonalSloMilestoneStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSloMilestoneStatus::Rejected => {
                return next == SeasonalSloMilestoneStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalSloMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal slo milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quantity as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalSloMilestone {
        SeasonalSloMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            ratio: self.ratio,
            unit: self.unit.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal slo milestone in status proposed.
pub fn new_seasonal_slo_milestone(code: &str, title: &str) -> SeasonalSloMilestone {
    SeasonalSloMilestone {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        ratio: 0.0,
        unit: "".to_string(),
        category: "".to_string(),
        status: SeasonalSloMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_slo_milestone("code-447", "title-937");
        assert_eq!(item.status, SeasonalSloMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_slo_milestone("code-447", "title-937");
        if item.move_to(SeasonalSloMilestoneStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalSloMilestoneStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_slo_milestone("code-447", "title-937");
        item.status = SeasonalSloMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalSloMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
