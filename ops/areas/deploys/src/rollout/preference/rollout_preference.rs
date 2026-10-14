use super::*;

use acme_kit::text;

/// Where a rollout preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutPreferenceStatus {
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

impl std::fmt::Display for RolloutPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreference {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub ends_at: i64,
    pub floor_value: f64,
    pub priority: i64,
    pub country: String,
    pub summary: String,
    pub status: RolloutPreferenceStatus,
}

impl RolloutPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutPreferenceStatus) -> bool {
        match self.status {
            RolloutPreferenceStatus::Open => {
                match next {
                    RolloutPreferenceStatus::Reopened | RolloutPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutPreferenceStatus::InReview => {
                return next == RolloutPreferenceStatus::Reopened || next == RolloutPreferenceStatus::Archived;
            }
            RolloutPreferenceStatus::Resolved => {
                return next == RolloutPreferenceStatus::Reopened || next == RolloutPreferenceStatus::Archived;
            }
            RolloutPreferenceStatus::Reopened => {
                return next == RolloutPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout preference {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.floor_value - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutPreference {
        RolloutPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            priority: self.priority,
            country: self.country.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rollout preference in status open.
pub fn new_rollout_preference(code: &str, name: &str) -> RolloutPreference {
    RolloutPreference {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        ends_at: 0,
        floor_value: 0.0,
        priority: 0,
        country: "".to_string(),
        summary: "".to_string(),
        status: RolloutPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollout_preference("code-260", "name-557");
        assert_eq!(item.status, RolloutPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rollout_preference("code-260", "name-557");
        if item.move_to(RolloutPreferenceStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutPreferenceStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollout_preference("code-260", "name-557");
        item.status = RolloutPreferenceStatus::Archived;
        assert!(
            item.move_to(RolloutPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
