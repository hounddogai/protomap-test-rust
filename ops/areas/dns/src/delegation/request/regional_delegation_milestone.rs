use super::*;

use acme_kit::text;

/// Where a regional delegation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDelegationMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalDelegationMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalDelegationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional delegation milestone as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestone {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub phone: String,
    pub tenant_id: String,
    pub attempts: i64,
    pub status: RegionalDelegationMilestoneStatus,
}

impl RegionalDelegationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDelegationMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDelegationMilestoneStatus) -> bool {
        match self.status {
            RegionalDelegationMilestoneStatus::Requested => {
                match next {
                    RegionalDelegationMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDelegationMilestoneStatus::Granted => {
                match next {
                    RegionalDelegationMilestoneStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDelegationMilestoneStatus::Revoked => {
                return next == RegionalDelegationMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDelegationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional delegation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDelegationMilestone {
        RegionalDelegationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional delegation milestone in status requested.
pub fn new_regional_delegation_milestone(code: &str, name: &str) -> RegionalDelegationMilestone {
    RegionalDelegationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        phone: "".to_string(),
        tenant_id: "".to_string(),
        attempts: 0,
        status: RegionalDelegationMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_delegation_milestone("code-634", "name-606");
        assert_eq!(item.status, RegionalDelegationMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_regional_delegation_milestone("code-634", "name-606");
        if item.move_to(RegionalDelegationMilestoneStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDelegationMilestoneStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_delegation_milestone("code-634", "name-606");
        item.status = RegionalDelegationMilestoneStatus::Expired;
        assert!(
            item.move_to(RegionalDelegationMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
