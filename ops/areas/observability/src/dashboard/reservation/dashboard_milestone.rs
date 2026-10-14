use super::*;

use acme_kit::text;

/// Where a dashboard milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DashboardMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DashboardMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestone {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub source: String,
    pub quota_units: i64,
    pub discount: f64,
    pub amount: f64,
    pub archived: bool,
    pub checksum: String,
    pub cost: f64,
    pub status: DashboardMilestoneStatus,
}

impl DashboardMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardMilestoneStatus) -> bool {
        match self.status {
            DashboardMilestoneStatus::Requested => {
                return next == DashboardMilestoneStatus::Granted;
            }
            DashboardMilestoneStatus::Granted => {
                return next == DashboardMilestoneStatus::Revoked || next == DashboardMilestoneStatus::Expired;
            }
            DashboardMilestoneStatus::Revoked => {
                return next == DashboardMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard milestone", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.discount - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardMilestone {
        DashboardMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            source: self.source.clone(),
            quota_units: self.quota_units,
            discount: self.discount,
            amount: self.amount,
            archived: self.archived,
            checksum: self.checksum.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a dashboard milestone in status requested.
pub fn new_dashboard_milestone(code: &str, name: &str) -> DashboardMilestone {
    DashboardMilestone {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        source: "".to_string(),
        quota_units: 0,
        discount: 0.0,
        amount: 0.0,
        archived: false,
        checksum: "".to_string(),
        cost: 0.0,
        status: DashboardMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dashboard_milestone("code-338", "name-366");
        assert_eq!(item.status, DashboardMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_dashboard_milestone("code-338", "name-366");
        if item.move_to(DashboardMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dashboard_milestone("code-338", "name-366");
        item.status = DashboardMilestoneStatus::Expired;
        assert!(
            item.move_to(DashboardMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
