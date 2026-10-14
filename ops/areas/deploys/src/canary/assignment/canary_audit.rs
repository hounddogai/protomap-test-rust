use super::*;

use acme_kit::text;

/// Where a canary audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryAuditStatus {
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

impl std::fmt::Display for CanaryAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAudit {
    pub code: String,
    pub name: String,
    pub title: String,
    pub port: i64,
    pub ceiling: f64,
    pub cost: f64,
    pub country: String,
    pub approved: bool,
    pub expires_at: i64,
    pub timezone: String,
    pub status: CanaryAuditStatus,
}

impl CanaryAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryAuditStatus) -> bool {
        match self.status {
            CanaryAuditStatus::Open => {
                match next {
                    CanaryAuditStatus::InReview | CanaryAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAuditStatus::InReview => {
                match next {
                    CanaryAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAuditStatus::Resolved => {
                match next {
                    CanaryAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAuditStatus::Reopened => {
                return next == CanaryAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary audit {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        (self.cost * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryAudit {
        CanaryAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            port: self.port,
            ceiling: self.ceiling,
            cost: self.cost,
            country: self.country.clone(),
            approved: self.approved,
            expires_at: self.expires_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a canary audit in status open.
pub fn new_canary_audit(code: &str, name: &str) -> CanaryAudit {
    CanaryAudit {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        port: 0,
        ceiling: 0.0,
        cost: 0.0,
        country: "".to_string(),
        approved: false,
        expires_at: 0,
        timezone: "".to_string(),
        status: CanaryAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_canary_audit("code-290", "name-700");
        assert_eq!(item.status, CanaryAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_canary_audit("code-290", "name-700");
        if item.move_to(CanaryAuditStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryAuditStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_canary_audit("code-290", "name-700");
        item.status = CanaryAuditStatus::Archived;
        assert!(item.move_to(CanaryAuditStatus::Open).is_err(), "expected move to fail");
    }
}
