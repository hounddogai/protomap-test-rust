use super::*;

use acme_kit::text;

/// Where a delegation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAllocation {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub port: i64,
    pub fingerprint: String,
    pub version: i64,
    pub maximum: f64,
    pub status: DelegationAllocationStatus,
}

impl DelegationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationAllocationStatus) -> bool {
        match self.status {
            DelegationAllocationStatus::Proposed => {
                return next == DelegationAllocationStatus::Rejected || next == DelegationAllocationStatus::Withdrawn;
            }
            DelegationAllocationStatus::Approved => {
                match next {
                    DelegationAllocationStatus::Rejected | DelegationAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAllocationStatus::Rejected => {
                match next {
                    DelegationAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationAllocation {
        DelegationAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation allocation in status proposed.
pub fn new_delegation_allocation(code: &str, label: &str) -> DelegationAllocation {
    DelegationAllocation {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        port: 0,
        fingerprint: "".to_string(),
        version: 0,
        maximum: 0.0,
        status: DelegationAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_allocation("code-931", "label-46");
        assert_eq!(item.status, DelegationAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_delegation_allocation("code-931", "label-46");
        if item.move_to(DelegationAllocationStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationAllocationStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_allocation("code-931", "label-46");
        item.status = DelegationAllocationStatus::Withdrawn;
        assert!(
            item.move_to(DelegationAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
