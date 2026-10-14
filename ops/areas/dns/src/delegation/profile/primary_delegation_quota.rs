use super::*;

use acme_kit::text;

/// Where a primary delegation quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDelegationQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryDelegationQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryDelegationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary delegation quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationQuota {
    pub code: String,
    pub name: String,
    pub region: String,
    pub channel: String,
    pub ceiling: f64,
    pub status: PrimaryDelegationQuotaStatus,
}

impl PrimaryDelegationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDelegationQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDelegationQuotaStatus) -> bool {
        match self.status {
            PrimaryDelegationQuotaStatus::Proposed => {
                match next {
                    PrimaryDelegationQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDelegationQuotaStatus::Approved => {
                return next == PrimaryDelegationQuotaStatus::Rejected;
            }
            PrimaryDelegationQuotaStatus::Rejected => {
                return next == PrimaryDelegationQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDelegationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary delegation quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> PrimaryDelegationQuota {
        PrimaryDelegationQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            channel: self.channel.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary delegation quota in status proposed.
pub fn new_primary_delegation_quota(code: &str, name: &str) -> PrimaryDelegationQuota {
    PrimaryDelegationQuota {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        channel: "".to_string(),
        ceiling: 0.0,
        status: PrimaryDelegationQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_delegation_quota("code-143", "name-853");
        assert_eq!(item.status, PrimaryDelegationQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_primary_delegation_quota("code-143", "name-853");
        if item.move_to(PrimaryDelegationQuotaStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDelegationQuotaStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_delegation_quota("code-143", "name-853");
        item.status = PrimaryDelegationQuotaStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryDelegationQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
