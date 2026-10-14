use super::*;

use acme_kit::text;

/// Where a default delegation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDelegationAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultDelegationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultDelegationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default delegation allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationAllocation {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub pinned: bool,
    pub namespace: String,
    pub reviewer: String,
    pub revision: i64,
    pub cost: f64,
    pub status: DefaultDelegationAllocationStatus,
}

impl DefaultDelegationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDelegationAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDelegationAllocationStatus) -> bool {
        match self.status {
            DefaultDelegationAllocationStatus::Proposed => {
                return next == DefaultDelegationAllocationStatus::Approved;
            }
            DefaultDelegationAllocationStatus::Approved => {
                return next == DefaultDelegationAllocationStatus::Withdrawn;
            }
            DefaultDelegationAllocationStatus::Rejected => {
                match next {
                    DefaultDelegationAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultDelegationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default delegation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default delegation allocation", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.cost + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDelegationAllocation {
        DefaultDelegationAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            namespace: self.namespace.clone(),
            reviewer: self.reviewer.clone(),
            revision: self.revision,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default delegation allocation in status proposed.
pub fn new_default_delegation_allocation(code: &str, title: &str) -> DefaultDelegationAllocation {
    DefaultDelegationAllocation {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        pinned: false,
        namespace: "".to_string(),
        reviewer: "".to_string(),
        revision: 0,
        cost: 0.0,
        status: DefaultDelegationAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_delegation_allocation("code-479", "title-156");
        assert_eq!(item.status, DefaultDelegationAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_delegation_allocation("code-479", "title-156");
        if item.move_to(DefaultDelegationAllocationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDelegationAllocationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_delegation_allocation("code-479", "title-156");
        item.status = DefaultDelegationAllocationStatus::Withdrawn;
        assert!(
            item.move_to(DefaultDelegationAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
