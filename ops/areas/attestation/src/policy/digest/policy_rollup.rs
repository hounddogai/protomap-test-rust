use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a policy rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PolicyRollupStatus {
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

impl std::fmt::Display for PolicyRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy rollup as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRollup {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub owner_id: String,
    pub pinned: bool,
    pub attributes: BTreeMap<String, String>,
    pub notes: String,
    pub status: PolicyRollupStatus,
}

impl PolicyRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyRollupStatus) -> bool {
        match self.status {
            PolicyRollupStatus::Draft => {
                match next {
                    PolicyRollupStatus::Pending | PolicyRollupStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyRollupStatus::Pending => {
                return next == PolicyRollupStatus::Active || next == PolicyRollupStatus::Suspended;
            }
            PolicyRollupStatus::Active => {
                return next == PolicyRollupStatus::Suspended || next == PolicyRollupStatus::Closed;
            }
            PolicyRollupStatus::Suspended => {
                return next == PolicyRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy rollup", self.code)
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

    pub fn copy(&self) -> PolicyRollup {
        PolicyRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            owner_id: self.owner_id.clone(),
            pinned: self.pinned,
            attributes: self.attributes.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy rollup in status draft.
pub fn new_policy_rollup(code: &str, title: &str) -> PolicyRollup {
    PolicyRollup {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        owner_id: "".to_string(),
        pinned: false,
        attributes: BTreeMap::new(),
        notes: "".to_string(),
        status: PolicyRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_policy_rollup("code-163", "title-368");
        assert_eq!(item.status, PolicyRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_policy_rollup("code-163", "title-368");
        if item.move_to(PolicyRollupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyRollupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_policy_rollup("code-163", "title-368");
        item.status = PolicyRollupStatus::Closed;
        assert!(
            item.move_to(PolicyRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
