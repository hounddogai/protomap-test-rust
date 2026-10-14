use super::*;

use acme_kit::text;

/// Where a policy assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PolicyAssignmentStatus {
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

impl std::fmt::Display for PolicyAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAssignment {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub rotation_days: i64,
    pub pinned: bool,
    pub quota_units: i64,
    pub burst: i64,
    pub description: String,
    pub ends_at: i64,
    pub discount: f64,
    pub name: String,
    pub status: PolicyAssignmentStatus,
}

impl PolicyAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyAssignmentStatus) -> bool {
        match self.status {
            PolicyAssignmentStatus::Draft => {
                match next {
                    PolicyAssignmentStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAssignmentStatus::Pending => {
                return next == PolicyAssignmentStatus::Active || next == PolicyAssignmentStatus::Closed;
            }
            PolicyAssignmentStatus::Active => {
                return next == PolicyAssignmentStatus::Suspended || next == PolicyAssignmentStatus::Closed;
            }
            PolicyAssignmentStatus::Suspended => {
                match next {
                    PolicyAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: PolicyAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyAssignment {
        PolicyAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            quota_units: self.quota_units,
            burst: self.burst,
            description: self.description.clone(),
            ends_at: self.ends_at,
            discount: self.discount,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy assignment in status draft.
pub fn new_policy_assignment(code: &str, title: &str) -> PolicyAssignment {
    PolicyAssignment {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        rotation_days: 0,
        pinned: false,
        quota_units: 0,
        burst: 0,
        description: "".to_string(),
        ends_at: 0,
        discount: 0.0,
        name: "".to_string(),
        status: PolicyAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_policy_assignment("code-906", "title-453");
        assert_eq!(item.status, PolicyAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_policy_assignment("code-906", "title-453");
        if item.move_to(PolicyAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_policy_assignment("code-906", "title-453");
        item.status = PolicyAssignmentStatus::Closed;
        assert!(
            item.move_to(PolicyAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
