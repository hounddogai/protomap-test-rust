use super::*;

use acme_kit::text;

/// Where a weighted rollout profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRolloutProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedRolloutProfileStatus {
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

impl std::fmt::Display for WeightedRolloutProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted rollout profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfile {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub reviewer: String,
    pub source: String,
    pub ceiling: f64,
    pub approved: bool,
    pub weight: f64,
    pub burst: i64,
    pub status: WeightedRolloutProfileStatus,
}

impl WeightedRolloutProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRolloutProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRolloutProfileStatus) -> bool {
        match self.status {
            WeightedRolloutProfileStatus::Open => {
                return next == WeightedRolloutProfileStatus::InReview
                    || next == WeightedRolloutProfileStatus::Resolved;
            }
            WeightedRolloutProfileStatus::InReview => {
                match next {
                    WeightedRolloutProfileStatus::Resolved | WeightedRolloutProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRolloutProfileStatus::Resolved => {
                return next == WeightedRolloutProfileStatus::Reopened
                    || next == WeightedRolloutProfileStatus::Archived;
            }
            WeightedRolloutProfileStatus::Reopened => {
                return next == WeightedRolloutProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRolloutProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rollout profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRolloutProfile {
        WeightedRolloutProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            weight: self.weight,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted rollout profile in status open.
pub fn new_weighted_rollout_profile(code: &str, name: &str) -> WeightedRolloutProfile {
    WeightedRolloutProfile {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        reviewer: "".to_string(),
        source: "".to_string(),
        ceiling: 0.0,
        approved: false,
        weight: 0.0,
        burst: 0,
        status: WeightedRolloutProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_rollout_profile("code-769", "name-852");
        assert_eq!(item.status, WeightedRolloutProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_weighted_rollout_profile("code-769", "name-852");
        if item.move_to(WeightedRolloutProfileStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRolloutProfileStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_rollout_profile("code-769", "name-852");
        item.status = WeightedRolloutProfileStatus::Archived;
        assert!(
            item.move_to(WeightedRolloutProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
