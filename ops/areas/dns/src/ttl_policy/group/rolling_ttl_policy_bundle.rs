use super::*;

use acme_kit::text;

/// Where a rolling ttl policy bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTtlPolicyBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingTtlPolicyBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingTtlPolicyBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling ttl policy bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundle {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub principal: String,
    pub contact_email: String,
    pub cost: f64,
    pub burst: i64,
    pub ratio: f64,
    pub status: RollingTtlPolicyBundleStatus,
}

impl RollingTtlPolicyBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTtlPolicyBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTtlPolicyBundleStatus) -> bool {
        match self.status {
            RollingTtlPolicyBundleStatus::Proposed => {
                return next == RollingTtlPolicyBundleStatus::Approved
                    || next == RollingTtlPolicyBundleStatus::Rejected;
            }
            RollingTtlPolicyBundleStatus::Approved => {
                return next == RollingTtlPolicyBundleStatus::Withdrawn;
            }
            RollingTtlPolicyBundleStatus::Rejected => {
                return next == RollingTtlPolicyBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingTtlPolicyBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling ttl policy bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.burst as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTtlPolicyBundle {
        RollingTtlPolicyBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            burst: self.burst,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling ttl policy bundle in status proposed.
pub fn new_rolling_ttl_policy_bundle(code: &str, title: &str) -> RollingTtlPolicyBundle {
    RollingTtlPolicyBundle {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        principal: "".to_string(),
        contact_email: "".to_string(),
        cost: 0.0,
        burst: 0,
        ratio: 0.0,
        status: RollingTtlPolicyBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_ttl_policy_bundle("code-102", "title-166");
        assert_eq!(item.status, RollingTtlPolicyBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_ttl_policy_bundle("code-102", "title-166");
        if item.move_to(RollingTtlPolicyBundleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTtlPolicyBundleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_ttl_policy_bundle("code-102", "title-166");
        item.status = RollingTtlPolicyBundleStatus::Withdrawn;
        assert!(
            item.move_to(RollingTtlPolicyBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
