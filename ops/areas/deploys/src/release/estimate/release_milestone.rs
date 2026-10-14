use super::*;

use acme_kit::text;

/// Where a release milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReleaseMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReleaseMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestone {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub issuer: String,
    pub label: String,
    pub status: ReleaseMilestoneStatus,
}

impl ReleaseMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseMilestoneStatus) -> bool {
        match self.status {
            ReleaseMilestoneStatus::Requested => {
                return next == ReleaseMilestoneStatus::Granted || next == ReleaseMilestoneStatus::Revoked;
            }
            ReleaseMilestoneStatus::Granted => {
                return next == ReleaseMilestoneStatus::Revoked;
            }
            ReleaseMilestoneStatus::Revoked => {
                return next == ReleaseMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> ReleaseMilestone {
        ReleaseMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            issuer: self.issuer.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release milestone in status requested.
pub fn new_release_milestone(code: &str, name: &str) -> ReleaseMilestone {
    ReleaseMilestone {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        issuer: "".to_string(),
        label: "".to_string(),
        status: ReleaseMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_release_milestone("code-797", "name-839");
        assert_eq!(item.status, ReleaseMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_release_milestone("code-797", "name-839");
        if item.move_to(ReleaseMilestoneStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseMilestoneStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_release_milestone("code-797", "name-839");
        item.status = ReleaseMilestoneStatus::Expired;
        assert!(
            item.move_to(ReleaseMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
