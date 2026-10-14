use super::*;

use acme_kit::text;

/// Where a default delegation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDelegationOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultDelegationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultDelegationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default delegation override as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationOverride {
    pub code: String,
    pub name: String,
    pub description: String,
    pub cipher: String,
    pub discount: f64,
    pub status: DefaultDelegationOverrideStatus,
}

impl DefaultDelegationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDelegationOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDelegationOverrideStatus) -> bool {
        match self.status {
            DefaultDelegationOverrideStatus::Proposed => {
                return next == DefaultDelegationOverrideStatus::Approved;
            }
            DefaultDelegationOverrideStatus::Approved => {
                return next == DefaultDelegationOverrideStatus::Rejected;
            }
            DefaultDelegationOverrideStatus::Rejected => {
                return next == DefaultDelegationOverrideStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultDelegationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default delegation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default delegation override", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> DefaultDelegationOverride {
        DefaultDelegationOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            cipher: self.cipher.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default delegation override in status proposed.
pub fn new_default_delegation_override(code: &str, name: &str) -> DefaultDelegationOverride {
    DefaultDelegationOverride {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        cipher: "".to_string(),
        discount: 0.0,
        status: DefaultDelegationOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_delegation_override("code-715", "name-15");
        assert_eq!(item.status, DefaultDelegationOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_default_delegation_override("code-715", "name-15");
        if item.move_to(DefaultDelegationOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDelegationOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_delegation_override("code-715", "name-15");
        item.status = DefaultDelegationOverrideStatus::Withdrawn;
        assert!(
            item.move_to(DefaultDelegationOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
