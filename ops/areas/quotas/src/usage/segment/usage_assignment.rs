use super::*;

use acme_kit::text;

/// Where a usage assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage assignment as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAssignment {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub key_id: String,
    pub timezone: String,
    pub visible: bool,
    pub status: UsageAssignmentStatus,
}

impl UsageAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageAssignmentStatus) -> bool {
        match self.status {
            UsageAssignmentStatus::Requested => {
                match next {
                    UsageAssignmentStatus::Revoked | UsageAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageAssignmentStatus::Granted => {
                return next == UsageAssignmentStatus::Revoked || next == UsageAssignmentStatus::Expired;
            }
            UsageAssignmentStatus::Revoked => {
                return next == UsageAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> UsageAssignment {
        UsageAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            timezone: self.timezone.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a usage assignment in status requested.
pub fn new_usage_assignment(code: &str, name: &str) -> UsageAssignment {
    UsageAssignment {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        key_id: "".to_string(),
        timezone: "".to_string(),
        visible: false,
        status: UsageAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_assignment("code-730", "name-161");
        assert_eq!(item.status, UsageAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_usage_assignment("code-730", "name-161");
        if item.move_to(UsageAssignmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageAssignmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_assignment("code-730", "name-161");
        item.status = UsageAssignmentStatus::Expired;
        assert!(
            item.move_to(UsageAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
