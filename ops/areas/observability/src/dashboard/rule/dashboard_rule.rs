use super::*;

use acme_kit::text;

/// Where a dashboard rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardRuleStatus {
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

impl std::fmt::Display for DashboardRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRule {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub country: String,
    pub quota_units: i64,
    pub starts_at: i64,
    pub scope: String,
    pub expires_at: i64,
    pub approved: bool,
    pub category: String,
    pub status: DashboardRuleStatus,
}

impl DashboardRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardRuleStatus) -> bool {
        match self.status {
            DashboardRuleStatus::New => {
                match next {
                    DashboardRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRuleStatus::Queued => {
                match next {
                    DashboardRuleStatus::Running | DashboardRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRuleStatus::Running => {
                match next {
                    DashboardRuleStatus::Done | DashboardRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRuleStatus::Done => {
                match next {
                    DashboardRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: DashboardRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard rule", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quota_units as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardRule {
        DashboardRule {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            expires_at: self.expires_at,
            approved: self.approved,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a dashboard rule in status new.
pub fn new_dashboard_rule(code: &str, name: &str) -> DashboardRule {
    DashboardRule {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        country: "".to_string(),
        quota_units: 0,
        starts_at: 0,
        scope: "".to_string(),
        expires_at: 0,
        approved: false,
        category: "".to_string(),
        status: DashboardRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_rule("code-625", "name-387");
        assert_eq!(item.status, DashboardRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_dashboard_rule("code-625", "name-387");
        if item.move_to(DashboardRuleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardRuleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_rule("code-625", "name-387");
        item.status = DashboardRuleStatus::Failed;
        assert!(item.move_to(DashboardRuleStatus::New).is_err(), "expected move to fail");
    }
}
