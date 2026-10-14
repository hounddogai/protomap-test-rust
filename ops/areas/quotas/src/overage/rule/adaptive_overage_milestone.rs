use super::*;

use acme_kit::text;

/// Where a adaptive overage milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveOverageMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveOverageMilestoneStatus {
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

impl std::fmt::Display for AdaptiveOverageMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive overage milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageMilestone {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub sequence: i64,
    pub priority: i64,
    pub channel: String,
    pub window_days: i64,
    pub status: AdaptiveOverageMilestoneStatus,
}

impl AdaptiveOverageMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveOverageMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveOverageMilestoneStatus) -> bool {
        match self.status {
            AdaptiveOverageMilestoneStatus::Open => {
                return next == AdaptiveOverageMilestoneStatus::InReview
                    || next == AdaptiveOverageMilestoneStatus::Resolved;
            }
            AdaptiveOverageMilestoneStatus::InReview => {
                return next == AdaptiveOverageMilestoneStatus::Resolved;
            }
            AdaptiveOverageMilestoneStatus::Resolved => {
                return next == AdaptiveOverageMilestoneStatus::Reopened
                    || next == AdaptiveOverageMilestoneStatus::Archived;
            }
            AdaptiveOverageMilestoneStatus::Reopened => {
                return next == AdaptiveOverageMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveOverageMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive overage milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive overage milestone", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveOverageMilestone {
        AdaptiveOverageMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            sequence: self.sequence,
            priority: self.priority,
            channel: self.channel.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive overage milestone in status open.
pub fn new_adaptive_overage_milestone(code: &str, label: &str) -> AdaptiveOverageMilestone {
    AdaptiveOverageMilestone {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        sequence: 0,
        priority: 0,
        channel: "".to_string(),
        window_days: 0,
        status: AdaptiveOverageMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_overage_milestone("code-771", "label-64");
        assert_eq!(item.status, AdaptiveOverageMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_overage_milestone("code-771", "label-64");
        if item.move_to(AdaptiveOverageMilestoneStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveOverageMilestoneStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_overage_milestone("code-771", "label-64");
        item.status = AdaptiveOverageMilestoneStatus::Archived;
        assert!(
            item.move_to(AdaptiveOverageMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
