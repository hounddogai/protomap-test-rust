use super::*;

use acme_kit::text;

/// Where a delegation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DelegationAssignmentStatus {
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

impl std::fmt::Display for DelegationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignment {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: f64,
    pub key_id: String,
    pub port: i64,
    pub quantity: i64,
    pub status: DelegationAssignmentStatus,
}

impl DelegationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationAssignmentStatus) -> bool {
        match self.status {
            DelegationAssignmentStatus::Draft => {
                match next {
                    DelegationAssignmentStatus::Pending | DelegationAssignmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAssignmentStatus::Pending => {
                match next {
                    DelegationAssignmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAssignmentStatus::Active => {
                match next {
                    DelegationAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAssignmentStatus::Suspended => {
                return next == DelegationAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationAssignment {
        DelegationAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            score: self.score,
            key_id: self.key_id.clone(),
            port: self.port,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a delegation assignment in status draft.
pub fn new_delegation_assignment(code: &str, name: &str) -> DelegationAssignment {
    DelegationAssignment {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        score: 0.0,
        key_id: "".to_string(),
        port: 0,
        quantity: 0,
        status: DelegationAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_delegation_assignment("code-480", "name-390");
        assert_eq!(item.status, DelegationAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_delegation_assignment("code-480", "name-390");
        if item.move_to(DelegationAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_delegation_assignment("code-480", "name-390");
        item.status = DelegationAssignmentStatus::Closed;
        assert!(
            item.move_to(DelegationAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
