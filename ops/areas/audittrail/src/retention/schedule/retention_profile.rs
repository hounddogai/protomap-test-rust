use super::*;

use acme_kit::text;

/// Where a retention profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfile {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub ends_at: i64,
    pub name: String,
    pub scope: String,
    pub quota_units: i64,
    pub attempts: i64,
    pub segment: String,
    pub status: RetentionProfileStatus,
}

impl RetentionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionProfileStatus) -> bool {
        match self.status {
            RetentionProfileStatus::Proposed => {
                return next == RetentionProfileStatus::Rejected || next == RetentionProfileStatus::Withdrawn;
            }
            RetentionProfileStatus::Approved => {
                match next {
                    RetentionProfileStatus::Rejected | RetentionProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionProfileStatus::Rejected => {
                match next {
                    RetentionProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RetentionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention profile {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.attempts as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionProfile {
        RetentionProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            name: self.name.clone(),
            scope: self.scope.clone(),
            quota_units: self.quota_units,
            attempts: self.attempts,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a retention profile in status proposed.
pub fn new_retention_profile(code: &str, title: &str) -> RetentionProfile {
    RetentionProfile {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        ends_at: 0,
        name: "".to_string(),
        scope: "".to_string(),
        quota_units: 0,
        attempts: 0,
        segment: "".to_string(),
        status: RetentionProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_profile("code-186", "title-107");
        assert_eq!(item.status, RetentionProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_retention_profile("code-186", "title-107");
        if item.move_to(RetentionProfileStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionProfileStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_profile("code-186", "title-107");
        item.status = RetentionProfileStatus::Withdrawn;
        assert!(
            item.move_to(RetentionProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
