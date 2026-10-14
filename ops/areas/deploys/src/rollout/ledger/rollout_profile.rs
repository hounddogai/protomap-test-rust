use super::*;

use acme_kit::text;

/// Where a rollout profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutProfileStatus {
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

impl std::fmt::Display for RolloutProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProfile {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub starts_at: i64,
    pub archived: bool,
    pub status: RolloutProfileStatus,
}

impl RolloutProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutProfileStatus) -> bool {
        match self.status {
            RolloutProfileStatus::Open => {
                match next {
                    RolloutProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutProfileStatus::InReview => {
                match next {
                    RolloutProfileStatus::Resolved | RolloutProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutProfileStatus::Resolved => {
                match next {
                    RolloutProfileStatus::Reopened | RolloutProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutProfileStatus::Reopened => {
                return next == RolloutProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.maximum - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutProfile {
        RolloutProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            starts_at: self.starts_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollout profile in status open.
pub fn new_rollout_profile(code: &str, name: &str) -> RolloutProfile {
    RolloutProfile {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        starts_at: 0,
        archived: false,
        status: RolloutProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollout_profile("code-925", "name-144");
        assert_eq!(item.status, RolloutProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_rollout_profile("code-925", "name-144");
        if item.move_to(RolloutProfileStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutProfileStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollout_profile("code-925", "name-144");
        item.status = RolloutProfileStatus::Archived;
        assert!(
            item.move_to(RolloutProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
