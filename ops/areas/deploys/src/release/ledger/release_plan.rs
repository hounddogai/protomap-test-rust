use super::*;

use acme_kit::text;

/// Where a release plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleasePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleasePlanStatus {
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

impl std::fmt::Display for ReleasePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlan {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub ratio: f64,
    pub active: bool,
    pub external_ref: String,
    pub status: ReleasePlanStatus,
}

impl ReleasePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleasePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleasePlanStatus) -> bool {
        match self.status {
            ReleasePlanStatus::Open => {
                match next {
                    ReleasePlanStatus::InReview | ReleasePlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePlanStatus::InReview => {
                match next {
                    ReleasePlanStatus::Resolved | ReleasePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePlanStatus::Resolved => {
                match next {
                    ReleasePlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePlanStatus::Reopened => {
                return next == ReleasePlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleasePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleasePlan {
        ReleasePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            ratio: self.ratio,
            active: self.active,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release plan in status open.
pub fn new_release_plan(code: &str, title: &str) -> ReleasePlan {
    ReleasePlan {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        ratio: 0.0,
        active: false,
        external_ref: "".to_string(),
        status: ReleasePlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_plan("code-14", "title-316");
        assert_eq!(item.status, ReleasePlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_release_plan("code-14", "title-316");
        if item.move_to(ReleasePlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleasePlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_plan("code-14", "title-316");
        item.status = ReleasePlanStatus::Archived;
        assert!(item.move_to(ReleasePlanStatus::Open).is_err(), "expected move to fail");
    }
}
