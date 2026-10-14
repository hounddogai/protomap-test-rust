use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a policy allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PolicyAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PolicyAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAllocation {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub port: i64,
    pub category: String,
    pub cost: f64,
    pub currency: String,
    pub status: PolicyAllocationStatus,
}

impl PolicyAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyAllocationStatus) -> bool {
        match self.status {
            PolicyAllocationStatus::Proposed => {
                match next {
                    PolicyAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAllocationStatus::Approved => {
                return next == PolicyAllocationStatus::Rejected || next == PolicyAllocationStatus::Withdrawn;
            }
            PolicyAllocationStatus::Rejected => {
                match next {
                    PolicyAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PolicyAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_value(&self) -> f64 {
        ((self.cost - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyAllocation {
        PolicyAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            port: self.port,
            category: self.category.clone(),
            cost: self.cost,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a policy allocation in status proposed.
pub fn new_policy_allocation(code: &str, name: &str) -> PolicyAllocation {
    PolicyAllocation {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        minimum: 0.0,
        attributes: BTreeMap::new(),
        port: 0,
        category: "".to_string(),
        cost: 0.0,
        currency: "".to_string(),
        status: PolicyAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_policy_allocation("code-633", "name-199");
        assert_eq!(item.status, PolicyAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_policy_allocation("code-633", "name-199");
        if item.move_to(PolicyAllocationStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyAllocationStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_policy_allocation("code-633", "name-199");
        item.status = PolicyAllocationStatus::Withdrawn;
        assert!(
            item.move_to(PolicyAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
