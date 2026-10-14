use super::*;

use acme_kit::text;

/// Where a strict dashboard milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDashboardMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictDashboardMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictDashboardMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict dashboard milestone as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestone {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub ratio: f64,
    pub host: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub status: StrictDashboardMilestoneStatus,
}

impl StrictDashboardMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDashboardMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDashboardMilestoneStatus) -> bool {
        match self.status {
            StrictDashboardMilestoneStatus::Created => {
                return next == StrictDashboardMilestoneStatus::Validated
                    || next == StrictDashboardMilestoneStatus::Published;
            }
            StrictDashboardMilestoneStatus::Validated => {
                return next == StrictDashboardMilestoneStatus::Published
                    || next == StrictDashboardMilestoneStatus::Retired;
            }
            StrictDashboardMilestoneStatus::Published => {
                match next {
                    StrictDashboardMilestoneStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictDashboardMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict dashboard milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ends_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDashboardMilestone {
        StrictDashboardMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            ratio: self.ratio,
            host: self.host.clone(),
            created_at: self.created_at,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict dashboard milestone in status created.
pub fn new_strict_dashboard_milestone(code: &str, name: &str) -> StrictDashboardMilestone {
    StrictDashboardMilestone {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        ratio: 0.0,
        host: "".to_string(),
        created_at: 0,
        ends_at: 0,
        status: StrictDashboardMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_dashboard_milestone("code-858", "name-607");
        assert_eq!(item.status, StrictDashboardMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_dashboard_milestone("code-858", "name-607");
        if item.move_to(StrictDashboardMilestoneStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictDashboardMilestoneStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_dashboard_milestone("code-858", "name-607");
        item.status = StrictDashboardMilestoneStatus::Retired;
        assert!(
            item.move_to(StrictDashboardMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
