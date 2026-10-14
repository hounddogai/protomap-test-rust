use super::*;

use acme_kit::text;

/// Where a dashboard template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DashboardTemplateStatus {
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

impl std::fmt::Display for DashboardTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard template as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTemplate {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub archived: bool,
    pub scope: String,
    pub active: bool,
    pub batch_size: i64,
    pub quota_units: i64,
    pub status: DashboardTemplateStatus,
}

impl DashboardTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardTemplateStatus) -> bool {
        match self.status {
            DashboardTemplateStatus::Open => {
                match next {
                    DashboardTemplateStatus::Resolved | DashboardTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardTemplateStatus::InReview => {
                return next == DashboardTemplateStatus::Reopened || next == DashboardTemplateStatus::Archived;
            }
            DashboardTemplateStatus::Resolved => {
                match next {
                    DashboardTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardTemplateStatus::Reopened => {
                match next {
                    DashboardTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: DashboardTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard template", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardTemplate {
        DashboardTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            archived: self.archived,
            scope: self.scope.clone(),
            active: self.active,
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a dashboard template in status open.
pub fn new_dashboard_template(code: &str, name: &str) -> DashboardTemplate {
    DashboardTemplate {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        archived: false,
        scope: "".to_string(),
        active: false,
        batch_size: 0,
        quota_units: 0,
        status: DashboardTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dashboard_template("code-19", "name-453");
        assert_eq!(item.status, DashboardTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_dashboard_template("code-19", "name-453");
        if item.move_to(DashboardTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dashboard_template("code-19", "name-453");
        item.status = DashboardTemplateStatus::Archived;
        assert!(
            item.move_to(DashboardTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
