use super::*;

use acme_kit::text;

/// Where a rolling identity milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingIdentityMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingIdentityMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingIdentityMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling identity milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestone {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub amount: f64,
    pub starts_at: i64,
    pub updated_at: i64,
    pub label: String,
    pub priority: i64,
    pub status: RollingIdentityMilestoneStatus,
}

impl RollingIdentityMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingIdentityMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingIdentityMilestoneStatus) -> bool {
        match self.status {
            RollingIdentityMilestoneStatus::Requested => {
                return next == RollingIdentityMilestoneStatus::Granted
                    || next == RollingIdentityMilestoneStatus::Expired;
            }
            RollingIdentityMilestoneStatus::Granted => {
                match next {
                    RollingIdentityMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIdentityMilestoneStatus::Revoked => {
                return next == RollingIdentityMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingIdentityMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling identity milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingIdentityMilestone {
        RollingIdentityMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            amount: self.amount,
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            label: self.label.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling identity milestone in status requested.
pub fn new_rolling_identity_milestone(code: &str, name: &str) -> RollingIdentityMilestone {
    RollingIdentityMilestone {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        amount: 0.0,
        starts_at: 0,
        updated_at: 0,
        label: "".to_string(),
        priority: 0,
        status: RollingIdentityMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_identity_milestone("code-4", "name-760");
        assert_eq!(item.status, RollingIdentityMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_identity_milestone("code-4", "name-760");
        if item.move_to(RollingIdentityMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingIdentityMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_identity_milestone("code-4", "name-760");
        item.status = RollingIdentityMilestoneStatus::Expired;
        assert!(
            item.move_to(RollingIdentityMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
