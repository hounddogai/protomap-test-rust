use super::*;

use acme_kit::text;

/// Where a rolling rollout preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRolloutPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingRolloutPreferenceStatus {
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

impl std::fmt::Display for RollingRolloutPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling rollout preference as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutPreference {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub pinned: bool,
    pub floor_value: f64,
    pub revision: i64,
    pub host: String,
    pub status: RollingRolloutPreferenceStatus,
}

impl RollingRolloutPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRolloutPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRolloutPreferenceStatus) -> bool {
        match self.status {
            RollingRolloutPreferenceStatus::Open => {
                return next == RollingRolloutPreferenceStatus::Resolved
                    || next == RollingRolloutPreferenceStatus::Reopened;
            }
            RollingRolloutPreferenceStatus::InReview => {
                return next == RollingRolloutPreferenceStatus::Resolved;
            }
            RollingRolloutPreferenceStatus::Resolved => {
                return next == RollingRolloutPreferenceStatus::Reopened
                    || next == RollingRolloutPreferenceStatus::Archived;
            }
            RollingRolloutPreferenceStatus::Reopened => {
                return next == RollingRolloutPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRolloutPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollout preference {} cannot move from {} to {}",
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

    pub fn projected_volume(&self) -> f64 {
        ((self.floor_value - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRolloutPreference {
        RollingRolloutPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            floor_value: self.floor_value,
            revision: self.revision,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling rollout preference in status open.
pub fn new_rolling_rollout_preference(code: &str, name: &str) -> RollingRolloutPreference {
    RollingRolloutPreference {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        pinned: false,
        floor_value: 0.0,
        revision: 0,
        host: "".to_string(),
        status: RollingRolloutPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_rollout_preference("code-207", "name-250");
        assert_eq!(item.status, RollingRolloutPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_rollout_preference("code-207", "name-250");
        if item.move_to(RollingRolloutPreferenceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRolloutPreferenceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_rollout_preference("code-207", "name-250");
        item.status = RollingRolloutPreferenceStatus::Archived;
        assert!(
            item.move_to(RollingRolloutPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
