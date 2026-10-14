use super::*;

use acme_kit::text;

/// Where a rolling grant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingGrantAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingGrantAgreementStatus {
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

impl std::fmt::Display for RollingGrantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling grant agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantAgreement {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub rotation_days: i64,
    pub capacity: i64,
    pub key_id: String,
    pub ceiling: f64,
    pub limit: i64,
    pub reviewer: String,
    pub status: RollingGrantAgreementStatus,
}

impl RollingGrantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingGrantAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingGrantAgreementStatus) -> bool {
        match self.status {
            RollingGrantAgreementStatus::Open => {
                match next {
                    RollingGrantAgreementStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingGrantAgreementStatus::InReview => {
                return next == RollingGrantAgreementStatus::Reopened;
            }
            RollingGrantAgreementStatus::Resolved => {
                match next {
                    RollingGrantAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingGrantAgreementStatus::Reopened => {
                return next == RollingGrantAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingGrantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling grant agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling grant agreement", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingGrantAgreement {
        RollingGrantAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            rotation_days: self.rotation_days,
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            ceiling: self.ceiling,
            limit: self.limit,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling grant agreement in status open.
pub fn new_rolling_grant_agreement(code: &str, name: &str) -> RollingGrantAgreement {
    RollingGrantAgreement {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        rotation_days: 0,
        capacity: 0,
        key_id: "".to_string(),
        ceiling: 0.0,
        limit: 0,
        reviewer: "".to_string(),
        status: RollingGrantAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_grant_agreement("code-330", "name-237");
        assert_eq!(item.status, RollingGrantAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_rolling_grant_agreement("code-330", "name-237");
        if item.move_to(RollingGrantAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingGrantAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_grant_agreement("code-330", "name-237");
        item.status = RollingGrantAgreementStatus::Archived;
        assert!(
            item.move_to(RollingGrantAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
