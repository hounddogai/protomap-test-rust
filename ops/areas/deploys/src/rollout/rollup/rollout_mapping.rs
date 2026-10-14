use super::*;

use acme_kit::text;

/// Where a rollout mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutMappingStatus {
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

impl std::fmt::Display for RolloutMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMapping {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub grace_days: i64,
    pub margin: f64,
    pub unit: String,
    pub threshold: f64,
    pub contact_email: String,
    pub revision: i64,
    pub notes: String,
    pub status: RolloutMappingStatus,
}

impl RolloutMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutMappingStatus) -> bool {
        match self.status {
            RolloutMappingStatus::Open => {
                return next == RolloutMappingStatus::Resolved || next == RolloutMappingStatus::Reopened;
            }
            RolloutMappingStatus::InReview => {
                return next == RolloutMappingStatus::Reopened;
            }
            RolloutMappingStatus::Resolved => {
                match next {
                    RolloutMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutMappingStatus::Reopened => {
                match next {
                    RolloutMappingStatus::Archived => {
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

    pub fn move_to(&mut self, next: RolloutMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutMapping {
        RolloutMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            margin: self.margin,
            unit: self.unit.clone(),
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            revision: self.revision,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollout mapping in status open.
pub fn new_rollout_mapping(code: &str, name: &str) -> RolloutMapping {
    RolloutMapping {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        grace_days: 0,
        margin: 0.0,
        unit: "".to_string(),
        threshold: 0.0,
        contact_email: "".to_string(),
        revision: 0,
        notes: "".to_string(),
        status: RolloutMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollout_mapping("code-912", "name-144");
        assert_eq!(item.status, RolloutMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rollout_mapping("code-912", "name-144");
        if item.move_to(RolloutMappingStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutMappingStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollout_mapping("code-912", "name-144");
        item.status = RolloutMappingStatus::Archived;
        assert!(
            item.move_to(RolloutMappingStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
