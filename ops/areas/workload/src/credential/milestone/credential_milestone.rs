use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a credential milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CredentialMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CredentialMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMilestone {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub description: String,
    pub status: CredentialMilestoneStatus,
}

impl CredentialMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialMilestoneStatus) -> bool {
        match self.status {
            CredentialMilestoneStatus::Draft => {
                return next == CredentialMilestoneStatus::Closed;
            }
            CredentialMilestoneStatus::Pending => {
                match next {
                    CredentialMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialMilestoneStatus::Active => {
                return next == CredentialMilestoneStatus::Suspended || next == CredentialMilestoneStatus::Closed;
            }
            CredentialMilestoneStatus::Suspended => {
                match next {
                    CredentialMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: CredentialMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> CredentialMilestone {
        CredentialMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential milestone in status draft.
pub fn new_credential_milestone(code: &str, name: &str) -> CredentialMilestone {
    CredentialMilestone {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        score: 0.0,
        description: "".to_string(),
        status: CredentialMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_credential_milestone("code-698", "name-171");
        assert_eq!(item.status, CredentialMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_credential_milestone("code-698", "name-171");
        if item.move_to(CredentialMilestoneStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialMilestoneStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_credential_milestone("code-698", "name-171");
        item.status = CredentialMilestoneStatus::Closed;
        assert!(
            item.move_to(CredentialMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
