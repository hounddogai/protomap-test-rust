use super::*;

use acme_kit::text;

/// Where a scheduled health check milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledHealthCheckMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledHealthCheckMilestoneStatus {
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

impl std::fmt::Display for ScheduledHealthCheckMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled health check milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestone {
    pub code: String,
    pub title: String,
    pub source: String,
    pub approved: bool,
    pub quantity: i64,
    pub phone: String,
    pub amount: f64,
    pub tenant_id: String,
    pub status: ScheduledHealthCheckMilestoneStatus,
}

impl ScheduledHealthCheckMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledHealthCheckMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledHealthCheckMilestoneStatus) -> bool {
        match self.status {
            ScheduledHealthCheckMilestoneStatus::Open => {
                return next == ScheduledHealthCheckMilestoneStatus::InReview;
            }
            ScheduledHealthCheckMilestoneStatus::InReview => {
                return next == ScheduledHealthCheckMilestoneStatus::Resolved
                    || next == ScheduledHealthCheckMilestoneStatus::Reopened;
            }
            ScheduledHealthCheckMilestoneStatus::Resolved => {
                return next == ScheduledHealthCheckMilestoneStatus::Reopened;
            }
            ScheduledHealthCheckMilestoneStatus::Reopened => {
                return next == ScheduledHealthCheckMilestoneStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledHealthCheckMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled health check milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.amount * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledHealthCheckMilestone {
        ScheduledHealthCheckMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            approved: self.approved,
            quantity: self.quantity,
            phone: self.phone.clone(),
            amount: self.amount,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled health check milestone in status open.
pub fn new_scheduled_health_check_milestone(code: &str, title: &str) -> ScheduledHealthCheckMilestone {
    ScheduledHealthCheckMilestone {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        approved: false,
        quantity: 0,
        phone: "".to_string(),
        amount: 0.0,
        tenant_id: "".to_string(),
        status: ScheduledHealthCheckMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_health_check_milestone("code-741", "title-208");
        assert_eq!(item.status, ScheduledHealthCheckMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_health_check_milestone("code-741", "title-208");
        if item.move_to(ScheduledHealthCheckMilestoneStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledHealthCheckMilestoneStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_health_check_milestone("code-741", "title-208");
        item.status = ScheduledHealthCheckMilestoneStatus::Archived;
        assert!(
            item.move_to(ScheduledHealthCheckMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
