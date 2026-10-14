use super::*;

use acme_kit::text;

/// Where a delegation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DelegationTargetStatus {
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

impl std::fmt::Display for DelegationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation target as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTarget {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub active: bool,
    pub revision: i64,
    pub port: i64,
    pub namespace: String,
    pub status: DelegationTargetStatus,
}

impl DelegationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationTargetStatus) -> bool {
        match self.status {
            DelegationTargetStatus::Draft => {
                return next == DelegationTargetStatus::Active || next == DelegationTargetStatus::Closed;
            }
            DelegationTargetStatus::Pending => {
                return next == DelegationTargetStatus::Active;
            }
            DelegationTargetStatus::Active => {
                match next {
                    DelegationTargetStatus::Suspended | DelegationTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationTargetStatus::Suspended => {
                match next {
                    DelegationTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: DelegationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.port as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationTarget {
        DelegationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            active: self.active,
            revision: self.revision,
            port: self.port,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a delegation target in status draft.
pub fn new_delegation_target(code: &str, name: &str) -> DelegationTarget {
    DelegationTarget {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        active: false,
        revision: 0,
        port: 0,
        namespace: "".to_string(),
        status: DelegationTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_delegation_target("code-416", "name-171");
        assert_eq!(item.status, DelegationTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_delegation_target("code-416", "name-171");
        if item.move_to(DelegationTargetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationTargetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_delegation_target("code-416", "name-171");
        item.status = DelegationTargetStatus::Closed;
        assert!(
            item.move_to(DelegationTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
