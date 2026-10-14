use super::*;

use acme_kit::text;

/// Where a nonce allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl NonceAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for NonceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceAllocation {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub archived: bool,
    pub namespace: String,
    pub unit: String,
    pub amount: f64,
    pub status: NonceAllocationStatus,
}

impl NonceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceAllocationStatus) -> bool {
        match self.status {
            NonceAllocationStatus::Proposed => {
                return next == NonceAllocationStatus::Rejected || next == NonceAllocationStatus::Withdrawn;
            }
            NonceAllocationStatus::Approved => {
                match next {
                    NonceAllocationStatus::Rejected | NonceAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceAllocationStatus::Rejected => {
                return next == NonceAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> NonceAllocation {
        NonceAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a nonce allocation in status proposed.
pub fn new_nonce_allocation(code: &str, name: &str) -> NonceAllocation {
    NonceAllocation {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        archived: false,
        namespace: "".to_string(),
        unit: "".to_string(),
        amount: 0.0,
        status: NonceAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_nonce_allocation("code-50", "name-625");
        assert_eq!(item.status, NonceAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_nonce_allocation("code-50", "name-625");
        if item.move_to(NonceAllocationStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceAllocationStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_nonce_allocation("code-50", "name-625");
        item.status = NonceAllocationStatus::Withdrawn;
        assert!(
            item.move_to(NonceAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
