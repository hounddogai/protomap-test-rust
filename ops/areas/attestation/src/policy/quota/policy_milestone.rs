use super::*;

use acme_kit::text;

/// Where a policy milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMilestone {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub quota_units: i64,
    pub weight: f64,
    pub status: PolicyMilestoneStatus,
}

impl PolicyMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyMilestoneStatus) -> bool {
        match self.status {
            PolicyMilestoneStatus::Requested => {
                return next == PolicyMilestoneStatus::Revoked;
            }
            PolicyMilestoneStatus::Granted => {
                return next == PolicyMilestoneStatus::Revoked;
            }
            PolicyMilestoneStatus::Revoked => {
                match next {
                    PolicyMilestoneStatus::Expired => {
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

    pub fn move_to(&mut self, next: PolicyMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyMilestone {
        PolicyMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy milestone in status requested.
pub fn new_policy_milestone(code: &str, label: &str) -> PolicyMilestone {
    PolicyMilestone {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        quota_units: 0,
        weight: 0.0,
        status: PolicyMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_policy_milestone("code-512", "label-667");
        assert_eq!(item.status, PolicyMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_policy_milestone("code-512", "label-667");
        if item.move_to(PolicyMilestoneStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyMilestoneStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_policy_milestone("code-512", "label-667");
        item.status = PolicyMilestoneStatus::Expired;
        assert!(
            item.move_to(PolicyMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
