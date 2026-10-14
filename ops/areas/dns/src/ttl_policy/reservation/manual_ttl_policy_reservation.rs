use super::*;

use acme_kit::text;

/// Where a manual ttl policy reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTtlPolicyReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualTtlPolicyReservationStatus {
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

impl std::fmt::Display for ManualTtlPolicyReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual ttl policy reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservation {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub category: String,
    pub priority: i64,
    pub ratio: f64,
    pub grace_days: i64,
    pub tenant_id: String,
    pub status: ManualTtlPolicyReservationStatus,
}

impl ManualTtlPolicyReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTtlPolicyReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTtlPolicyReservationStatus) -> bool {
        match self.status {
            ManualTtlPolicyReservationStatus::Open => {
                match next {
                    ManualTtlPolicyReservationStatus::Resolved | ManualTtlPolicyReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyReservationStatus::InReview => {
                match next {
                    ManualTtlPolicyReservationStatus::Reopened | ManualTtlPolicyReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyReservationStatus::Resolved => {
                match next {
                    ManualTtlPolicyReservationStatus::Reopened | ManualTtlPolicyReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyReservationStatus::Reopened => {
                return next == ManualTtlPolicyReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualTtlPolicyReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual ttl policy reservation {} cannot move from {} to {}",
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
        ((self.grace_days as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTtlPolicyReservation {
        ManualTtlPolicyReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            category: self.category.clone(),
            priority: self.priority,
            ratio: self.ratio,
            grace_days: self.grace_days,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual ttl policy reservation in status open.
pub fn new_manual_ttl_policy_reservation(code: &str, title: &str) -> ManualTtlPolicyReservation {
    ManualTtlPolicyReservation {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        category: "".to_string(),
        priority: 0,
        ratio: 0.0,
        grace_days: 0,
        tenant_id: "".to_string(),
        status: ManualTtlPolicyReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_ttl_policy_reservation("code-277", "title-101");
        assert_eq!(item.status, ManualTtlPolicyReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_manual_ttl_policy_reservation("code-277", "title-101");
        if item.move_to(ManualTtlPolicyReservationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTtlPolicyReservationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_ttl_policy_reservation("code-277", "title-101");
        item.status = ManualTtlPolicyReservationStatus::Archived;
        assert!(
            item.move_to(ManualTtlPolicyReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
