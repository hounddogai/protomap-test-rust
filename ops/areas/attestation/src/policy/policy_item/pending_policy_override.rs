use super::*;

use acme_kit::text;

/// Where a pending policy override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPolicyOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingPolicyOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PendingPolicyOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending policy override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyOverride {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub minimum: f64,
    pub checksum: String,
    pub unit: String,
    pub status: PendingPolicyOverrideStatus,
}

impl PendingPolicyOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPolicyOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPolicyOverrideStatus) -> bool {
        match self.status {
            PendingPolicyOverrideStatus::Open => {
                return next == PendingPolicyOverrideStatus::Archived;
            }
            PendingPolicyOverrideStatus::InReview => {
                match next {
                    PendingPolicyOverrideStatus::Resolved | PendingPolicyOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyOverrideStatus::Resolved => {
                match next {
                    PendingPolicyOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicyOverrideStatus::Reopened => {
                match next {
                    PendingPolicyOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingPolicyOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending policy override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> PendingPolicyOverride {
        PendingPolicyOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending policy override in status open.
pub fn new_pending_policy_override(code: &str, name: &str) -> PendingPolicyOverride {
    PendingPolicyOverride {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        minimum: 0.0,
        checksum: "".to_string(),
        unit: "".to_string(),
        status: PendingPolicyOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_policy_override("code-474", "name-873");
        assert_eq!(item.status, PendingPolicyOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_pending_policy_override("code-474", "name-873");
        if item.move_to(PendingPolicyOverrideStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPolicyOverrideStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_policy_override("code-474", "name-873");
        item.status = PendingPolicyOverrideStatus::Archived;
        assert!(
            item.move_to(PendingPolicyOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
