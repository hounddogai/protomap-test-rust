use super::*;

use acme_kit::text;

/// Where a health check group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl HealthCheckGroupStatus {
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

impl std::fmt::Display for HealthCheckGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckGroup {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub timezone: String,
    pub principal: String,
    pub port: i64,
    pub rate: f64,
    pub cost: f64,
    pub status: HealthCheckGroupStatus,
}

impl HealthCheckGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckGroupStatus) -> bool {
        match self.status {
            HealthCheckGroupStatus::Open => {
                match next {
                    HealthCheckGroupStatus::InReview | HealthCheckGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckGroupStatus::InReview => {
                return next == HealthCheckGroupStatus::Resolved || next == HealthCheckGroupStatus::Archived;
            }
            HealthCheckGroupStatus::Resolved => {
                return next == HealthCheckGroupStatus::Archived;
            }
            HealthCheckGroupStatus::Reopened => {
                match next {
                    HealthCheckGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: HealthCheckGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check group", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckGroup {
        HealthCheckGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            port: self.port,
            rate: self.rate,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a health check group in status open.
pub fn new_health_check_group(code: &str, name: &str) -> HealthCheckGroup {
    HealthCheckGroup {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        timezone: "".to_string(),
        principal: "".to_string(),
        port: 0,
        rate: 0.0,
        cost: 0.0,
        status: HealthCheckGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_health_check_group("code-325", "name-231");
        assert_eq!(item.status, HealthCheckGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_health_check_group("code-325", "name-231");
        if item.move_to(HealthCheckGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_health_check_group("code-325", "name-231");
        item.status = HealthCheckGroupStatus::Archived;
        assert!(
            item.move_to(HealthCheckGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
